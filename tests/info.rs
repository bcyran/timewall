mod common;

use assert_cmd::Command;
use common::{timewall, EXAMPLE_SUN, EXAMPLE_TIME};
use predicates::prelude::*;
use rstest::rstest;
use std::path::PathBuf;

const EXAMPLE_TIME_INFO: &str = r#"
Size: 88723B
Resolution: 512x512px
Schedule type: time
Number of images: 2
Number of frames: 2
Schedule:
Frame Image Time
1     1     00:00:00
2     0     12:00:00
Appearance:
Light: 0
Dark: 1
"#;

const EXAMPLE_SUN_INFO: &str = r#"
Size: 91566B
Resolution: 512x512px
Schedule type: solar
Number of images: 2
Number of frames: 2
Schedule:
Frame Image Azimuth Altitude
1     0     169.0   31.0
2     1     346.0   -45.0
Appearance:
Light: 0
Dark: 1
"#;

#[rstest]
#[case(EXAMPLE_TIME.to_path_buf(), EXAMPLE_TIME_INFO)]
#[case(EXAMPLE_SUN.to_path_buf(), EXAMPLE_SUN_INFO)]
fn test_info(mut timewall: Command, #[case] input_path: PathBuf, #[case] expected_output: &str) {
    let expected_stdout = predicate::str::ends_with(expected_output);
    timewall.arg("info").arg(input_path);

    timewall.assert().success().stdout(expected_stdout);
}
