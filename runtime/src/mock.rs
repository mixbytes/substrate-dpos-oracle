use super::*;
use crate::tablescore::*;
use aura_primitives::sr25519::AuthorityId as AuraId;
use sr_primitives::traits::{BlakeTwo256, ConvertInto};
use sr_primitives::weights::Weight;
use sr_primitives::{generic, traits::IdentityLookup};

pub use assets::Call as AssetsCall;
pub use balances::Call as BalancesCall;
#[cfg(any(feature = "std", test))]
pub use sr_primitives::BuildStorage;
pub use sr_primitives::{Perbill, Permill};
pub use support::{
    construct_runtime, impl_outer_origin, parameter_types, traits::Randomness, StorageValue,
};
use system::IsDeadAccount;
pub use timestamp::Call as TimestampCall;

pub type BlockNumber = u32;
pub type AccountId = u64;
pub type AccountIndex = u32;
pub type Balance = u128;
pub type Index = u32;
pub type Hash = primitives::H256;

pub const MILLISECS_PER_BLOCK: u64 = 6000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

impl_outer_origin! {
    pub enum Origin for Test  where system = system {}
}

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
    pub const MaximumBlockWeight: Weight = 1_000_000;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

impl system::Trait for Test
{
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Index = Index;
    type BlockNumber = BlockNumber;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    type Origin = Origin;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Call = tablescore::Call<Test>;
    type Event = ();
    type Version = ();
}

impl timestamp::Trait for Test
{
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = MinimumPeriod;
}

impl aura::Trait for Test
{
    type AuthorityId = AuraId;
}

impl grandpa::Trait for Test
{
    type Event = ();
}

pub struct TestIsDeadAccount {}
impl IsDeadAccount<u64> for TestIsDeadAccount
{
    fn is_dead_account(_: &u64) -> bool
    {
        false
    }
}

impl indices::Trait for Test
{
    type AccountIndex = AccountIndex;
    type ResolveHint = indices::SimpleResolveHint<AccountId, AccountIndex>;
    type IsDeadAccount = TestIsDeadAccount;
    type Event = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const TransferFee: u128 = 0;
    pub const CreationFee: u128 = 0;
}

impl balances::Trait for Test
{
    type Balance = Balance;
    type OnFreeBalanceZero = ();
    type OnNewAccount = ();
    type Event = ();
    type DustRemoval = ();
    type TransferPayment = ();
    type ExistentialDeposit = ExistentialDeposit;
    type TransferFee = TransferFee;
    type CreationFee = CreationFee;
}

parameter_types! {
    pub const TransactionBaseFee: Balance = 0;
    pub const TransactionByteFee: Balance = 1;
}

impl transaction_payment::Trait for Test
{
    type Currency = balances::Module<Test>;
    type OnTransactionPayment = ();
    type TransactionBaseFee = TransactionBaseFee;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = ConvertInto;
    type FeeMultiplierUpdate = ();
}

impl sudo::Trait for Test
{
    type Event = ();
    type Proposal = tablescore::Call<Test>;
}

impl assets::Trait for Test
{
    type Event = ();
    type Balance = u128;
    type AssetId = u64;
}

impl tablescore::Trait for Test
{
    type Event = ();
    type TargetType = u64;
    type TableId = u64;
}

pub type TablescoreModule = Module<Test>;

pub fn new_test_ext() -> runtime_io::TestExternalities
{
    let t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    t.into()
}
