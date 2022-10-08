use assert_cmd::Command;
use rstest::fixture;

pub const LENNA_TIME: &str = "tests/examples/lenna_time.heic";

#[fixture]
pub fn timewall() -> Command {
    Command::cargo_bin("timewall").unwrap()
}
