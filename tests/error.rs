mod common;

use assert_cmd::Command;
use common::{testenv, timewall, TestEnv, EXAMPLE_UNSUPPORTED};
use predicates::prelude::*;
use rstest::rstest;

const EXAMPLE_NOT_EXISTING: &str = "just/some/random/path";
const EXAMPLE_DIR: &str = ".";

#[rstest]
#[case(&["info", EXAMPLE_UNSUPPORTED.to_str().unwrap()])]
#[case(&["preview", EXAMPLE_UNSUPPORTED.to_str().unwrap()])]
#[case(&["set", EXAMPLE_UNSUPPORTED.to_str().unwrap()])]
#[case(&["unpack", EXAMPLE_UNSUPPORTED.to_str().unwrap(), "out"])]
fn test_unsupported_image(testenv: TestEnv, mut timewall: Command, #[case] args: &[&str]) {
    let expected_message = "Error: only HEIF files are supported";

    timewall.args(args);
    testenv
        .run(&mut timewall)
        .failure()
        .stderr(predicate::str::contains(expected_message));
}

#[rstest]
#[case(&["info", EXAMPLE_NOT_EXISTING])]
#[case(&["preview", EXAMPLE_NOT_EXISTING])]
#[case(&["set", EXAMPLE_NOT_EXISTING])]
#[case(&["unpack", EXAMPLE_NOT_EXISTING, "out"])]
fn test_not_existing_path(testenv: TestEnv, mut timewall: Command, #[case] args: &[&str]) {
    let expected_message = format!("Error: file '{EXAMPLE_NOT_EXISTING}' is not accessible");

    timewall.args(args);
    testenv
        .run(&mut timewall)
        .failure()
        .stderr(predicate::str::contains(expected_message));
}

#[rstest]
#[case(&["info", EXAMPLE_DIR])]
#[case(&["preview", EXAMPLE_DIR])]
#[case(&["set", EXAMPLE_DIR])]
#[case(&["unpack", EXAMPLE_DIR, "out"])]
fn test_dir_path(testenv: TestEnv, mut timewall: Command, #[case] args: &[&str]) {
    let expected_message = format!("Error: '{EXAMPLE_DIR}' is not a file");

    timewall.args(args);
    testenv
        .run(&mut timewall)
        .failure()
        .stderr(predicate::str::contains(expected_message));
}
