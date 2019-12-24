use crate::mock::{
    new_test_ext,
    Origin,
    TablescoreModule
};

#[test]
fn create_table()
{
    new_test_ext().execute_with(|| {
        let who = Origin::signed(1);
        assert!(TablescoreModule::create_table(who, 0, 10, None).is_ok());
        TablescoreModule::Scores::get(1);
    });
}

#[test]
fn vote() {}

#[test]
fn get_head() {}
