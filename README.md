# Substrate DPOS Oracle

DPOS oracle substrate-module written in Rust.

## Build

### Install dependencies

Run script `./rustup.sh`.

### Build

```bash
# Simple build
cargo build

# Run unit tests
cargo test 

# CI test 
# ToDo in progress

```

### How it works

We have two modules:
- Tablescore - can create table with accounts and vote for tham by locking generic-assets
- Dpos Oracle - can calculate from accounts external values data to one caclulated external assets vector by DPOS algorithm

In Tablescore module you can call:
    - `vote` - lock part of your assets to one of target;
    - `get_head` - get top of table;

In DposOracle module you can call:
    - `commit` - commit value to oracle, if you have permission (from tablescore module);
    - `calculate` - update one external asset value;

### Example SRML module

```rust
pub trait Trait: oracle::Trait {...}

// In `decl_module` part you can get value from oracle
fn func(origin) {
    let oracle_id = ...;
    let external_asset_id = ...;
    let oracle: OracleData = oracle::Oracles::<T>::get(oracle_id).unwrap();

    if let Some(value) = oracle.value.0[external_asset_id].value {
        // Value here if calculated from sources value
    }
}

// In `decl_module` part you can create oracle
#[inline]
fn to_raw(data: &&'static str) -> Vec<u8> {
    data.to_owned().as_bytes().to_vec()
}

fn func(origin) {
    oracle::OracleModule::create_oracle(
        who,
        to_raw("name_of_oracle"),
        ASSET_ID, // Id of vote asset
        5, // Minimum of accounts count for DPOS
        60, // Period for aggregate data from sources-accounts
        120, // Period for calculate data from aggregated
        AssetsVec { // Assets names
            0: ["one", "two", "three"].iter().map(to_raw).collect(),
        },
}

```
