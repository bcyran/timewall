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

use common::{testenv, TestEnv, EXAMPLE_UNSUPPORTED};
use predicates::prelude::*;
use rstest::rstest;

const EXAMPLE_NOT_EXISTING: &str = "just/some/random/path";
const EXAMPLE_DIR: &str = ".";

#[rstest]
#[case(&["info", EXAMPLE_UNSUPPORTED.to_str().unwrap()])]
#[case(&["preview", EXAMPLE_UNSUPPORTED.to_str().unwrap()])]
#[case(&["set", EXAMPLE_UNSUPPORTED.to_str().unwrap()])]
#[case(&["unpack", EXAMPLE_UNSUPPORTED.to_str().unwrap(), "out"])]
fn test_unsupported_image(testenv: TestEnv, #[case] args: &[&str]) {
    let expected_message = "Error: only HEIF files are supported";

    testenv
        .run(args)
        .failure()
        .stderr(predicate::str::contains(expected_message));
}

#[rstest]
#[case(&["info", EXAMPLE_NOT_EXISTING])]
#[case(&["preview", EXAMPLE_NOT_EXISTING])]
#[case(&["set", EXAMPLE_NOT_EXISTING])]
#[case(&["unpack", EXAMPLE_NOT_EXISTING, "out"])]
fn test_not_existing_path(testenv: TestEnv, #[case] args: &[&str]) {
    let expected_message = format!("Error: file '{EXAMPLE_NOT_EXISTING}' is not accessible");

    testenv
        .run(args)
        .failure()
        .stderr(predicate::str::contains(expected_message));
}

#[rstest]
#[case(&["info", EXAMPLE_DIR])]
#[case(&["preview", EXAMPLE_DIR])]
#[case(&["set", EXAMPLE_DIR])]
#[case(&["unpack", EXAMPLE_DIR, "out"])]
fn test_dir_path(testenv: TestEnv, #[case] args: &[&str]) {
    let expected_message = format!("Error: '{EXAMPLE_DIR}' is not a file");

    testenv
        .run(args)
        .failure()
        .stderr(predicate::str::contains(expected_message));
}
