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

        pub fn create(
            origin,
            name: RawString,
            asset_id: AssetId<T>,
            source_calculate_count: u8,
            aggregate_period: TimeInterval<T>,
            calculate_period: TimeInterval<T>,
            assets: AssetsVec<RawString>) -> SimpleResult
        {
            let _ = ensure_signed(origin)?;
            let table = tablescore::Module::<T>::create(asset_id, source_calculate_count, Some(name.clone()))?;

            Oracles::<T>::insert(Self::pop_new_oracle_id()?,
                Oracle::new(name, table, PeriodHandler::new(calculate_period, aggregate_period), source_calculate_count, assets),
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

            let now = timestamp::Module::<T>::get();

            if oracle.period_handler.is_source_update_time(now) {
                Self::update_accounts(oracle_id);
            }

            if values.0.len() != oracle.value.0.len()
            {
                Err("The number of assets does not match")
            }
            else if !oracle.sources.contains_key(&who)
            {
                Err("Your account is not a source for the oracle.")
            }
            else if !oracle.period_handler.is_aggregate_time(now)
            {
                Err("No data aggregation at this time.")
            }
            else
            {
                Oracles::<T>::mutate(oracle_id, |oracle| {
                    if let Some(assets) = oracle.sources.get_mut(&who)
                    {
                        assets.0.iter_mut().zip(values.0.iter()).for_each(|(external, new_val)| external.update(*new_val));
                    }
                    else
                    {
                        oracle.sources.insert(who, AssetsVec { 0: values.0.into_iter().map(ExternalValue::with_value).collect() });
                    }
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
            let _ = ensure_signed(origin)?;
            let mut result = Err("Can't find oracle.");

            Oracles::<T>::mutate(oracle_id, |oracle| {
                if oracle.is_calculate_time(number as usize, timestamp::Module::<T>::get())
                {
                    result = oracle.calculate_median(number as usize);
                }
                else
                {
                    result = Err("The calculation time has not come. Use old value.");
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
        Oracles::<T>::mutate(oracle_id, |oracle| {
            oracle.update_accounts(tablescore::Module::<T>::get_head(&oracle.table))
        });
    }

    fn pop_new_oracle_id() -> Result<T::OracleId, &'static str>
    {
        let mut result = Err("Unknown error");

        OracleIdSequnce::<T>::mutate(|id| match id.checked_add(&One::one())
        {
            Some(res) =>
            {
                result = Ok(*id);
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
