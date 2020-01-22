pub use crate::tablescore;
use codec::{Decode, Encode};
use rstd::prelude::*;
use sr_primitives::traits::{Member, SimpleArithmetic};
use support::Parameter;

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
