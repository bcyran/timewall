mod common;

use std::path::PathBuf;

use assert_fs::prelude::*;
use chrono::{DateTime, Local};
use common::{
    cached_image_path_str, testenv, TestEnv, COMMAND_RUN_MESSAGE, DATETIME_DAY, DATETIME_NIGHT,
    EXAMPLE_SUN, EXAMPLE_TIME, IMAGE_DAY, IMAGE_NIGHT, IMAGE_SET_MESSAGE, WALLPAPER_HASHES,
};
use predicates::prelude::*;
use rstest::rstest;

#[rstest]
#[case(*DATETIME_DAY, IMAGE_DAY)]
#[case(*DATETIME_NIGHT, IMAGE_NIGHT)]
fn test_sets_correct_image(
    testenv: TestEnv,
    #[values(EXAMPLE_SUN.to_path_buf(), EXAMPLE_TIME.to_path_buf())] wall_path: PathBuf,
    #[case] datetime: DateTime<Local>,
    #[case] expected_image: &str,
) {
    let expected_image_path_str =
        cached_image_path_str(&testenv.cache_dir, &wall_path, expected_image);

    testenv
        .with_time(datetime)
        .run(&["set", wall_path.to_str().unwrap()])
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
    let wall_path = EXAMPLE_SUN.to_path_buf();
    let config =
        "[location]\nlat = 51.11\nlon = 17.02\n[setter]\ncommand = ['feh', '--bg-fill', '%f']";
    let expected_image_path_str =
        cached_image_path_str(&testenv.cache_dir, &wall_path, IMAGE_NIGHT);
    let expected_command_str = format!("feh --bg-fill {expected_image_path_str}");

    testenv
        .with_config(&config)
        .with_time(*DATETIME_NIGHT)
        .run(&["set", wall_path.to_str().unwrap()])
        .success()
        .stdout(predicate::str::contains(COMMAND_RUN_MESSAGE).count(1))
        .stdout(predicate::str::contains(expected_command_str));
}

#[rstest]
fn test_creates_config(testenv: TestEnv) {
    let config_path = testenv.config_dir.child("config.toml");
    let expected_config = "[location]\nlat = 51.11\nlon = 17.02\n";
    let expected_stderr = format!("Default config written to {}", config_path.display());

    testenv
        .run(&["set", EXAMPLE_SUN.to_str().unwrap()])
        .success()
        .stderr(predicate::str::contains(expected_stderr));

    config_path
        .assert(predicate::path::is_file())
        .assert(predicate::eq(expected_config));
}

#[rstest]
fn test_saves_last_wallpaper(testenv: TestEnv) {
    let wall_path = EXAMPLE_SUN.to_path_buf();

    testenv.run(&["set", wall_path.to_str().unwrap()]).success();

    let saved_wallpaper = testenv.cache_dir.child("last_wall");
    assert!(saved_wallpaper.is_symlink());
    saved_wallpaper.assert(predicate::path::eq_file(wall_path));
}

#[rstest]
fn test_caches_wallpaper(testenv: TestEnv) {
    let wall_path = EXAMPLE_SUN.to_path_buf();

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
