#![allow(
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::new_without_default,
    clippy::missing_const_for_fn,
    clippy::too_long_first_doc_paragraph,
    clippy::use_self
)]

mod common;
use std::path::PathBuf;

use assert_fs::prelude::*;
use common::{
    testenv, TestEnv, EXAMPLE_SUN, EXAMPLE_TIME, IMAGE_DAY, IMAGE_NIGHT, PROPERTIES_SUN,
    PROPERTIES_TIME,
};
use predicates::prelude::*;
use rstest::rstest;

#[rstest]
#[case(EXAMPLE_SUN.to_path_buf(), PROPERTIES_SUN.to_path_buf())]
#[case(EXAMPLE_TIME.to_path_buf(), PROPERTIES_TIME.to_path_buf())]
fn test_unpack(testenv: TestEnv, #[case] wall_path: PathBuf, #[case] expected_properties: PathBuf) {
    let unpack_path = "unpacked";
    let unpack_dir = testenv.cwd.child(unpack_path);
    unpack_dir.create_dir_all().unwrap();

    testenv
        .run(&[
            "unpack",
            wall_path.to_str().unwrap(),
            unpack_dir.to_str().unwrap(),
        ])
        .success();

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
