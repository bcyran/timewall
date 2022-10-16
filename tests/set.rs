mod common;

use crate::common::timewall;
use assert_cmd::Command;
use common::{testenv, TestEnv, EXAMPLE_SUN, EXAMPLE_TIME};
use rstest::rstest;
use std::path::PathBuf;

// #[rstest]
// #[case(EXAMPLE_SUN.to_path_buf())]
// #[case(EXAMPLE_TIME.to_path_buf())]
// fn test_set(testenv: TestEnv, mut timewall: Command, #[case] input_path: PathBuf) {
//     timewall.arg("set").arg(input_path);
//     testenv.run(&mut timewall).success();
// }
