extern crate alloc;
use alloc::collections::BinaryHeap;

use rstd::cmp::{Ord, Ordering};

use codec::{Decode, Encode};
use rstd::collections::btree_map::BTreeMap;
use rstd::prelude::*;
use sr_primitives::traits::{Member, One, SimpleArithmetic};
use support::Parameter;

pub use crate::tablescore;

pub type Balance<T> = <T as assets::Trait>::Balance;
pub type AssetId<T> = <T as assets::Trait>::AssetId;
pub type AccountId<T> = <T as system::Trait>::AccountId;

pub type RawString = Vec<u8>;

pub trait Trait:
    assets::Trait + timestamp::Trait + tablescore::Trait<TargetType = AccountId<Self>>
{
    type Event: Into<<Self as system::Trait>::Event>;
    type OracleId: Parameter + Member + SimpleArithmetic + Default + Copy;

    type ValueType: Member + Parameter + SimpleArithmetic + Default + Copy;
}

pub type TableId<T> = <T as tablescore::Trait>::TableId;
pub type Moment<T> = <T as timestamp::Trait>::Moment;
pub type TimeInterval<T> = <T as timestamp::Trait>::Moment;

#[derive(Encode, Decode, Debug, Clone, Eq, PartialEq)]
pub struct AssetsVec<T>(pub Vec<T>);

impl<T> Default for AssetsVec<T>
{
    fn default() -> AssetsVec<T>
    {
        AssetsVec { 0: Vec::new() }
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ExternalValue<T: Trait>
{
    value: Option<T::ValueType>,
    last_changed: Option<Moment<T>>,
}

impl<T: Trait> PartialOrd for ExternalValue<T>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        Some(self.cmp(&other))
    }
}

impl<T: Trait> Ord for ExternalValue<T>
{
    fn cmp(&self, other: &Self) -> Ordering
    {
        match self.value.cmp(&other.value)
        {
            Ordering::Equal => self.last_changed.cmp(&other.last_changed),
            ord => ord,
        }
    }
}

impl<T: Trait> ExternalValue<T>
{
    pub fn new() -> ExternalValue<T>
    {
        ExternalValue {
            value: None,
            last_changed: None,
        }
    }

    pub fn clean(&mut self)
    {
        self.value = None;
        self.last_changed = None;
    }

    pub fn update_time(&mut self)
    {
        self.last_changed = Some(timestamp::Module::<T>::get());
    }

    pub fn update(&mut self, value: T::ValueType)
    {
        self.value = Some(value);
        self.update_time();
    }

    fn average(&self, other: &Self) -> Self
    {
        ExternalValue {
            value: match (self.value, other.value)
            {
                (Some(lval), Some(rval)) =>
                {
                    let two: T::ValueType = One::one();
                    Some((lval + rval) / (two + One::one()))
                }
                _ => None,
            },
            last_changed: Some(timestamp::Module::<T>::get()),
        }
    }
}

impl<T: Trait> Default for ExternalValue<T>
{
    fn default() -> Self
    {
        ExternalValue {
            value: None,
            last_changed: None,
        }
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Oracle<T: Trait>
{
    pub name: RawString,
    pub table: TableId<T>,

    aggregate: TimeInterval<T>,
    peace: TimeInterval<T>,

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
            aggregate: TimeInterval::<T>::default(),
            peace: TimeInterval::<T>::default(),
            sources: BTreeMap::default(),
            assets_name: AssetsVec::default(),
            value: AssetsVec::default(),
        }
    }
}

impl<T: Trait> Oracle<T>
{
    pub fn new(
        name: RawString,
        table: TableId<T>,
        aggregate: TimeInterval<T>,
        peace: TimeInterval<T>,
        assets: AssetsVec<RawString>,
    ) -> Oracle<T>
    {
        Oracle {
            name,
            table,
            aggregate,
            peace,
            sources: BTreeMap::new(),
            value: AssetsVec {
                0: assets.0.iter().map(|_| ExternalValue::<T>::new()).collect(),
            },
            assets_name: AssetsVec {
                0: assets.0.iter().map(|name| name.clone()).collect(),
            },
        }
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
        let mut external_value: AssetsVec<ExternalValue<T>> = self.value.clone();
        external_value.0.iter_mut().for_each(|val| val.clean());

        self.sources = accounts
            .into_iter()
            .map(|account| (account, external_value.clone()))
            .collect()
    }

    pub fn calculate_median(&mut self, number: usize) -> Option<ExternalValue<T>>
    {
        let (mut min_heap, mut max_heap) = self
            .sources
            .iter()
            .map(|(_, assets)| assets.0.get(number))
            .filter(|external| external.is_some())
            .fold(
                (BinaryHeap::new(), BinaryHeap::new()),
                |(mut max_heap, mut min_heap), value| {
                    min_heap.push(Reverse(value.unwrap()));

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

        match &mut min_heap.len().cmp(&max_heap.len())
        {
            Ordering::Greater => min_heap.pop().map(|val| {
                let mut val = val.clone();
                val.update_time();
                val
            }),
            _ => match (min_heap.pop(), max_heap.pop())
            {
                (Some(min), Some(max)) => Some(min.average(max.0)),
                _ => None,
            },
        }
    }
}
