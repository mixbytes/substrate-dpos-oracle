use codec::{Decode, Encode};
use rstd::collections::btree_map::BTreeMap;
use rstd::prelude::*;
use sr_primitives::traits::One;
use support::dispatch::Result as SimpleResult;

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

    source_calculate_count: u8,
    pub period_handler: PeriodHandler<T::Moment>,

    pub assets_name: AssetsVec<RawString>,

    pub sources: BTreeMap<AccountId<T>, AssetsVec<ExternalValue<T>>>,
    pub value: AssetsVec<ExternalValue<T>>,
}

impl<T: Trait> Default for Oracle<T>
{
    fn default() -> Oracle<T>
    {
        Oracle {
            name: Vec::new(),
            table: TableId::<T>::default(),
            source_calculate_count: u8::default(),
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
        source_calculate_count: u8,
        assets: AssetsVec<RawString>,
    ) -> Oracle<T>
    {
        Oracle {
            name,
            table,
            source_calculate_count,
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

    pub fn update_accounts<I>(&mut self, accounts: I)
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
            .collect()
    }

    pub fn commit_value(
        &mut self,
        account: &AccountId<T>,
        values: AssetsVec<T::ValueType>,
        now: Moment<T>,
    ) -> SimpleResult
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
            Err("Can't find account in accepted")
        }
    }

    pub fn calculate_median(
        &mut self,
        number: usize,
        now: Moment<T>,
    ) -> Result<T::ValueType, &'static str>
    {
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

        if assets.len() < self.source_calculate_count as usize
        {
            return Err("Not enough sources");
        }

        let median = match get_median(assets)
        {
            Some(Median::Value(value)) =>
            {
                self.value.0[number].update(value.clone(), now);
                self.value.0[number].value
            }
            Some(Median::Pair(left, right)) =>
            {
                let sum = *left + *right;
                let divider: T::ValueType = One::one();

                self.value.0[number].update(sum / (divider + One::one()), now);
                self.value.0[number].value
            }
            _ => None,
        };

        median.map_or(Err("Can't calculate median"), |med| Ok(med))
    }
}

#[cfg(test)]
mod tests
{
    use crate::mock::Test;

    type Oracle = super::Oracle<Test>;
    use super::{AssetsVec, PeriodHandler};

    fn get_period_handler() -> PeriodHandler<crate::module_trait::Moment<Test>>
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
            10,
            get_assets_vec(
                vec!["f", "s", "t"]
                    .iter()
                    .map(|s| s.to_owned().as_bytes().to_vec()),
            ),
        )
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
    fn calculate_error()
    {
        let mut oracle = get_oracle();
        assert_eq!(oracle.value.0.len(), 3);
        oracle.update_accounts(1..=3);
        for account in 1..=3u64
        {
            oracle
                .commit_value(&account, AssetsVec { 0: vec![1, 2, 3] }, 100)
                .expect(&format!("Can't commit for {}.", account).to_string());
        }

        assert!(oracle.calculate_median(0, 101).is_err());
    }

    #[test]
    fn simple_calculate_median()
    {
        let mut oracle = get_oracle();
        oracle.update_accounts(1..=10);
        for account in 1..=10u64
        {
            oracle
                .commit_value(&account, AssetsVec { 0: vec![1, 2, 3] }, 100)
                .expect(&format!("Can't commit for {}.", account).to_string());
        }

        assert_eq!(oracle.calculate_median(0, 101), Ok(1));
        assert_eq!(oracle.calculate_median(1, 101), Ok(2));
        assert_eq!(oracle.calculate_median(2, 101), Ok(3));
    }

    #[test]
    fn calculate_median()
    {
        let mut oracle = get_oracle();
        oracle.update_accounts(1..=11);
        for (account, value) in (1..=11u64).zip(100..=112)
        {
            oracle
                .commit_value(
                    &account,
                    AssetsVec {
                        0: vec![value, 0, 0],
                    },
                    100,
                )
                .expect(&format!("Can't commit for {}.", account).to_string());
        }

        assert_eq!(oracle.calculate_median(0, 101), Ok(105));
    }
}
