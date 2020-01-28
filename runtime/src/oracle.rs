use rstd::prelude::*;

use codec::{Decode, Encode};
use rstd::cmp::{Ord, Ordering};
use rstd::collections::btree_map::BTreeMap;
use sr_primitives::traits::One;

pub use crate::external_value::*;
pub use crate::module_trait::*;
pub use crate::period_handler::PeriodHandler;

use crate::median::{get_median, Median};

#[derive(Encode, Decode, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Oracle<T: Trait>
{
    pub name: RawString,
    pub table: TableId<T>,

    sources_threshold: u8,
    pub period_handler: PeriodHandler<T::Moment>,

    pub assets_name: AssetsVec<RawString>,

    pub sources: BTreeMap<AccountId<T>, AssetsVec<ExternalValue<T>>>,
    pub value: AssetsVec<ExternalValue<T>>,
}

#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
pub enum OracleError
{
    CalculationError,
    TooFewCommitedValue(usize, usize),
    WrongAssetId(usize),
    TooFewAccounts(usize, usize),
    AccountAccess,
}

impl OracleError
{
    pub fn to_str(&self) -> &'static str
    {
        match self
        {
            Self::WrongAssetId(_) => "Wrong asset id.",
            Self::TooFewAccounts(_, _) => "There are fewer accounts than the minimum.",
            Self::CalculationError => "Unknown calculation error.",
            Self::AccountAccess => "Your account does not have access to send.",
            Self::TooFewCommitedValue(_, _) => "There are fewer actual values than the minimum.",
        }
    }
}

impl<T: Trait> Default for Oracle<T>
{
    fn default() -> Oracle<T>
    {
        Oracle {
            name: Vec::new(),
            table: TableId::<T>::default(),
            sources_threshold: u8::default(),
            sources: BTreeMap::default(),
            assets_name: AssetsVec::default(),
            value: AssetsVec::default(),
            period_handler: PeriodHandler::default(),
        }
    }
}

impl<T: Trait> Oracle<T>
{
    pub fn new(
        name: RawString,
        table: TableId<T>,
        period_handler: PeriodHandler<T::Moment>,
        sources_threshold: u8,
        assets: AssetsVec<RawString>,
    ) -> Oracle<T>
    {
        Oracle {
            name,
            table,
            sources_threshold,
            period_handler,
            sources: BTreeMap::new(),
            value: AssetsVec {
                0: assets.0.iter().map(|_| ExternalValue::<T>::new()).collect(),
            },
            assets_name: AssetsVec {
                0: assets.0.iter().cloned().collect(),
            },
        }
    }

    pub fn is_calculate_time(&self, external_asset_id: usize, now: Moment<T>) -> bool
    {
        external_asset_id < self.get_assets_count()
            && self
                .period_handler
                .is_calculate_time(self.value.0[external_asset_id].last_changed, now)
    }

    pub fn get_assets_count(&self) -> usize
    {
        self.assets_name.0.len()
    }

    pub fn add_asset(&mut self, name: RawString)
    {
        self.assets_name.0.push(name);
        self.value.0.push(ExternalValue::new());
    }

    pub fn update_accounts<I>(&mut self, accounts: I) -> Result<(), OracleError>
    where
        I: Iterator<Item = AccountId<T>>,
    {
        let mut default_external_value: AssetsVec<ExternalValue<T>> = self.value.clone();
        default_external_value
            .0
            .iter_mut()
            .for_each(|val| val.clean());

        self.sources = accounts
            .map(|account| {
                let external_value = self
                    .sources
                    .get(&account)
                    .unwrap_or(&default_external_value)
                    .clone();
                (account, external_value)
            })
            .collect();

        match (self.sources.len() as u8).cmp(&self.sources_threshold)
        {
            Ordering::Less => Err(OracleError::TooFewAccounts(
                self.sources_threshold as usize,
                self.sources.len() as usize,
            )),
            _ => Ok(()),
        }
    }

    pub fn commit_value(
        &mut self,
        account: &AccountId<T>,
        values: AssetsVec<T::ValueType>,
        now: Moment<T>,
    ) -> Result<(), OracleError>
    {
        if let Some(assets) = self.sources.get_mut(account)
        {
            assets
                .0
                .iter_mut()
                .zip(values.0.iter())
                .for_each(|(external, new_val)| external.update(*new_val, now));
            Ok(())
        }
        else
        {
            Err(OracleError::AccountAccess)
        }
    }

    pub fn calculate_median(
        &mut self,
        number: usize,
        now: Moment<T>,
    ) -> Result<T::ValueType, OracleError>
    {
        if number >= self.get_assets_count()
        {
            return Err(OracleError::WrongAssetId(number));
        }

        if self.sources.len() < self.sources_threshold as usize
        {
            return Err(OracleError::TooFewAccounts(
                self.sources_threshold as usize,
                self.sources.len() as usize,
            ));
        }

        let assets: Vec<&T::ValueType> = self
            .sources
            .iter()
            .map(|(_, assets)| assets.0.get(number))
            .filter(|external| external.and_then(|ext| ext.value).is_some())
            .map(|external| {
                external
                    .as_ref()
                    .map(|ext| ext.value.as_ref().unwrap())
                    .unwrap()
            })
            .collect();

        if assets.len() < self.sources_threshold as usize
        {
            return Err(OracleError::TooFewCommitedValue(
                self.sources_threshold as usize,
                assets.len() as usize,
            ));
        }

        match get_median(assets)
        {
            Some(Median::Value(value)) => Some(value.clone()),
            Some(Median::Pair(left, right)) =>
            {
                let sum = *left + *right;
                let divider: T::ValueType = One::one();

                Some(sum / (divider + One::one()))
            }
            _ => None,
        }
        .map_or(Err(OracleError::CalculationError), |med| {
            self.value.0[number].update(med, now);
            Ok(med)
        })
    }
}

#[cfg(test)]
mod tests
{
    use crate::mock::Test;

    type Oracle = super::Oracle<Test>;
    use super::OracleError;
    type Moment = crate::module_trait::Moment<Test>;
    use super::{AssetsVec, PeriodHandler};

    fn get_period_handler() -> PeriodHandler<Moment>
    {
        super::PeriodHandler::new(100, 10, 5).unwrap()
    }

    fn get_assets_vec<Item, It>(iter: It) -> super::AssetsVec<Item>
    where
        It: Iterator<Item = Item>,
    {
        AssetsVec { 0: iter.collect() }
    }

    fn get_oracle() -> Oracle
    {
        Oracle::new(
            "test".to_owned().as_bytes().to_vec(),
            0,
            get_period_handler(),
            9,
            get_assets_vec(
                vec!["f", "s", "t"]
                    .iter()
                    .map(|s| s.to_owned().as_bytes().to_vec()),
            ),
        )
    }

    fn update_values(oracle: &mut Oracle, accounts: Vec<u64>, now: Moment, values: Vec<Vec<u128>>)
    {
        for account in accounts.into_iter()
        {
            oracle
                .commit_value(
                    &account,
                    AssetsVec {
                        0: values
                            .iter()
                            .map(|assets| {
                                assets
                                    .get(account as usize)
                                    .expect("Wrong values matrix")
                                    .clone()
                            })
                            .collect(),
                    },
                    now,
                )
                .expect(&format!("Can't commit for {}.", account).to_string());
        }
    }

    fn update_oracle(oracle: &mut Oracle, now: Moment, values: Vec<Vec<u128>>)
    {
        let accounts: Vec<u64> = oracle.sources.keys().cloned().collect();
        update_values(oracle, accounts, now, values);
    }

    #[test]
    fn create_oracle()
    {
        let oracle = get_oracle();
        assert_eq!(oracle.value.0.len(), 3);
        assert!(oracle
            .value
            .0
            .iter()
            .all(|ex_val| ex_val.value.is_none() && ex_val.last_changed.is_none()));
    }

    #[test]
    fn calculate_error_few_accounts()
    {
        let mut oracle = get_oracle();
        assert_eq!(oracle.value.0.len(), 3);
        assert_eq!(
            oracle.update_accounts(1..=3),
            Err(OracleError::TooFewAccounts(9, 3))
        );
        update_oracle(
            &mut oracle,
            101,
            vec![vec![1; 10], vec![2; 10], vec![3; 10]],
        );

        assert_eq!(
            oracle.calculate_median(0, 102),
            Err(OracleError::TooFewAccounts(9, 3))
        );
    }

    #[test]
    fn calculate_error_few_commits()
    {
        let mut oracle = get_oracle();
        assert_eq!(oracle.value.0.len(), 3);
        assert_eq!(oracle.update_accounts(0..=10), Ok(()));

        update_values(
            &mut oracle,
            (0..=5).collect(),
            101,
            vec![vec![1; 10], vec![2; 10], vec![3; 10]],
        );

        assert_eq!(
            oracle.calculate_median(0, 102),
            Err(OracleError::TooFewCommitedValue(9, 6))
        );
    }

    #[test]
    fn simple_calculate_median()
    {
        let mut oracle = get_oracle();
        assert_eq!(oracle.update_accounts(0..=10), Ok(()));
        update_oracle(
            &mut oracle,
            101,
            vec![vec![1; 11], vec![2; 11], vec![3; 11]],
        );

        assert_eq!(oracle.calculate_median(0, 102), Ok(1));
        assert_eq!(oracle.calculate_median(1, 102), Ok(2));
        assert_eq!(oracle.calculate_median(2, 102), Ok(3));
    }

    #[test]
    fn calculate_median()
    {
        let mut oracle = get_oracle();
        assert_eq!(oracle.update_accounts(0..=11), Ok(()));
        update_oracle(
            &mut oracle,
            101,
            vec![
                (100..=112u128).collect::<Vec<u128>>(),
                vec![0; 12],
                vec![0; 12],
            ],
        );
        assert_eq!(oracle.calculate_median(0, 102), Ok(106));
    }
}
