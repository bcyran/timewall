#![allow(dead_code)]

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use assert_cmd::{assert::Assert, crate_name, Command};
use assert_fs::{
    prelude::{FileWriteStr, PathChild},
    TempDir,
};
use chrono::{DateTime, Local, TimeZone};
use lazy_static::lazy_static;
use rstest::fixture;

lazy_static! {
    /// Absolute path to example time-based wallpaper.
    pub static ref EXAMPLE_TIME: PathBuf = PathBuf::from("tests/examples/time.heic")
        .canonicalize()
        .unwrap();
    /// Absolute path to example sun-based wallpaper.
    pub static ref EXAMPLE_SUN: PathBuf = PathBuf::from("tests/examples/sun.heic")
        .canonicalize()
        .unwrap();
    /// Absolute path to example unsupported image file.
    pub static ref EXAMPLE_UNSUPPORTED: PathBuf = PathBuf::from("tests/examples/unsupported.jpg")
        .canonicalize()
        .unwrap();
    /// Absolute path to example time-based properties XML file.
    pub static ref PROPERTIES_TIME: PathBuf = PathBuf::from("tests/examples/properties_time.xml")
        .canonicalize()
        .unwrap();
    /// Absolute path to example sun-based properties XML file.
    pub static ref PROPERTIES_SUN: PathBuf = PathBuf::from("tests/examples/properties_sun.xml")
        .canonicalize()
        .unwrap();
    /// Mapping of examples wallpaper paths to their hashes.
    pub static ref WALLPAPER_HASHES: HashMap<PathBuf, &'static str> = HashMap::from([
        (
            EXAMPLE_TIME.to_path_buf(),
            "1ac969ea8c5394f9a3ff800f7907681029c90152599a682ff925adfcd4cf5b94"
        ),
        (
            EXAMPLE_SUN.to_path_buf(),
            "7cd9ab090db69844d39806b1694e1e708d6f06b6eb6c3e723ad1189d6f789a96"
        )
    ]);
    /// Datetime that should result in day-time image in example wallpapers.
    pub static ref DATETIME_DAY: DateTime<Local> = Local.ymd(2022, 10, 18).and_hms(14, 30, 30);
    /// Datetime that should result in night-time image in example wallpapers.
    pub static ref DATETIME_NIGHT: DateTime<Local> = Local.ymd(2022, 10, 18).and_hms(22, 30, 30);
}

/// Name of the image that should be set for day-time in example wallpapers.
pub const IMAGE_DAY: &str = "0.png";
/// Name of the image that should be set for night-time in example wallpapers.
pub const IMAGE_NIGHT: &str = "1.png";
/// Message printed to stdout in dry-run mode to indicate that image is being set as a wallpaper.
pub const IMAGE_SET_MESSAGE: &str = "Set: ";
/// Message printed to stdout in dry-run mode to indicate that command is being run.
pub const COMMAND_RUN_MESSAGE: &str = "Run: ";

/// Get full path to cached wallpaper image as a string.
/// Path is determined based on root cache directory, used wallpaper and name of the image in this
/// wallpaper.
pub fn cached_image_path_str<CP: AsRef<Path>>(
    cache_dir: CP,
    wallpaper: &PathBuf,
    image: &str,
) -> String {
    cache_dir
        .as_ref()
        .join("wallpapers")
        .join(WALLPAPER_HASHES.get(wallpaper).unwrap())
        .join(image)
        .to_str()
        .unwrap()
        .to_owned()
}

/// Test environment for `timewall` binary tests.
/// Sets up temporary directory to run the binary in, overrides config and cache directories,
/// enables dry-run, allows overriding date and time seen by the binary.
pub struct TestEnv {
    pub cwd: TempDir,
    pub config_dir: TempDir,
    pub cache_dir: TempDir,
    datetime: Option<DateTime<Local>>,
}

impl TestEnv {
    pub fn new() -> Self {
        TestEnv {
            cwd: assert_fs::TempDir::new().unwrap(),
            config_dir: assert_fs::TempDir::new().unwrap(),
            cache_dir: assert_fs::TempDir::new().unwrap(),
            datetime: None,
        }
    }

    /// Write given string as a contents of the config file.
    pub fn with_config(self, config: &str) -> Self {
        self.config_dir
            .child("config.toml")
            .write_str(config)
            .unwrap();
        self
    }

    /// Override datetime seen by the binary.
    pub fn with_time(mut self, time: DateTime<Local>) -> Self {
        self.datetime = Some(time);
        self
    }

    /// Run the command and return `Assert` object.
    pub fn run(&self, command: &mut Command) -> Assert {
        command
            .current_dir(&self.cwd)
            .env("TIMEWALL_DRY_RUN", "true")
            .env("TIMEWALL_CONFIG_DIR", &self.config_dir.path())
            .env("TIMEWALL_CACHE_DIR", &self.cache_dir.path());
        if let Some(datetime) = self.datetime {
            command.env("TIMEWALL_OVERRIDE_TIME", datetime.to_rfc3339());
        }
        command.assert()
    }
}

#[fixture]
pub fn timewall() -> Command {
    Command::cargo_bin(crate_name!()).unwrap()
}

#[fixture]
pub fn testenv() -> TestEnv {
    TestEnv::new()
}
