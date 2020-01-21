use crate::mock::{
    new_test_ext, MockModule, Origin, Test, ALICE, ASSET_ID, BALANCE, BOB, CAROL, CHUCK,
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
        let id = MockModule::next_tablescore_id();

        let table = get_test_table();
        assert!(MockModule::create_table(
            who,
            table.vote_asset,
            table.head_count,
            table.name.clone()
        )
        .is_ok());

        assert_eq!(MockModule::scores(&id), table);
    });
}

#[test]
fn vote_reserve_err_tablescore()
{
    new_test_ext().execute_with(|| {
        let id = MockModule::next_tablescore_id();
        let table = get_test_table();

        assert!(MockModule::create_table(
            Origin::signed(ALICE),
            table.vote_asset,
            table.head_count,
            table.name.clone()
        )
        .is_ok());

        assert!(MockModule::vote(Origin::signed(ALICE), id, BALANCE + 1, 1).is_err());
    });
}

#[test]
fn vote_tablescore()
{
    new_test_ext().execute_with(|| {
        let id = MockModule::next_tablescore_id();
        let table = get_test_table();
        assert!(MockModule::create_table(
            Origin::signed(ALICE),
            ASSET_ID,
            table.head_count,
            table.name.clone()
        )
        .is_ok());

        assert!(MockModule::vote(Origin::signed(ALICE), id, 3u128, 1).is_ok());
        assert!(MockModule::vote(Origin::signed(BOB), id, 2u128, 2).is_ok());
        assert!(MockModule::vote(Origin::signed(CAROL), id, 1u128, 3).is_ok());

        assert_eq!(MockModule::get_head(&id), vec![1, 2]);

        assert!(MockModule::vote(Origin::signed(CAROL), id, 4u128, 3).is_ok());

        assert_eq!(MockModule::get_head(&id), vec![3, 1]);
    });
}

#[inline]
fn to_raw(data: &'static str) -> Vec<u8>
{
    data.to_owned().as_bytes().to_vec()
}

#[test]
fn create_oracle()
{
    new_test_ext().execute_with(|| {
        let id = MockModule::next_oracle_id();
        MockModule::create_oracle(
            Origin::signed(ALICE),
            Some("test".to_owned().as_bytes().to_vec()),
            ASSET_ID,
            5,
            100,
            100,
            ["one", "two", "three"].iter().map(to_raw).collect(),
        );
    });
}

#[test]
fn calculate_oracle() {}

#[test]
fn recalculate_oracle() {}

#[test]
fn commit_in_oracle() {}
