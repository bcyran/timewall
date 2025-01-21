use std::{
    env, fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

use crate::constants::{APP_NAME, APP_QUALIFIER};

/// Abstraction over a pidfile containing PID of the last ran wallpaper setter process.
pub struct SetterPidFile {
    pidfile_path: PathBuf,
}

impl SetterPidFile {
    /// Find user's runtime directory and load instance from there.
    pub fn find() -> Self {
        let runtime_dir = if let Result::Ok(path_str) = env::var("TIMEWALL_RUNTIME_DIR") {
            PathBuf::from(path_str)
        } else {
            match ProjectDirs::from(APP_QUALIFIER, "", APP_NAME) {
                Some(app_dirs) => app_dirs
                    .runtime_dir()
                    .map_or_else(|| env::temp_dir().join(APP_NAME), Path::to_path_buf),
                None => panic!("couldn't determine user's home directory"),
            }
        };
        Self::load(runtime_dir.join("last_setter.pid"))
    }

    /// Load instance from given path.
    fn load<P: AsRef<Path>>(pid_path: P) -> Self {
        let pid_path = pid_path.as_ref();
        let parent_dir = pid_path.parent().unwrap();

        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir).expect("couldn't create runtime directory");
        }

        Self {
            pidfile_path: pid_path.to_path_buf(),
        }
    }

    /// Save the PID.
    pub fn save(&self, pid: u32) {
        fs::write(&self.pidfile_path, pid.to_string()).expect("couldn't write setter pidfile");
    }

    /// Read the PID value if it exists.
    pub fn read(&self) -> Option<u32> {
        if self.pidfile_path.exists() {
            Some(
                fs::read_to_string(&self.pidfile_path)
                    .unwrap()
                    .parse::<u32>()
                    .unwrap(),
            )
        } else {
            None
        }
    }

    /// Remove the pidfile.
    pub fn clear(&self) {
        fs::remove_file(&self.pidfile_path).expect("cound't remove setter pidfile");
    }
}
