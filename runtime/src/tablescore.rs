use rstd::collections::btree_map::BTreeMap;

use support::{decl_event, decl_module, decl_storage, Parameter};

use codec::{Decode, Encode};
use rstd::vec::Vec;
use sr_primitives::traits::{
    //CheckedAdd,
    Member,
    //One,
    SimpleArithmetic,
};
//use system::ensure_signed;

pub trait Trait: assets::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type TargetType: Parameter + SimpleArithmetic;
    type TableId: Parameter + Member + SimpleArithmetic + Default + Copy;
}

type Balance<T: Trait> = <T as assets::Trait>::Balance;
type AssetId<T: Trait> = <T as assets::Trait>::AssetId;
type AccountId<T: Trait> = <T as system::Trait>::AccountId;

#[derive(Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
struct Record<T: Trait>
{
    target: T::TargetType,
    balances: BTreeMap<AccountId<T>, Balance<T>>,
}

impl<T: Trait> Record<T>
{
    fn get_balance(&self) -> Balance<T>
    {
        self.balances
            .iter()
            .fold(Balance::<T>::default(), |all, (_, balance)| *balance + all)
    }
}

#[derive(Decode, Encode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Table<T: Trait>
{
    vote_asset: AssetId<T>,
    scores: Vec<Record<T>>,
}

impl<T: Trait> Default for Table<T>
{
    fn default() -> Self
    {
        Table {
            vote_asset: AssetId::<T>::default(),
            scores: Vec::new(),
        }
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Tablescore {
        Scores get(scores): map T::TableId => Table<T>;
        TableScoreIdSequence get(next_asset_id): T::TableId;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        //pub fn vote(origin, table_id: T::TableId, count: Balance<T>)
        //{
        //    let who = ensure_signed(origin)?;

        //    <Balance<T>>::get(&origin);
        //}
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

impl<T: Trait> Module<T> {}
