use rstd::collections::{btree_map::BTreeMap, btree_set::BTreeSet};

use support::{decl_event, decl_module, decl_storage, dispatch::Result, Parameter};

use codec::{Decode, Encode};
use core::cmp::{Ord, Ordering, PartialOrd};
use rstd::prelude::*;
use sr_primitives::traits::{
    //CheckedAdd,
    Member,
    //One,
    SimpleArithmetic,
};
use system::ensure_signed;
pub trait Trait: assets::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type TargetType: Default + Parameter + SimpleArithmetic;
    type TableId: Parameter + Member + SimpleArithmetic + Default + Copy;
}

type Balance<T> = <T as assets::Trait>::Balance;
type AssetId<T> = <T as assets::Trait>::AssetId;
type AccountId<T> = <T as system::Trait>::AccountId;

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
        self.balance.cmp(&other.balance)
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
    vote_asset: AssetId<T>,
    scores: BTreeSet<Record<T>>,
    reserved: BTreeMap<AccountId<T>, Balance<T>>,
}

impl<T: Trait> Default for Table<T>
{
    fn default() -> Self
    {
        Table {
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

        pub fn vote(origin, table_id: T::TableId, count: Balance<T>, target: T::TargetType) -> Result
        {
            let voter = ensure_signed(origin)?;

            Self::rereserve(&voter, &table_id, count)?;
            unimplemented!()
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
    fn rereserve(voter: &AccountId<T>, table_id: &T::TableId, count: Balance<T>) -> Result
    {
            let mut result: Result = Ok(());
            Scores::<T>::mutate(table_id, |table| {
                match table.reserved.get(voter)
                {
                    Some(reserved) =>
                    {
                        match reserved.cmp(&count)
                        {
                            Ordering::Greater =>
                            {
                                assets::Module::<T>::unreserve(&table.vote_asset, voter, *reserved - count);
                            }
                            Ordering::Less =>
                            {
                                result = assets::Module::<T>::reserve(&table.vote_asset, voter, count - *reserved);
                            }
                            _ => {}
                        }
                    }
                    None =>
                    {
                        result = assets::Module::<T>::reserve(&table.vote_asset, voter, count);
                    }
                }

                //table.reserved.replace(voter, count); ToDo Don't work
                table.reserved.remove(voter);
                table.reserved.insert(voter.clone(), count);
            });
            result
    }
}
