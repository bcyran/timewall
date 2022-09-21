use directories::ProjectDirs;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

const APP_QUALIFIER: &str = "dev.cyran";
const APP_NAME: &str = "timewall";

/// Abstraction over a cache directory. Manges multiple cache subdirectories accessed by
/// a string key.
#[derive(Debug)]
pub struct Cache {
    base_dir: PathBuf,
    entry_dirs: HashSet<String>,
}

impl Cache {
    /// Find user's cache directory and and create or load a cache of given name in it.
    ///
    /// E.g. `Cache::find("wallpapers")` will create 'wallpapers' directory in timewall
    /// directory in user's main cache directory.
    pub fn find(name: &str) -> Self {
        match ProjectDirs::from(APP_QUALIFIER, "", APP_NAME) {
            Some(app_dirs) => Cache::ensure(app_dirs.cache_dir().join(name)),
            None => panic!("couldn't determine user's home directory"),
        }
    }

    /// Load cache from a given directory. Create it if it doesn't exist.
    fn ensure<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();

        if !path.exists() {
            fs::create_dir_all(path).expect("couldn't create cache directory");
        }
        let entry_dirs = path
            .read_dir()
            .unwrap()
            .flatten()
            .filter(|e| e.file_type().unwrap().is_dir())
            .flat_map(|e| e.file_name().into_string())
            .collect();

        Cache {
            base_dir: path.to_path_buf(),
            entry_dirs,
        }
    }

    /// Get path to the dir for a given key. Create the dir if it doesn't exist.
    pub fn entry_dir(&mut self, key: &String) -> PathBuf {
        if self.entry_dirs.contains(key) {
            self.get_entry_dir(key)
        } else {
            self.add_entry_dir(key)
        }
    }

    /// Create cache dir for a given key. Panics if the dir already exists.
    fn add_entry_dir(&mut self, key: &str) -> PathBuf {
        let entry_path = self.base_dir.join(key);
        fs::create_dir(&entry_path).expect("couldn't create cache entry directory");
        self.entry_dirs.insert(key.to_owned());
        entry_path
    }

    /// Construct path to cache dir for a given key. Does not check whether the dir exists or not!
    fn get_entry_dir(&self, key: &str) -> PathBuf {
        self.base_dir.join(key)
    }
}
