mod common;
use std::path::PathBuf;

use assert_cmd::Command;
use assert_fs::prelude::*;
use common::{
    testenv, timewall, TestEnv, EXAMPLE_SUN, EXAMPLE_TIME, IMAGE_DAY, IMAGE_NIGHT, PROPERTIES_SUN,
    PROPERTIES_TIME,
};
use predicates::prelude::*;
use rstest::rstest;

#[rstest]
#[case(EXAMPLE_SUN.to_path_buf(), PROPERTIES_SUN.to_path_buf())]
#[case(EXAMPLE_TIME.to_path_buf(), PROPERTIES_TIME.to_path_buf())]
fn test_unpack(
    testenv: TestEnv,
    mut timewall: Command,
    #[case] input_path: PathBuf,
    #[case] expected_properties: PathBuf,
) {
    let unpack_path = "unpacked";
    let unpack_dir = testenv.cwd.child(unpack_path);
    unpack_dir.create_dir_all().unwrap();

    timewall.arg("unpack").arg(input_path).arg(unpack_path);
    testenv.run(&mut timewall).success();

    unpack_dir
        .child(IMAGE_DAY)
        .assert(predicate::path::is_file());
    unpack_dir
        .child(IMAGE_NIGHT)
        .assert(predicate::path::is_file());
    unpack_dir
        .child("properties.xml")
        .assert(predicate::path::eq_file(expected_properties));
}
