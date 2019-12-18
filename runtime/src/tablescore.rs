use rstd::collections::btree_map::BTreeMap;

use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, storage::StorageMap, Parameter,
    StorageValue,
};

use codec::{Decode, Encode};
use rstd::result;
use rstd::vec::Vec;
use sr_primitives::traits::{CheckedAdd, Member, One, SimpleArithmetic};
use system::ensure_signed;

pub trait Trait: assets::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type TargetType: Parameter + SimpleArithmetic;
    type TableId: Parameter + Member + SimpleArithmetic + Default + Copy;
}

type Balance<T: Trait> = <T as assets::Trait>::Balance;
type AssetId<T: Trait> = <T as assets::Trait>::AssetId;
type AccountId<T: Trait> = <T as system::Trait>::AccountId;

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
            .map(|(_acc, balance)| balance)
            .sum()
    }
}

#[derive(Decode, Encode)]
struct Table<T: Trait>
{
    vote_asset: AssetId<T>,
    scores: Vec<Record<T>>,
}

decl_storage! {
    trait Store for Module<T: Trait> as tablescore {
        Scores get(scores): map T::TableId => Table<T>;
        TableScoreIdSequence get(next_asset_id): T::TableId;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        pub fn vote(origin, table_id: T::TableId, count: Balance<T>)
        {
            let who = ensure_signed(origin)?;

            <Balance<T>>::get(&origin);
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId, {}
);

impl<T: Trait> Module<T> {}
