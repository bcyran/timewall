use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

use crate::constants::{APP_NAME, APP_QUALIFIER};

/// Abstraction over a cache directory. Manges multiple cache subdirectories accessed by
/// a string key.
#[derive(Debug)]
pub struct Cache {
    base_dir: PathBuf,
    pub entries: HashSet<String>,
}

impl Cache {
    /// Find user's cache directory and and create or load a cache of given name in it.
    ///
    /// E.g. `Cache::find("wallpapers")` will create 'wallpapers' directory in timewall
    /// directory in user's main cache directory.
    pub fn find(name: &str) -> Self {
        let cache_dir = if let Result::Ok(path_str) = env::var("TIMEWALL_CACHE_DIR") {
            PathBuf::from(path_str)
        } else {
            match ProjectDirs::from(APP_QUALIFIER, "", APP_NAME) {
                Some(app_dirs) => app_dirs.cache_dir().to_path_buf(),
                None => panic!("couldn't determine user's home directory"),
            }
        };
        Self::in_dir(cache_dir.join(name))
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

        Self {
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

    /// Remove the cache dir for a given key.
    pub fn remove_entry(&mut self, key: &str) {
        let entry_path = self.get_entry(key);
        if entry_path.is_dir() {
            fs::remove_dir_all(entry_path).expect("couldn't remove cache entry directory");
        }
        self.entries.remove(key);
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
        let cache_dir = if let Result::Ok(path_str) = env::var("TIMEWALL_CACHE_DIR") {
            PathBuf::from(path_str)
        } else {
            match ProjectDirs::from(APP_QUALIFIER, "", APP_NAME) {
                Some(app_dirs) => app_dirs.cache_dir().to_path_buf(),
                None => panic!("couldn't determine user's home directory"),
            }
        };
        Self::load(cache_dir.join("last_wall"))
    }

    /// Load instance from given link path.
    fn load<P: AsRef<Path>>(link_path: P) -> Self {
        let link_path = link_path.as_ref();
        let parent_dir = link_path.parent().unwrap();

        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir).expect("couldn't create cache directory");
        }

        Self {
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

    /// Remove the link to the last used wallpaper.
    pub fn clear(&self) {
        if fs::read_link(&self.link_path).is_ok() {
            fs::remove_file(&self.link_path).ok();
        }
    }
}

/// Abstraction over a symlink to the last used PID.
pub struct LastPid {
    pid_path: PathBuf,
}

impl LastPid {
    /// Find user's cache directory and load instance from there.
    pub fn find() -> Self {
        let cache_dir = if let Result::Ok(path_str) = env::var("TIMEWALL_CACHE_DIR") {
            PathBuf::from(path_str)
        } else {
            match ProjectDirs::from(APP_QUALIFIER, "", APP_NAME) {
                Some(app_dirs) => app_dirs.cache_dir().to_path_buf(),
                None => panic!("couldn't determine user's home directory"),
            }
        };
        Self::load(cache_dir.join("last_pid"))
    }

    /// Load instance from given path.
    fn load<P: AsRef<Path>>(pid_path: P) -> Self {
        let pid_path = pid_path.as_ref();
        let parent_dir = pid_path.parent().unwrap();

        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir).expect("couldn't create cache directory");
        }

        Self {
            pid_path: pid_path.to_path_buf(),
        }
    }

    /// Set last pid.
    pub fn set(&self, pid: u32) {
        fs::remove_file(&self.pid_path).ok();
        fs::write(&self.pid_path, pid.to_string()).unwrap();
    }

    /// Get path to the last used pid, if it exists.
    pub fn get(&self) -> Option<u32> {
        if self.pid_path.exists() {
            Some(
                fs::read_to_string(&self.pid_path)
                    .unwrap()
                    .parse::<u32>()
                    .unwrap(),
            )
        } else {
            None
        }
    }

    /// Remove the link to the last used pid.
    pub fn clear(&self) {
        if fs::read_link(&self.pid_path).is_ok() {
            fs::remove_file(&self.pid_path).ok();
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

        cache.remove_entry(&entry_name);
        expected_dir.assert(predicate::path::missing());
    }

    #[rstest]
    fn test_cache_entry_remove_exists(tmp_dir: TempDir) {
        let entry_name = String::from("some_entry");
        let expected_dir = tmp_dir.child(&entry_name);
        expected_dir.create_dir_all().unwrap();

        let mut cache = Cache::in_dir(&tmp_dir);

        expected_dir.assert(predicate::path::is_dir());
        cache.remove_entry(&entry_name);
        expected_dir.assert(predicate::path::missing());
    }

    #[rstest]
    fn test_cache_entry_remove_not_exists(tmp_dir: TempDir) {
        let entry_name = String::from("some_entry");
        let expected_dir = tmp_dir.child(&entry_name);

        let mut cache = Cache::in_dir(&tmp_dir);

        expected_dir.assert(predicate::path::missing());
        cache.remove_entry(&entry_name);
        expected_dir.assert(predicate::path::missing());
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

    #[rstest]
    fn test_last_wallpaper_save_clear(tmp_dir: TempDir) {
        let target_path = tmp_dir.child("target.heic");
        let link_path = tmp_dir.child("test_link");
        target_path.touch().unwrap();

        let last_wall = LastWallpaper::load(&link_path);

        last_wall.save(&target_path);
        link_path.assert(predicate::path::exists());

        last_wall.clear();
        link_path.assert(predicate::path::missing());
    }
}
