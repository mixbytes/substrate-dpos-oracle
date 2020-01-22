extern crate alloc;
use alloc::collections::{BinaryHeap, LinkedList};

use rstd::cmp::{Ord, Ordering};

use codec::{Decode, Encode};
use rstd::collections::btree_map::BTreeMap;
use rstd::prelude::*;
use sr_primitives::traits::One;
use support::dispatch::Result as SimpleResult;

pub use crate::module_trait::*;
pub use crate::external_value::*;

#[derive(Encode, Decode, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct PeriodHandler<T: Trait>
{
    start: Moment<T>,
    calculate_period: TimeInterval<T>,
    aggregate_period: TimeInterval<T>,
    last_sources_update: Moment<T>,
}

impl<T: Trait> PeriodHandler<T>
{
    pub fn new(
        calculate_period: TimeInterval<T>,
        aggregate_period: TimeInterval<T>,
    ) -> PeriodHandler<T>
    {
        PeriodHandler {
            calculate_period,
            aggregate_period,
            start: timestamp::Module::<T>::get(),
            last_sources_update: Moment::<T>::default(),
        }
    }
}

impl<T: Trait> Default for PeriodHandler<T>
{
    fn default() -> PeriodHandler<T>
    {
        PeriodHandler {
            start: Moment::<T>::default(),
            calculate_period: TimeInterval::<T>::default(),
            aggregate_period: TimeInterval::<T>::default(),
            last_sources_update: Moment::<T>::default(),
        }
    }
}

impl<T: Trait> PeriodHandler<T>
{
    pub fn get_period(&self, now: Moment<T>) -> TimeInterval<T>
    {
        (now - self.start) % self.calculate_period
    }

    pub fn is_aggregate_time(&self, now: Moment<T>) -> bool
    {
        ((self.get_period(now) + One::one()) * self.calculate_period - now) < self.aggregate_period
    }

    pub fn is_calculate_time(&self, last_update_time: Option<Moment<T>>, now: Moment<T>) -> bool
    {
        match last_update_time
        {
            Some(last_changed) => self.get_period(now) > self.get_period(last_changed),
            None => true,
        }
    }

    pub fn is_source_update_time(&self, now: Moment<T>) -> bool
    {
        self.is_aggregate_time(now)
            && self.get_period(self.last_sources_update) < self.get_period(now)
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Oracle<T: Trait>
{
    pub name: RawString,
    pub table: TableId<T>,

    source_calculate_count: u8,
    pub period_handler: PeriodHandler<T>,

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
        period_handler: PeriodHandler<T>,
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
        let assets: LinkedList<&T::ValueType> = self
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

        let (mut min_heap, mut max_heap) = assets.into_iter().fold(
            (BinaryHeap::new(), BinaryHeap::new()),
            |(mut max_heap, mut min_heap), value| {
                min_heap.push(Reverse(value));

                if let Some(val) = min_heap.pop()
                {
                    max_heap.push(val.0);
                }

                if min_heap.len() < max_heap.len()
                {
                    min_heap.push(Reverse(max_heap.pop().unwrap()));
                }
                (max_heap, min_heap)
            },
        );

        let new_val = match min_heap.len().cmp(&max_heap.len())
        {
            Ordering::Greater => min_heap.pop().copied(),

            Ordering::Less | Ordering::Equal => match (min_heap.pop(), max_heap.pop())
            {
                (Some(min), Some(Reverse(max))) =>
                {
                    let sum = *min + *max;
                    let divider: T::ValueType = One::one();

                    Some(sum / (divider + One::one()))
                }
                _ => None,
            },
        };

        match new_val
        {
            Some(value) =>
            {
                self.value.0[number].update(value);
                Ok(())
            }
            None => Err("Error in calculating process"),
        }
    }
}
