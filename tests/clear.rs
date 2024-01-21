mod common;

use common::{cached_wallpaper_dir, testenv, TestEnv, EXAMPLE_SUN, EXAMPLE_TIME};
use predicates::prelude::*;
use rstest::rstest;

#[rstest]
fn test_clear(testenv: TestEnv) {
    let expected_first_wall_cache_dir = cached_wallpaper_dir(&testenv.cache_dir, &EXAMPLE_SUN);
    let expected_second_wall_cache_dir = cached_wallpaper_dir(&testenv.cache_dir, &EXAMPLE_TIME);

    testenv
        .run(&["set", EXAMPLE_SUN.to_str().unwrap()])
        .success();
    testenv
        .run(&["set", EXAMPLE_TIME.to_str().unwrap()])
        .success();
    assert!(predicates::path::is_dir().eval(&expected_first_wall_cache_dir));
    assert!(predicates::path::is_dir().eval(&expected_second_wall_cache_dir));

    testenv.run(&["clear"]).success();
    assert!(predicates::path::missing().eval(&expected_first_wall_cache_dir));
    assert!(predicates::path::is_dir().eval(&expected_second_wall_cache_dir));

    testenv.run(&["clear", "--all"]).success();
    assert!(predicates::path::missing().eval(&expected_first_wall_cache_dir));
    assert!(predicates::path::missing().eval(&expected_second_wall_cache_dir));
}
