mod common;

use std::path::PathBuf;

use common::{testenv, TestEnv, EXAMPLE_SUN, EXAMPLE_TIME};
use predicates::prelude::*;
use rstest::rstest;

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
fn test_info(testenv: TestEnv, #[case] wall_path: PathBuf, #[case] expected_output: &str) {
    testenv
        .run(&["info", wall_path.to_str().unwrap()])
        .success()
        .stdout(predicate::str::ends_with(expected_output));
}
