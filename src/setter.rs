use std::path::Path;
use std::process::Command;
use std::time::Duration;
use std::{env, thread};

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use libc::{kill, SIGKILL};
use log::debug;
use wallpape_rs as wallpaper;

use crate::config::{Config, Setter};
use crate::pidfile::SetterPidFile;

/// Set wallpaper to the image pointed by a given path. Use custom command if provided.
pub fn set_wallpaper<P: AsRef<Path>>(path: P, maybe_setter_config: Option<&Setter>) -> Result<()> {
    let setter = get_setter();
    if let Some(setter_config) = maybe_setter_config {
        setter.set_wallpaper_custom_command(path.as_ref(), setter_config)
    } else {
        setter.set_wallpaper(path.as_ref())
    }
}

fn get_setter() -> Box<dyn WallpaperSetter> {
    match env::var("TIMEWALL_DRY_RUN") {
        Err(_) => Box::new(DefaultSetter {}),
        Ok(_) => Box::new(DryRunSetter {}),
    }
}

pub fn cleanup() {
    let setter = get_setter();
    setter.cleanup();
}

trait WallpaperSetter {
    fn set_wallpaper(&self, path: &Path) -> Result<()>;
    fn set_wallpaper_custom_command(&self, path: &Path, setter_config: &Setter) -> Result<()>;
    fn cleanup(&self);
}

/// Real, actual wallpaper setter.
struct DefaultSetter {}
impl WallpaperSetter for DefaultSetter {
    fn set_wallpaper(&self, path: &Path) -> Result<()> {
        let abs_path = path.canonicalize()?;

        wallpaper::set_from_path(abs_path.to_str().unwrap()).map_err(|err| {
            anyhow!(format!(
                concat!(
                    "Automated wallpaper setting failed: {}\n",
                    "This is most likely caused by an unsupported DE or WM.\n",
                    "Please configure a custom wallpaper setting command in the config file.\n",
                    "You can find it at {}"
                ),
                err,
                Config::find_path().unwrap().display()
            ))
        })
    }

    fn set_wallpaper_custom_command(&self, path: &Path, setter_config: &Setter) -> Result<()> {
        let path_str = path.to_str().unwrap();
        let expended_command = expand_command(&setter_config.command, path_str);
        let mut command = expended_command.iter();

        let mut process_command = Command::new(command.next().unwrap());
        process_command.args(command);
        debug!("running custom command: {process_command:?}");

        let wallpaper_process = process_command
            .spawn()
            .with_context(|| "failed to run custom command")?;

        thread::sleep(Duration::from_millis(setter_config.overlap));

        let pidfile = SetterPidFile::find();
        if let Some(last_pid) = pidfile.read() {
            debug!("terminating previous process with PID: {:?}", last_pid);
            unsafe {
                #[allow(clippy::cast_possible_wrap)]
                if kill(last_pid as i32, SIGKILL) != 0 {
                    eprintln!("failed to kill process: {last_pid}");
                } else {
                    println!("process killed: {last_pid}");
                }
            }
        }

        pidfile.save(wallpaper_process.id());

        Ok(())
    }

    fn cleanup(&self) {
        let pidfile = SetterPidFile::find();
        if let Some(last_pid) = pidfile.read() {
            debug!("terminating previous process with PID: {:?}", last_pid);
            unsafe {
                #[allow(clippy::cast_possible_wrap)]
                if kill(last_pid as i32, SIGKILL) != 0 {
                    eprintln!("failed to kill process: {last_pid}");
                } else {
                    println!("process killed: {last_pid}");
                }
            }
        }
        pidfile.clear();
    }
}

/// Dry run setter, mainly for use in tests.
/// Instead of actually setting the wallpaper, prints out the actions.
struct DryRunSetter;
impl WallpaperSetter for DryRunSetter {
    fn set_wallpaper(&self, path: &Path) -> Result<()> {
        println!("Set: {}", path.display());
        Ok(())
    }

    fn set_wallpaper_custom_command(&self, path: &Path, setter_config: &Setter) -> Result<()> {
        let expanded_command = expand_command(&setter_config.command, path.to_str().unwrap());
        println!("Run: {}", expanded_command.join(" "));
        Ok(())
    }

    fn cleanup(&self) {}
}

/// Replace '%f' in command with file path.
fn expand_command(command_str: &[String], path_str: &str) -> Vec<String> {
    command_str
        .iter()
        .map(|item| item.replace("%f", path_str))
        .collect_vec()
}
