mod common;

use std::path::PathBuf;

use assert_cmd::Command;
use chrono::{DateTime, Local};
use common::{
    cached_image_path_str, testenv, timewall, TestEnv, DATETIME_DAY, DATETIME_NIGHT, EXAMPLE_SUN,
    EXAMPLE_TIME, IMAGE_DAY, IMAGE_NIGHT, IMAGE_SET_MESSAGE,
};
use predicate::str::contains;
use predicates::prelude::*;
use rstest::rstest;

#[rstest]
#[case(*DATETIME_DAY, IMAGE_DAY)]
#[case(*DATETIME_NIGHT, IMAGE_NIGHT)]
fn test_set_correct_image(
    testenv: TestEnv,
    mut timewall: Command,
    #[values(EXAMPLE_SUN.to_path_buf(), EXAMPLE_TIME.to_path_buf())] wallpaper_path: PathBuf,
    #[case] datetime: DateTime<Local>,
    #[case] expected_image: &str,
) {
    let expected_image_path_str =
        cached_image_path_str(&testenv.cache_dir, &wallpaper_path, expected_image);

    timewall.arg("set").arg(wallpaper_path);
    testenv
        .with_time(datetime)
        .run(&mut timewall)
        .success()
        .stdout(contains(IMAGE_SET_MESSAGE).count(1))
        .stdout(contains(expected_image_path_str));
}
