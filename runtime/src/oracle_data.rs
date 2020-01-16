use codec::{Decode, Encode};
use rstd::collections::btree_map::BTreeMap;
use rstd::prelude::*;
use sr_primitives::traits::{CheckedAdd, Member, One, SimpleArithmetic, Zero};
use support::{decl_event, decl_module, decl_storage, dispatch::Result as SimpleResult, Parameter};

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

#[derive(Encode, Decode, Debug, Clone, Eq, PartialEq)]
pub enum AggregateType
{
    Mediana,
    Average,
    TimeWeightedAverage,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ExternalValue<T: Trait>
{
    value: Option<T::ValueType>,
    last_changed: Option<Moment<T>>,

    aggregate_type: AggregateType,
}

impl<T: Trait> ExternalValue<T>
{
    pub fn new(aggregate_type: AggregateType) -> ExternalValue<T>
    {
        ExternalValue {
            value: None,
            last_changed: None,
            aggregate_type: aggregate_type,
        }
    }

    pub fn clean(&mut self)
    {
        self.value = None;
        self.last_changed = None;
    }

    pub fn update(&mut self, value: T::ValueType)
    {
        self.value = Some(value);
        self.last_changed = Some(timestamp::Module::<T>::get());
    }
}

impl<T: Trait> Default for ExternalValue<T>
{
    fn default() -> Self
    {
        ExternalValue {
            value: None,
            last_changed: None,
            aggregate_type: AggregateType::Average,
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

impl<T: Trait> Oracle<T>
{
    pub fn update_accounts(&mut self, accounts: Vec<AccountId<T>>)
    {
        let mut external_value: AssetsVec<ExternalValue<T>> = self.value.clone();
        external_value.0.iter_mut().for_each(|val| val.clean());

        self.sources = accounts
            .into_iter()
            .map(|account| (account, external_value.clone()))
            .collect()
    }
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
        assets: AssetsVec<(RawString, AggregateType)>,
    ) -> Oracle<T>
    {
        Oracle {
            name,
            table,
            aggregate,
            peace,
            sources: BTreeMap::new(),
            value: AssetsVec {
                0: assets
                    .0
                    .iter()
                    .map(|(_, agg_type)| ExternalValue::<T>::new(agg_type.clone()))
                    .collect(),
            },
            assets_name: AssetsVec {
                0: assets.0.iter().map(|(name, _)| name.clone()).collect(),
            },
        }
    }

    pub fn get_assets_count(&self) -> usize
    {
        self.assets_name.0.len()
    }

    pub fn add_asset(&mut self, name: RawString, aggregate_type: AggregateType)
    {
        self.assets_name.0.push(name);
        self.value.0.push(ExternalValue::new(aggregate_type));
    }

    pub fn calculate_value(&mut self, number: usize)
    {
        todo!()
    }
}
