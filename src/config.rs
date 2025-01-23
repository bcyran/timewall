use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context, Ok, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::constants::{APP_NAME, APP_QUALIFIER};
use crate::geo::Coords;

const CONFIG_FILE_NAME: &str = "config.toml";

const DEFAULT_CONFIG_FILE_CONTENT: &str = "\
# Configuration file for timewall

# Dynamic location service
# [geoclue]
# enable = true
# prefer = false
# timeout = 1000

# Set your geographical location coordinates here
# [location]
# lat = 51.11
# lon = 17.02

# Uncomment and adjust the following section to use a custom wallpaper setter command.
# The example uses `swww`: https://github.com/LGFae/swww.
# [setter]
# command = ['swww', 'img', '%f']
# overlap = 0
# quiet = true

# Change how often the wallpaper is updated in daemon mode
# [daemon]
# update_interval_seconds = 300
";

#[derive(Deserialize, Serialize, Debug)]
pub struct Setter {
    pub command: Vec<String>,
    #[serde(default = "Setter::overlap_default_value")]
    pub overlap: u64,
    #[serde(default = "Setter::quiet_default_value")]
    pub quiet: bool,
}

impl Setter {
    const fn overlap_default_value() -> u64 {
        0
    }

    const fn quiet_default_value() -> bool {
        true
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub struct Daemon {
    pub update_interval_seconds: u64,
}

impl Default for Daemon {
    fn default() -> Self {
        Self {
            update_interval_seconds: 5 * 60,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub struct Geoclue {
    #[serde(default = "Geoclue::enable_default_value")]
    pub enable: bool,
    #[serde(default = "Geoclue::prefer_default_value")]
    pub prefer: bool,
    #[serde(default = "Geoclue::timeout_default_value")]
    pub timeout: u64,
}

impl Geoclue {
    const fn enable_default_value() -> bool {
        true
    }

    const fn prefer_default_value() -> bool {
        false
    }

    const fn timeout_default_value() -> u64 {
        1000
    }
}

impl Default for Geoclue {
    fn default() -> Self {
        Self {
            enable: Self::enable_default_value(),
            prefer: Self::prefer_default_value(),
            timeout: Self::timeout_default_value(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub daemon: Daemon,
    #[serde(default)]
    pub geoclue: Geoclue,
    pub location: Option<Coords>,
    pub setter: Option<Setter>,
}

impl Config {
    pub fn find() -> Result<Self> {
        Self::load_or_create(Self::find_path()?)
    }

    pub fn find_path() -> Result<PathBuf> {
        let config_dir = if let Result::Ok(path_str) = env::var("TIMEWALL_CONFIG_DIR") {
            PathBuf::from(path_str)
        } else {
            match ProjectDirs::from(APP_QUALIFIER, "", APP_NAME) {
                Some(app_dirs) => app_dirs.config_dir().to_path_buf(),
                None => bail!("couldn't determine user's home directory"),
            }
        };
        Ok(config_dir.join(CONFIG_FILE_NAME))
    }

    fn load_or_create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            Self::create_default(path)?;
        }
        Self::load(path)
    }

    fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let config_str =
            fs::read_to_string(path).with_context(|| "couldn't read the configuration file")?;
        let config: Self =
            toml::from_str(&config_str).with_context(|| "couldn't parse the configuation file")?;
        Ok(config)
    }

    fn create_default<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        let config_dir = path.parent().unwrap();
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).context("couldn't create config directory")?;
        }

        fs::write(path, DEFAULT_CONFIG_FILE_CONTENT).with_context(|| {
            format!("couldn't write default configuration to {}", path.display())
        })?;

        eprintln!("Default config written to {}.", path.display());
        eprintln!("You should probably adjust it to your needs!");
        Ok(())
    }

    pub fn try_get_location(&self) -> Result<Coords> {
        self.location
            .ok_or_else(|| anyhow!("location not set in the configuration"))
    }
}
