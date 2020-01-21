use support::{decl_event, decl_module, decl_storage, dispatch::Result as SimpleResult};

use rstd::prelude::*;
use rstd::result::Result;
use sr_primitives::traits::{CheckedAdd, One};
use system::ensure_signed;

pub use crate::oracle_data::*;
use crate::tablescore;

decl_storage! {
    trait Store for Module<T: Trait> as Tablescore
    {
        pub Oracles get(oracles): map T::OracleId => Oracle<T>;

        OracleIdSequnce get(next_oracle_id): T::OracleId;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin
    {
        fn deposit_event() = default;

        pub fn create_oracle(
            origin,
            name: RawString,
            asset_id: AssetId<T>,
            source_calculate_count: u8,
            aggregate: TimeInterval<T>,
            calculate_period: TimeInterval<T>,
            assets: AssetsVec<RawString>) -> SimpleResult
        {
            let _ = ensure_signed(origin)?;
            let table = tablescore::Module::<T>::create(asset_id, source_calculate_count, Some(name.clone()))?;

            Oracles::<T>::insert(Self::pop_new_oracle_id()?,
                Oracle::new(name, table, aggregate, calculate_period, source_calculate_count, assets),
            );

            Ok(())
        }

        pub fn commit(
            origin,
            oracle_id: T::OracleId,
            values: AssetsVec<T::ValueType>,
        ) -> SimpleResult
        {
            let who = ensure_signed(origin)?;

            let oracle = Oracles::<T>::get(oracle_id);

            if values.0.len() != oracle.value.0.len() {
                Err("The number of assets does not match")
            } else if !oracle.sources.contains_key(&who) {
                Err("Your account is not a source for the oracle.")
            } else {
                Oracles::<T>::mutate(oracle_id, |oracle|
                    {
                        oracle.sources.get_mut(&who).map(|assets|
                        {
                            assets.0.iter_mut().zip(values.0.iter()).for_each(|(external, new_val)| external.update(*new_val));
                        });
                });
                Ok(())
            }
        }

        pub fn calculate(
            origin,
            oracle_id: T::OracleId,
            number: u8,
        ) -> SimpleResult
        {
            let mut result = Err("Can't find oracle.");

            Oracles::<T>::mutate(oracle_id, |oracle| {
                if oracle.is_calculate_time(number as usize, timestamp::Module::<T>::get()) {
                    result = oracle.calculate_median(number as usize);
                } else {
                    result = Err("The calculation time has not come.");
                }
            });

            result
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

impl<T: Trait> Module<T>
{
    fn update_accounts(oracle_id: T::OracleId)
    {
        let table_id = Oracles::<T>::get(oracle_id).table;
        Oracles::<T>::mutate(oracle_id, |oracle| {
            oracle.update_accounts(tablescore::Module::<T>::get_head(&table_id))
        });
    }

    fn pop_new_oracle_id() -> Result<T::OracleId, &'static str>
    {
        let mut result = Err("Unknown error");

        OracleIdSequnce::<T>::mutate(|id| match id.checked_add(&One::one())
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
}
