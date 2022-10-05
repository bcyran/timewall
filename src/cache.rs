use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

use crate::constants::{APP_NAME, APP_QUALIFIER};

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

/// Abstraction over a symlink to the last used wallpaper.
pub struct LastWallpaper {
    link_path: PathBuf,
}

impl LastWallpaper {
    /// Find user's cache directory and load instance from there.
    pub fn find() -> Self {
        match ProjectDirs::from(APP_QUALIFIER, "", APP_NAME) {
            Some(app_dirs) => LastWallpaper::load(app_dirs.cache_dir().join("last_wall")),
            None => panic!("couldn't determine user's home directory"),
        }
    }

    /// Load instance from given link path.
    fn load<P: AsRef<Path>>(link_path: P) -> Self {
        let link_path = link_path.as_ref();
        let parent_dir = link_path.parent().unwrap();

        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir).expect("couldn't create cache directory");
        }

        LastWallpaper {
            link_path: link_path.to_path_buf(),
        }
    }

    /// Save path to the last wallpaper.
    /// This may silently fail. We don't care because it's not a critical functionality.
    pub fn save<P: AsRef<Path>>(&self, path: P) {
        if fs::read_link(&self.link_path).is_ok() {
            fs::remove_file(&self.link_path).ok();
        }
        std::os::unix::fs::symlink(path.as_ref().canonicalize().unwrap(), &self.link_path).ok();
    }

    /// Get path to the last used wallpaper, if it exists.
    pub fn get(&self) -> Option<PathBuf> {
        if self.link_path.exists() {
            Some(fs::read_link(&self.link_path).unwrap())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use predicates::prelude::*;
    use rstest::*;

    use super::*;

    #[fixture]
    fn tmp_dir() -> TempDir {
        assert_fs::TempDir::new().unwrap()
    }

    #[rstest]
    fn test_cache_in_dir_not_exists(tmp_dir: TempDir) {
        let expected_dir = tmp_dir.child("random_dir");

        Cache::in_dir(&expected_dir);

        expected_dir.assert(predicate::path::is_dir());
    }

    #[rstest]
    fn test_cache_in_dir_exists(tmp_dir: TempDir) {
        let expected_entries = HashSet::from([String::from("first"), String::from("other")]);
        for entry in &expected_entries {
            tmp_dir.child(entry).create_dir_all().unwrap();
        }

        let cache = Cache::in_dir(&tmp_dir);

        assert_eq!(cache.entries, expected_entries);
    }

    #[rstest]
    fn test_cache_entry_not_exists(tmp_dir: TempDir) {
        let entry_name = String::from("random-entry");
        let expected_dir = tmp_dir.child(&entry_name);

        let mut cache = Cache::in_dir(&tmp_dir);

        assert_eq!(cache.entry(&entry_name), expected_dir.path());
        expected_dir.assert(predicate::path::is_dir());
    }

    #[rstest]
    fn test_cache_entry_exists(tmp_dir: TempDir) {
        let entry_name = String::from("some_entry");
        let expected_dir = tmp_dir.child(&entry_name);
        expected_dir.create_dir_all().unwrap();

        let mut cache = Cache::in_dir(&tmp_dir);

        assert_eq!(cache.entry(&entry_name), expected_dir.path());
        expected_dir.assert(predicate::path::is_dir());
    }

    #[rstest]
    #[should_panic]
    fn test_cache_entry_file_conflict(tmp_dir: TempDir) {
        let entry_name = String::from("some_entry");
        tmp_dir.child(&entry_name).touch().unwrap();

        Cache::in_dir(&tmp_dir).entry(&entry_name);
    }

    #[rstest]
    fn test_last_wallpaper_load_not_exists(tmp_dir: TempDir) {
        let fake_cache_dir = tmp_dir.child("cache_dir");
        let link_path = fake_cache_dir.child("test_link");

        LastWallpaper::load(&link_path);

        fake_cache_dir.assert(predicate::path::exists());
    }

    #[rstest]
    fn test_last_wallpaper_save_get(tmp_dir: TempDir) {
        let target_path_1 = tmp_dir.child("target.heic");
        let target_path_2 = tmp_dir.child("other_target.heic");
        let link_path = tmp_dir.child("test_link");
        target_path_1.touch().unwrap();
        target_path_2.touch().unwrap();

        let last_wall = LastWallpaper::load(&link_path);
        link_path.assert(predicate::path::missing());
        assert_eq!(last_wall.get(), None);

        last_wall.save(&target_path_1);
        assert_eq!(last_wall.get(), Some(target_path_1.to_path_buf()));

        fs::remove_file(target_path_1).unwrap();
        last_wall.save(&target_path_2);
        assert_eq!(last_wall.get(), Some(target_path_2.to_path_buf()));
    }
}
