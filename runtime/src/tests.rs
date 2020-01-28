use crate::mock::{new_test_ext, AssetsVec, OracleModule, Origin, ALICE, ASSET_ID};

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
        let oracle = OracleModule::create(
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
        todo!()
    });
}
