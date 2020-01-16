use rstd::collections::{btree_map::BTreeMap, btree_set::BTreeSet};

use support::{decl_event, decl_module, decl_storage, dispatch::Result, Parameter};

use codec::{Decode, Encode};
use core::cmp::{Ord, Ordering, PartialOrd};
use rstd::prelude::*;
use rstd::result;
use sr_primitives::traits::{CheckedAdd, Member, One, SimpleArithmetic, Zero};

use system::ensure_signed;

type Balance<T> = <T as assets::Trait>::Balance;
type AssetId<T> = <T as assets::Trait>::AssetId;
type AccountId<T> = <T as system::Trait>::AccountId;

const DEFAULT_HEAD_COUNT: u8 = 5;

pub trait Trait: assets::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type TargetType: Default + Parameter + Ord;
    type TableId: Parameter + Member + SimpleArithmetic + Default + Copy;
}

#[derive(Encode, Decode, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Record<T: Trait>
{
    target: T::TargetType,
    balance: Balance<T>,
}

impl<T: Trait> Ord for Record<T>
{
    fn cmp(&self, other: &Self) -> Ordering
    {
        match self.balance.cmp(&other.balance)
        {
            Ordering::Equal => self.target.cmp(&other.target),
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
        }
    }
}

impl<T: Trait> PartialOrd for Record<T>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        Some(self.cmp(&other))
    }
}

impl<T: Trait> Default for Record<T>
{
    fn default() -> Self
    {
        Record {
            target: T::TargetType::default(),
            balance: Balance::<T>::default(),
        }
    }
}

#[derive(Decode, Encode, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Table<T: Trait>
{
    pub name: Option<Vec<u8>>,
    pub head_count: u8,
    pub vote_asset: AssetId<T>,
    pub scores: BTreeSet<Record<T>>,
    pub reserved: BTreeMap<AccountId<T>, Record<T>>,
}

impl<T: Trait> Default for Table<T>
{
    fn default() -> Self
    {
        Table {
            name: None,
            head_count: DEFAULT_HEAD_COUNT,
            vote_asset: AssetId::<T>::default(),
            scores: BTreeSet::new(),
            reserved: BTreeMap::new(),
        }
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Tablescore {
        pub Scores get(scores): map T::TableId => Table<T>;
        TableScoreIdSequence get(next_asset_id): T::TableId;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        pub fn create_table(
            origin,
            vote_asset: AssetId<T>,
            head_count: u8,
            name: Option<Vec<u8>>) -> Result
        {
            let _ = ensure_signed(origin)?;
            Self::create(vote_asset, head_count, name)?;
            Ok(())
        }

        pub fn vote(
            origin,
            table_id: T::TableId,
            balance: Balance<T>,
            target: T::TargetType) -> Result {
            let voter = ensure_signed(origin)?;
            let table = Scores::<T>::get(&table_id);

            let new_record = Record { target, balance };
            let old_record = table.reserved.get(&voter);

            Self::rereserve(&voter, &table.vote_asset, old_record, &new_record)?;

            Scores::<T>::mutate(&table_id, |table| {
                table.reserved.remove(&voter);
                if let Some(record) = old_record { table.scores.remove(record); }

                if new_record.balance != Zero::zero() {
                    table.reserved.insert(voter.clone(), new_record.clone());
                    table.scores.insert(new_record);
                }
            });

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        Voted(AccountId),
    }
);

impl<T: Trait> Module<T>
{
    pub fn create(
        vote_asset: AssetId<T>,
        head_count: u8,
        name: Option<Vec<u8>>,
    ) -> result::Result<T::TableId, &'static str>
    {
        let id = Self::pop_new_table_id()?;
        Scores::<T>::insert(
            id.clone(),
            Table {
                name: name,
                head_count: head_count,
                vote_asset: vote_asset,
                scores: BTreeSet::new(),
                reserved: BTreeMap::new(),
            },
        );

        Ok(id)
    }

    fn pop_new_table_id() -> result::Result<T::TableId, &'static str>
    {
        let mut result = Err("Unknown error");

        TableScoreIdSequence::<T>::mutate(|id| match id.checked_add(&One::one())
        {
            Some(res) =>
            {
                result = Ok(id.clone());
                *id = res;
            }
            None =>
            {
                result = Err("T::TableId overflow. Can't get next id.");
            }
        });

        result
    }

    fn rereserve(
        voter: &AccountId<T>,
        asset_id: &AssetId<T>,
        old_record: Option<&Record<T>>,
        new_record: &Record<T>,
    ) -> Result
    {
        match old_record
        {
            Some(record) => match record.balance.cmp(&new_record.balance)
            {
                Ordering::Greater =>
                {
                    assets::Module::<T>::unreserve(
                        asset_id,
                        voter,
                        record.balance - new_record.balance,
                    );
                }
                Ordering::Less =>
                {
                    assets::Module::<T>::reserve(
                        asset_id,
                        voter,
                        new_record.balance - record.balance,
                    )?;
                }
                _ =>
                {}
            },
            None =>
            {
                assets::Module::<T>::reserve(asset_id, voter, new_record.balance)?;
            }
        }
        Ok(())
    }

    pub fn get_head(table_id: &T::TableId) -> Vec<T::TargetType>
    {
        let table = Scores::<T>::get(table_id);
        table
            .scores
            .iter()
            .map(|record| record.target.clone())
            .take(table.head_count as usize)
            .collect()
    }
}
