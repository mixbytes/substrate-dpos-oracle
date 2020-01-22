use crate::mock::{
    new_test_ext, AssetsVec, OracleModule, Origin, TablescoreModule, Test, ALICE, ASSET_ID,
    BALANCE, BOB, CAROL, CHUCK,
};

use crate::tablescore::Table;
use rstd::collections::{btree_map::BTreeMap, btree_set::BTreeSet};

fn get_test_table() -> Table<Test>
{
    Table::<Test> {
        name: Some("test".to_owned().as_bytes().to_vec()),
        head_count: 2,
        vote_asset: ASSET_ID,
        scores: BTreeSet::new(),
        reserved: BTreeMap::new(),
    }
}

#[test]
fn create_tablescore()
{
    new_test_ext().execute_with(|| {
        let who = Origin::signed(ALICE);
        let id = TablescoreModule::next_tablescore_id();

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
fn vote_reserve_err_tablescore()
{
    new_test_ext().execute_with(|| {
        let id = TablescoreModule::next_tablescore_id();
        let table = get_test_table();

        assert!(TablescoreModule::create_table(
            Origin::signed(ALICE),
            table.vote_asset,
            table.head_count,
            table.name.clone()
        )
        .is_ok());

        assert!(TablescoreModule::vote(Origin::signed(ALICE), id, BALANCE + 1, 1).is_err());
    });
}

#[test]
fn vote_tablescore()
{
    new_test_ext().execute_with(|| {
        let id = TablescoreModule::next_tablescore_id();
        let table = get_test_table();
        assert!(TablescoreModule::create_table(
            Origin::signed(ALICE),
            ASSET_ID,
            table.head_count,
            table.name.clone()
        )
        .is_ok());

        assert!(TablescoreModule::vote(Origin::signed(ALICE), id, 3u128, 1).is_ok());
        assert!(TablescoreModule::vote(Origin::signed(BOB), id, 2u128, 2).is_ok());
        assert!(TablescoreModule::vote(Origin::signed(CAROL), id, 1u128, 3).is_ok());

        assert_eq!(TablescoreModule::get_head(&id), vec![1, 2]);

        assert!(TablescoreModule::vote(Origin::signed(CAROL), id, 4u128, 3).is_ok());

        assert_eq!(TablescoreModule::get_head(&id), vec![3, 1]);
    });
}

#[inline]
fn to_raw(data: &&'static str) -> Vec<u8>
{
    data.to_owned().as_bytes().to_vec()
}

#[test]
fn create_oracle()
{
    new_test_ext().execute_with(|| {
        let id = OracleModule::next_oracle_id();
        OracleModule::create(
            Origin::signed(ALICE),
            "test".to_owned().as_bytes().to_vec(),
            ASSET_ID,
            5,
            60,
            120,
            AssetsVec {
                0: ["one", "two", "three"].iter().map(to_raw).collect(),
            },
        );
    });
}

#[test]
fn calculate_oracle() {}

#[test]
fn recalculate_oracle() {}

#[test]
fn commit_in_oracle() {}
