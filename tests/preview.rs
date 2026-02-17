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

use common::{
    testenv, TestEnv, EXAMPLE_SUN, EXAMPLE_TIME, IMAGE_DAY, IMAGE_NIGHT, IMAGE_SET_MESSAGE,
};
use predicates::prelude::*;
use rstest::rstest;

#[rstest]
#[case(EXAMPLE_SUN.to_path_buf())]
#[case(EXAMPLE_TIME.to_path_buf())]
fn test_preview(testenv: TestEnv, #[case] wall_path: PathBuf) {
    testenv
        .run(&["preview", wall_path.to_str().unwrap()])
        .success()
        .stdout(predicate::str::contains(IMAGE_SET_MESSAGE).count(2))
        .stdout(predicate::str::contains(IMAGE_DAY).count(1))
        .stdout(predicate::str::contains(IMAGE_NIGHT).count(1));
}
