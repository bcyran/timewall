use std::{fs, path::Path};

use anyhow::{anyhow, Context, Result};
use directories::ProjectDirs;
use serde::Deserialize;

use crate::constants::{APP_NAME, APP_QUALIFIER};
use crate::geo::Coords;

#[derive(Deserialize, PartialEq, Debug)]
pub struct Config {
    pub coords: Coords,
    pub setter_cmd: Option<String>,
}

impl Config {
    pub fn find() -> Result<Self> {
        match ProjectDirs::from(APP_QUALIFIER, "", APP_NAME) {
            Some(app_dirs) => Config::load(app_dirs.config_dir().join("conf.toml")),
            None => Err(anyhow!("couldn't determine user's home directory")),
        }
    }

    fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_str = fs::read_to_string(path)
            .with_context(|| format!("couldn't read the configuration file"))?;
        let config: Config = toml::from_str(&config_str)
            .with_context(|| format!("couldn't parse the configuation file"))?;
        Ok(config)
    }
}
