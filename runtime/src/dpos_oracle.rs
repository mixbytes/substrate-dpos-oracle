use rstd::collections::{btree_map::BTreeMap, btree_set::BTreeSet};

use support::{decl_event, decl_module, decl_storage, dispatch::Result as SimpleResult, Parameter};

use codec::{Decode, Encode};
use core::cmp::{Ord, Ordering, PartialOrd};
use rstd::prelude::*;
use rstd::result::Result;
use sr_primitives::traits::{CheckedAdd, Member, One, SimpleArithmetic, Zero};
use system::ensure_signed;

use crate::tablescore;

type Balance<T> = <T as assets::Trait>::Balance;
type AssetId<T> = <T as assets::Trait>::AssetId;
type AccountId<T> = <T as system::Trait>::AccountId;

type RawString = Vec<u8>;

pub trait Trait: assets::Trait + timestamp::Trait + tablescore::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type OracleId: Parameter + Member + SimpleArithmetic + Default + Copy;
    type ValueType: Member + Parameter + SimpleArithmetic + Default + Copy;
}

type TableId<T> = <T as tablescore::Trait>::TableId;
type Moment<T> = <T as timestamp::Trait>::Moment;
type TimeInterval<T> = <T as timestamp::Trait>::Moment;

#[derive(Encode, Decode, Debug, Clone, Eq, PartialEq)]
pub struct AssetsVec<T>(Vec<T>);

impl<T> Default for AssetsVec<T> {
    fn default() -> AssetsVec<T> {
        AssetsVec { 0: Vec::new() }
    }
}

#[derive(Encode, Decode, Debug, Clone, Eq, PartialEq)]
pub enum AggregateType {
    Mediana,
    Average,
    TimeWeightedAverage,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
struct ExternalValue<T: Trait> {
    value: Option<T::ValueType>,
    last_changed: Option<Moment<T>>,

    aggregate_type: AggregateType,
}

impl<T: Trait> Default for ExternalValue<T> {
    fn default() -> Self {
        ExternalValue {
            value: None,
            last_changed: None,
            aggregate_type: AggregateType::Average,
        }
    }
}

impl<T: Trait> ExternalValue<T> {
    fn new(aggregate_type: AggregateType) -> ExternalValue<T> {
        ExternalValue {
            value: None,
            last_changed: None,
            aggregate_type,
        }
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Oracle<T: Trait> {
    name: RawString,
    table: TableId<T>,

    aggregate: TimeInterval<T>,
    peace: TimeInterval<T>,

    sources: BTreeMap<AccountId<T>, AssetsVec<ExternalValue<T>>>,
    assets_name: AssetsVec<RawString>,
    value: AssetsVec<ExternalValue<T>>,
}

impl<T: Trait> Default for Oracle<T> {
    fn default() -> Oracle<T> {
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

impl<T: Trait> Oracle<T> {
    fn new(
        name: RawString,
        table: TableId<T>,
        aggregate: TimeInterval<T>,
        peace: TimeInterval<T>,
        assets: AssetsVec<(RawString, AggregateType)>,
    ) -> Oracle<T> {
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

    fn get_assets_count(&self) -> usize {
        self.assets_name.0.len()
    }

    fn add_asset(&mut self, name: RawString, aggregate_type: AggregateType) {
        self.assets_name.0.push(name);
        self.value.0.push(ExternalValue::new(aggregate_type));
    }

    fn calculate(&mut self) {
        todo!()
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Tablescore {
        pub Oracles get(oracles): map T::OracleId => Oracle<T>;

        IdSequnce get(last_oracle_id): T::OracleId;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        pub fn create_oracle(
            origin,
            name: RawString,
            asset_id: AssetId<T>,
            source_count: u8,
            aggregate: TimeInterval<T>,
            peace: TimeInterval<T>,
            assets: AssetsVec<(RawString, AggregateType)>) -> SimpleResult
        {
            let _ = ensure_signed(origin)?;
            let table = tablescore::Module::<T>::create(asset_id, source_count, Some(name.clone()))?;

            Oracles::<T>::insert(Self::pop_new_oracle_id()?,
                Oracle::new(name, table, aggregate, peace, assets),
            );

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        OracleCreated(AccountId),
    }
);

impl<T: Trait> Module<T> {
    fn pop_new_oracle_id() -> Result<T::OracleId, &'static str> {
        let mut result = Err("Unknown error");

        IdSequnce::<T>::mutate(|id| match id.checked_add(&One::one()) {
            Some(res) => {
                result = Ok(id.clone());
                *id = res;
            }
            None => {
                result = Err("T::TableId overflow. Can't get next id.");
            }
        });

        result
    }
}
