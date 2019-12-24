use crate::mock::{
    new_test_ext,
    AccountId,
    Balance,
    Origin,
    TablescoreModule,
    Test,
};

use crate::tablescore::Table;
use crate::Assets;
use rstd::collections::{btree_map::BTreeMap, btree_set::BTreeSet};

const ASSET_ID: u64 = 123;
const ALICE: AccountId = 1;

fn get_test_table() -> Table<Test>
{
    Table::<Test> {
        name: Some("test".to_owned().as_bytes().to_vec()),
        head_count: 10,
        vote_asset: ASSET_ID,
        scores: BTreeSet::new(),
        reserved: BTreeMap::new(),
    }
}

#[test]
fn create_table()
{
    new_test_ext().execute_with(|| {
        let who = Origin::signed(ALICE);
        let id = TablescoreModule::next_asset_id();

        let table = get_test_table();
        assert!(TablescoreModule::create_table(
            who,
            table.vote_asset,
            table.head_count,
            table.name.clone()
        )
        .is_ok());

        assert_eq!(TablescoreModule::scores(&id), table);
    });
}

#[test]
fn vote_reserve_err()
{
    new_test_ext().execute_with(|| {
        let id = TablescoreModule::next_asset_id();
        let balance = 1u128;

        let table = get_test_table();

        assert!(TablescoreModule::create_table(
            Origin::signed(ALICE),
            table.vote_asset,
            table.head_count,
            table.name.clone()
        )
        .is_ok());

        assert!(TablescoreModule::vote(Origin::signed(ALICE), id, balance, 1).is_err());
    });
}

#[test]
fn vote()
{
    new_test_ext().execute_with(|| {
        let id = TablescoreModule::next_asset_id();
        let balance = 1u128;

        let table = get_test_table();

        assert!(TablescoreModule::create_table(
            Origin::signed(ALICE),
            table.vote_asset,
            table.head_count,
            table.name.clone()
        )
        .is_ok());

        Assets::create_asset(
            Some(ASSET_ID),
            Some(ALICE),
            Assets::AssetOptions {
                initial_issuance: 100,
                permissions: Assets::PermissionLatest::default(),
            },
        );
    });
}
