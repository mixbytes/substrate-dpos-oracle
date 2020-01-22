pub use crate::module_trait::*;
use codec::{Decode, Encode};
use rstd::cmp::{Ord, Ordering};

#[derive(Encode, Decode, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ExternalValue<T: Trait>
{
    pub value: Option<T::ValueType>,
    pub last_changed: Option<Moment<T>>,
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

    pub fn with_value(value: T::ValueType) -> ExternalValue<T>
    {
        ExternalValue {
            value: Some(value),
            last_changed: Some(timestamp::Module::<T>::get()),
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

    pub fn update(&mut self, new_value: T::ValueType)
    {
        self.value = Some(new_value);
        self.update_time();
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
