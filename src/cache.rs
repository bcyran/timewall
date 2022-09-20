use crate::helpers::hash_file;
use anyhow::{anyhow, Context, Result};
use directories::ProjectDirs;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

const APP_QUALIFIER: &str = "dev.cyran";
const APP_NAME: &str = "timewall";

#[derive(Debug)]
pub struct Cache {
    base_dir: PathBuf,
    entry_dirs: HashSet<String>,
}

impl Cache {
    /// Find user's cache directory and load or create cache in it.
    pub fn find() -> Result<Self> {
        match ProjectDirs::from(APP_QUALIFIER, "", APP_NAME) {
            Some(app_dirs) => Cache::load(app_dirs.cache_dir()),
            None => Err(anyhow!("couldn't determine user's home directory")),
        }
    }

    /// Load cache from a given base directory. Create it if it doesn't exist.
    fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            fs::create_dir_all(path).with_context(|| format!("couldn't create cache directory"))?;
        }
        let entry_dirs = fs::read_dir(path)?
            .flatten()
            .filter(|e| e.file_type().unwrap().is_dir())
            .flat_map(|e| e.file_name().into_string())
            .collect();
        Ok(Cache {
            base_dir: path.to_path_buf(),
            entry_dirs,
        })
    }

    /// Get path to entry dir. Create it if it doesn't exist.
    pub fn entry_dir<P: AsRef<Path>>(&mut self, file: P) -> Result<PathBuf> {
        let hash = hash_file(file)?;
        if self.entry_dirs.contains(&hash) {
            Ok(self.get_entry_dir(&hash))
        } else {
            self.add_entry_dir(&hash)
        }
    }

    /// Create cache dir for a given file.
    fn add_entry_dir(&mut self, hash: &str) -> Result<PathBuf> {
        let entry_path = self.base_dir.join(&hash);
        fs::create_dir(&entry_path)?;
        self.entry_dirs.insert(hash.to_owned());
        Ok(entry_path)
    }

    /// Get path to cache dir for a given file. Does not check whether the path exists or not!
    fn get_entry_dir(&self, hash: &str) -> PathBuf {
        self.base_dir.join(&hash)
    }
}
