use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, storage::StorageMap, Parameter,
    StorageValue,
};

use codec::{Decode, Encode};
use rstd::result;
use rstd::vec::Vec;
use sr_primitives::traits::{CheckedAdd, Member, One, SimpleArithmetic};
use system::ensure_signed;

pub trait Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    
    type TargetType;
}

decl_storage! {
    trait Store for Module<T: Trait> as tablescore {
    }

    // Add custom field to module configuration
    add_extra_genesis {
    }
}

// External API. Can be called from external client.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

    }
}

decl_event!(
    pub enum Event<T>
    where AccountId = <T as system::Trait>::AccountId,
    {
    }
);

// Internal API. Can be called from other modules
impl<T: Trait> Module<T>
{
}
