mod common;

use std::path::PathBuf;

use assert_cmd::Command;
use assert_fs::prelude::*;
use chrono::{DateTime, Local};
use common::{
    cached_image_path_str, testenv, timewall, TestEnv, DATETIME_DAY, DATETIME_NIGHT, EXAMPLE_SUN,
    EXAMPLE_TIME, IMAGE_DAY, IMAGE_NIGHT, IMAGE_SET_MESSAGE, WALLPAPER_HASHES,
};
use predicates::prelude::*;
use rstest::rstest;

#[rstest]
#[case(*DATETIME_DAY, IMAGE_DAY)]
#[case(*DATETIME_NIGHT, IMAGE_NIGHT)]
fn test_sets_correct_image(
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
        .stdout(predicate::str::contains(IMAGE_SET_MESSAGE).count(1))
        .stdout(predicate::str::contains(expected_image_path_str));
}

#[rstest]
fn test_creates_config(testenv: TestEnv, mut timewall: Command) {
    let expected_config = "[location]\nlat = 51.11\nlon = 17.02\n";

    timewall.arg("set").arg(EXAMPLE_SUN.to_path_buf());
    testenv.run(&mut timewall).success();
    testenv
        .config_dir
        .child("config.toml")
        .assert(predicate::path::is_file())
        .assert(predicate::eq(expected_config));
}

#[rstest]
fn test_saves_last_wallpaper(testenv: TestEnv, mut timewall: Command) {
    let expected_wallpaper = EXAMPLE_SUN.to_path_buf();

    timewall.arg("set").arg(&expected_wallpaper);
    testenv.run(&mut timewall).success();

    let saved_wallpaper = testenv.cache_dir.child("last_wall");
    assert!(saved_wallpaper.is_symlink());
    saved_wallpaper.assert(predicate::path::eq_file(expected_wallpaper));
}

#[rstest]
fn test_caches_wallpaper(testenv: TestEnv, mut timewall: Command) {
    let expected_wallpaper = EXAMPLE_SUN.to_path_buf();

    timewall.arg("set").arg(&expected_wallpaper);
    testenv.run(&mut timewall).success();

    let wallpaper_cache_dir = testenv
        .cache_dir
        .child("wallpapers")
        .child(WALLPAPER_HASHES.get(&expected_wallpaper).unwrap());
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
