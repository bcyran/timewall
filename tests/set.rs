mod common;

use crate::common::timewall;
use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;
use rstest::{fixture, rstest};

#[rstest]
fn test_set(mut timewall: Command) {}
