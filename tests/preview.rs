mod common;
use std::path::PathBuf;

use assert_cmd::Command;
use common::{
    testenv, timewall, TestEnv, EXAMPLE_SUN, EXAMPLE_TIME, IMAGE_DAY, IMAGE_NIGHT,
    IMAGE_SET_MESSAGE,
};
use predicates::prelude::*;
use rstest::rstest;

#[rstest]
#[case(EXAMPLE_SUN.to_path_buf())]
#[case(EXAMPLE_TIME.to_path_buf())]
fn test_preview(testenv: TestEnv, mut timewall: Command, #[case] wall_path: PathBuf) {
    testenv
        .run(timewall.arg("preview").arg(wall_path))
        .success()
        .stdout(predicate::str::contains(IMAGE_SET_MESSAGE).count(2))
        .stdout(predicate::str::contains(IMAGE_DAY).count(1))
        .stdout(predicate::str::contains(IMAGE_NIGHT).count(1));
}
