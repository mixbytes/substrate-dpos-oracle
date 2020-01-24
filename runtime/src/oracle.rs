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
        self.period_handler
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

    pub fn update_accounts(&mut self, accounts: Vec<AccountId<T>>)
    {
        let mut default_external_value: AssetsVec<ExternalValue<T>> = self.value.clone();
        default_external_value
            .0
            .iter_mut()
            .for_each(|val| val.clean());

        self.sources = accounts
            .into_iter()
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

    pub fn calculate_median(&mut self, number: usize) -> SimpleResult
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

        match get_median(assets)
        {
            Some(Median::Value(value)) =>
            {
                self.value.0[number].update(value.clone());
                Ok(())
            }
            Some(Median::Pair(left, right)) =>
            {
                let sum = *left + *right;
                let divider: T::ValueType = One::one();

                self.value.0[number].update(sum / (divider + One::one()));
                Ok(())
            }
            _ => Err("Error in calculating process"),
        }
    }
}
