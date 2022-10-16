use std::env;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use log::debug;

/// Set wallpaper to the image pointed by a given path. Use custom command if provided.
pub fn set_wallpaper<P: AsRef<Path>>(path: P, custom_command: Option<&Vec<String>>) -> Result<()> {
    let setter = get_setter();
    if let Some(command) = custom_command {
        setter.set_wallpaper_custom_command(path.as_ref(), command)
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

trait WallpaperSetter {
    fn set_wallpaper(&self, path: &Path) -> Result<()>;
    fn set_wallpaper_custom_command(&self, path: &Path, custom_command: &[String]) -> Result<()>;
}

/// Real, actual wallpaper setter.
struct DefaultSetter;
impl WallpaperSetter for DefaultSetter {
    fn set_wallpaper(&self, path: &Path) -> Result<()> {
        let abs_path = path.canonicalize()?;
        wallpaper::set_from_path(abs_path.to_str().unwrap())
            .map_err(|e| anyhow!("could not set wallpaper: {}", e))
    }

    fn set_wallpaper_custom_command(&self, path: &Path, command_str: &[String]) -> Result<()> {
        let path_str = path.to_str().unwrap();
        let expended_command = expand_command(command_str, path_str);
        let mut command = expended_command.iter();

        let mut process_command = Command::new(command.next().unwrap());
        process_command.args(command);
        debug!("running custom command: {process_command:?}");

        let command_status = process_command
            .status()
            .with_context(|| "failed to run custom command")?;

        match command_status.success() {
            true => Ok(()),
            false => Err(anyhow!("custom command process failed")),
        }
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

    fn set_wallpaper_custom_command(&self, path: &Path, command_str: &[String]) -> Result<()> {
        let expanded_command = expand_command(command_str, path.to_str().unwrap());
        println!("Run: {}", expanded_command.join(" "));
        Ok(())
    }
}

/// Replace '%f' in command with file path.
fn expand_command(command_str: &[String], path_str: &str) -> Vec<String> {
    command_str
        .iter()
        .map(|item| item.replace("%f", path_str))
        .collect_vec()
}
