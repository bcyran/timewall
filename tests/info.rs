mod utils;

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;
use utils::{timewall, LENNA_TIME};

#[rstest]
fn test_info_time(mut timewall: Command) {
    let expected_stdout = predicate::str::ends_with(
        r#"Size: 88723B
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
"#,
    );
    timewall.arg("info").arg(LENNA_TIME);

    timewall.assert().success().stdout(expected_stdout);
}
