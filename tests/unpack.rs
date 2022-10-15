mod common;
use std::path::PathBuf;

use assert_fs::prelude::*;
use predicates::prelude::*;

use assert_cmd::Command;
use common::{testenv, timewall, TestEnv, EXAMPLE_SUN, EXAMPLE_TIME};
use rstest::rstest;

#[rstest]
#[case(EXAMPLE_SUN.to_path_buf())]
#[case(EXAMPLE_TIME.to_path_buf())]
fn test_unpack(testenv: TestEnv, mut timewall: Command, #[case] input_path: PathBuf) {
    let unpack_path = "unpacked";
    let unpack_dir = testenv.cwd.child(unpack_path);
    unpack_dir.create_dir_all().unwrap();

    timewall.arg("unpack").arg(input_path).arg(unpack_path);
    testenv.run(&mut timewall).success();

    unpack_dir.child("0.png").assert(predicate::path::is_file());
    unpack_dir.child("1.png").assert(predicate::path::is_file());
    unpack_dir
        .child("properties.xml")
        .assert(predicate::path::is_file());
}
