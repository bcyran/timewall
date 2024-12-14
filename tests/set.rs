mod common;

use std::path::PathBuf;

use assert_fs::prelude::*;
use chrono::{DateTime, Local};
use common::{
    cached_image_path_str, testenv, TestEnv, COMMAND_RUN_MESSAGE, CONFIG_WITH_COMMAND,
    CONFIG_WITH_LOCATION, DATETIME_DAY, DATETIME_NIGHT, EXAMPLE_SUN, EXAMPLE_TIME, IMAGE_DAY,
    IMAGE_NIGHT, IMAGE_SET_MESSAGE, WALLPAPER_HASHES,
};
use predicates::prelude::*;
use rstest::rstest;

#[rstest]
#[case(*DATETIME_DAY, IMAGE_DAY)]
#[case(*DATETIME_NIGHT, IMAGE_NIGHT)]
fn test_sets_correct_image_time(
    testenv: TestEnv,
    #[case] datetime: DateTime<Local>,
    #[case] expected_image: &str,
) {
    let expected_image_path_str =
        cached_image_path_str(&testenv.cache_dir, &EXAMPLE_TIME, expected_image);

    testenv
        .with_time(datetime)
        .run(&["set", EXAMPLE_TIME.to_str().unwrap()])
        .success()
        .stdout(predicate::str::contains(IMAGE_SET_MESSAGE).count(1))
        .stdout(predicate::str::contains(expected_image_path_str));
}

#[rstest]
#[case(*DATETIME_DAY, IMAGE_DAY)]
#[case(*DATETIME_NIGHT, IMAGE_NIGHT)]
fn test_sets_correct_image_solar(
    testenv: TestEnv,
    #[case] datetime: DateTime<Local>,
    #[case] expected_image: &str,
) {
    let expected_image_path_str =
        cached_image_path_str(&testenv.cache_dir, &EXAMPLE_SUN, expected_image);

    testenv
        .with_config(CONFIG_WITH_LOCATION)
        .with_time(datetime)
        .run(&["set", EXAMPLE_SUN.to_str().unwrap()])
        .success()
        .stdout(predicate::str::contains(IMAGE_SET_MESSAGE).count(1))
        .stdout(predicate::str::contains(expected_image_path_str));
}

#[rstest]
#[case("light", IMAGE_DAY)]
#[case("dark", IMAGE_NIGHT)]
fn test_sets_correct_image_appearance(
    testenv: TestEnv,
    #[values(EXAMPLE_SUN.to_path_buf(), EXAMPLE_TIME.to_path_buf())] wall_path: PathBuf,
    #[values(*DATETIME_DAY, *DATETIME_NIGHT)] datetime: DateTime<Local>,
    #[case] appearance_value: &str,
    #[case] expected_image: &str,
) {
    let expected_image_path_str =
        cached_image_path_str(&testenv.cache_dir, &wall_path, expected_image);

    testenv
        .with_time(datetime)
        .run(&[
            "set",
            "--appearance",
            appearance_value,
            wall_path.to_str().unwrap(),
        ])
        .success()
        .stdout(predicate::str::contains(IMAGE_SET_MESSAGE).count(1))
        .stdout(predicate::str::contains(expected_image_path_str));
}

#[rstest]
fn test_runs_command(testenv: TestEnv) {
    let wall_path = EXAMPLE_TIME.to_path_buf();
    let expected_image_path_str =
        cached_image_path_str(&testenv.cache_dir, &wall_path, IMAGE_NIGHT);
    let expected_command_str = format!("feh --bg-fill {expected_image_path_str}");

    testenv
        .with_config(CONFIG_WITH_COMMAND)
        .with_time(*DATETIME_NIGHT)
        .run(&["set", wall_path.to_str().unwrap()])
        .success()
        .stdout(predicate::str::contains(COMMAND_RUN_MESSAGE).count(1))
        .stdout(predicate::str::contains(expected_command_str));
}

#[rstest]
fn test_creates_config(testenv: TestEnv) {
    let config_path = testenv.config_dir.child("config.toml");
    let expected_stderr = format!("Default config written to {}", config_path.display());

    testenv
        .run(&["set", EXAMPLE_TIME.to_str().unwrap()])
        .success()
        .stderr(predicate::str::contains(expected_stderr));

    config_path.assert(predicate::path::is_file());
}

#[rstest]
fn test_saves_last_wallpaper(testenv: TestEnv) {
    let wall_path = EXAMPLE_TIME.to_path_buf();
    let expected_image_path_str =
        cached_image_path_str(&testenv.cache_dir, &wall_path, IMAGE_NIGHT);

    let testenv = testenv.with_time(*DATETIME_NIGHT);
    testenv
        .run(&["set", wall_path.to_str().unwrap()])
        .success()
        .stdout(predicate::str::contains(IMAGE_SET_MESSAGE).count(1))
        .stdout(predicate::str::contains(&expected_image_path_str).count(1));
    testenv
        .run(&["set"])
        .success()
        .stdout(predicate::str::contains(IMAGE_SET_MESSAGE).count(1))
        .stdout(predicate::str::contains(&expected_image_path_str).count(1));
}

#[rstest]
fn test_caches_wallpaper(testenv: TestEnv) {
    let wall_path = EXAMPLE_TIME.to_path_buf();

    testenv.run(&["set", wall_path.to_str().unwrap()]).success();

    let wallpaper_cache_dir = testenv
        .cache_dir
        .child("wallpapers")
        .child(WALLPAPER_HASHES.get(&wall_path).unwrap());
    wallpaper_cache_dir.assert(predicate::path::is_dir());
    wallpaper_cache_dir
        .child(IMAGE_DAY)
        .assert(predicate::path::is_file());
    wallpaper_cache_dir
        .child(IMAGE_NIGHT)
        .assert(predicate::path::is_file());
    wallpaper_cache_dir
        .child("properties.xml")
        .assert(predicates::path::is_file());
}
