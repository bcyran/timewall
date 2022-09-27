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
    entries: HashSet<String>,
}

impl Cache {
    /// Find user's cache directory and and create or load a cache of given name in it.
    ///
    /// E.g. `Cache::find("wallpapers")` will create 'wallpapers' directory in timewall
    /// directory in user's main cache directory.
    pub fn find(name: &str) -> Self {
        match ProjectDirs::from(APP_QUALIFIER, "", APP_NAME) {
            Some(app_dirs) => Cache::in_dir(app_dirs.cache_dir().join(name)),
            None => panic!("couldn't determine user's home directory"),
        }
    }

    /// Load cache from a given directory. Create it if it doesn't exist.
    fn in_dir<P: AsRef<Path>>(path: P) -> Self {
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
            entries: entry_dirs,
        }
    }

    /// Get path to the dir for a given key. Create the dir if it doesn't exist.
    pub fn entry(&mut self, key: &String) -> PathBuf {
        if self.entries.contains(key) {
            self.get_entry(key)
        } else {
            self.add_entry(key)
        }
    }

    /// Create cache dir for a given key. Panics if the dir already exists.
    fn add_entry(&mut self, key: &str) -> PathBuf {
        let entry_path = self.base_dir.join(key);
        fs::create_dir(&entry_path).expect("couldn't create cache entry directory");
        self.entries.insert(key.to_owned());
        entry_path
    }

    /// Construct path to cache dir for a given key. Does not check whether the dir exists or not!
    fn get_entry(&self, key: &str) -> PathBuf {
        self.base_dir.join(key)
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;
    use predicates::prelude::*;

    use super::*;

    #[test]
    fn test_in_dir_not_exists() {
        let tmp_dir = assert_fs::TempDir::new().unwrap();
        let expected_dir = tmp_dir.child("random_dir");

        Cache::in_dir(&expected_dir);

        expected_dir.assert(predicate::path::is_dir());
    }

    #[test]
    fn test_in_dir_exists() {
        let tmp_dir = assert_fs::TempDir::new().unwrap();
        let expected_entries = HashSet::from([String::from("first"), String::from("other")]);
        for entry in &expected_entries {
            tmp_dir.child(entry).create_dir_all().unwrap();
        }

        let cache = Cache::in_dir(&tmp_dir);

        assert_eq!(cache.entries, expected_entries);
    }

    #[test]
    fn test_entry_not_exists() {
        let tmp_dir = assert_fs::TempDir::new().unwrap();
        let entry_name = String::from("random-entry");
        let expected_dir = tmp_dir.child(&entry_name);

        let mut cache = Cache::in_dir(&tmp_dir);

        assert_eq!(cache.entry(&entry_name), expected_dir.path());
        expected_dir.assert(predicate::path::is_dir());
    }

    #[test]
    fn test_entry_exists() {
        let tmp_dir = assert_fs::TempDir::new().unwrap();
        let entry_name = String::from("some_entry");
        let expected_dir = tmp_dir.child(&entry_name);
        expected_dir.create_dir_all().unwrap();

        let mut cache = Cache::in_dir(&tmp_dir);

        assert_eq!(cache.entry(&entry_name), expected_dir.path());
        expected_dir.assert(predicate::path::is_dir());
    }

    #[test]
    #[should_panic]
    fn test_entry_file_conflict() {
        let tmp_dir = assert_fs::TempDir::new().unwrap();
        let entry_name = String::from("some_entry");
        tmp_dir.child(&entry_name).touch().unwrap();

        Cache::in_dir(&tmp_dir).entry(&entry_name);
    }
}
