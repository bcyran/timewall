use std::fs::File;
use std::io::Write;
use std::{fs, path::Path};

use anyhow::{anyhow, Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::constants::{APP_NAME, APP_QUALIFIER};
use crate::geo::Coords;

const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Deserialize, Serialize, Debug)]
pub struct Setter {
    pub cmd: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub location: Coords,
    pub setter: Option<Setter>,
}

impl Config {
    pub fn find() -> Result<Self> {
        match ProjectDirs::from(APP_QUALIFIER, "", APP_NAME) {
            Some(app_dirs) => Config::load_or_create(app_dirs.config_dir().join(CONFIG_FILE_NAME)),
            None => Err(anyhow!("couldn't determine user's home directory")),
        }
    }

    fn load_or_create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if path.exists() {
            Config::load(path)
        } else {
            let config = Config::default();
            config
                .write(path)
                .with_context(|| "couldn't write the configuration file")?;
            eprintln!("Default config written to {}.", path.display());
            eprintln!("You should probably adjust it to your needs!");
            Ok(config)
        }
    }

    fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let config_str = fs::read_to_string(path)
            .with_context(|| format!("couldn't read the configuration file"))?;
        let config: Config = toml::from_str(&config_str)
            .with_context(|| format!("couldn't parse the configuation file"))?;
        Ok(config)
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let mut config_file = File::create(path)?;
        config_file.write_all(toml::to_string_pretty(self)?.as_bytes())?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            location: Coords {
                lat: 51.11,
                lon: 17.02,
            },
            setter: None,
        }
    }
}
