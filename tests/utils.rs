use assert_cmd::Command;
use rstest::fixture;

pub const EXAMPLE_TIME: &str = "tests/examples/time.heic";
pub const EXAMPLE_SUN: &str = "tests/examples/sun.heic";

#[fixture]
pub fn timewall() -> Command {
    Command::cargo_bin("timewall").unwrap()
}
