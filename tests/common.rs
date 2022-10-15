use assert_cmd::{assert::Assert, crate_name, Command};
use assert_fs::{
    prelude::{FileWriteStr, PathChild},
    TempDir,
};
#[macro_use]
use lazy_static::lazy_static;
use rstest::fixture;
use std::path::PathBuf;

lazy_static! {
    pub static ref EXAMPLE_TIME: PathBuf = PathBuf::from("tests/examples/time.heic")
        .canonicalize()
        .unwrap();
    pub static ref EXAMPLE_SUN: PathBuf = PathBuf::from("tests/examples/sun.heic")
        .canonicalize()
        .unwrap();
}

pub struct TestEnv {
    pub cwd: TempDir,
    pub config_dir: TempDir,
    pub cache_dir: TempDir,
}

impl TestEnv {
    pub fn new() -> Self {
        TestEnv {
            cwd: assert_fs::TempDir::new().unwrap(),
            config_dir: assert_fs::TempDir::new().unwrap(),
            cache_dir: assert_fs::TempDir::new().unwrap(),
        }
    }

    pub fn with_config(self, config: &str) -> Self {
        self.config_dir
            .child("config.toml")
            .write_str(config)
            .unwrap();
        self
    }

    pub fn run(&self, command: &mut Command) -> Assert {
        command
            .current_dir(&self.cwd)
            .env("TIMEWALL_CONFIG_DIR", &self.config_dir.path())
            .env("TIMEWALL_CACHE_DIR", &self.cache_dir.path())
            .assert()
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
