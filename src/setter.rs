use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use log::debug;

pub fn set_wallpaper<P: AsRef<Path>>(path: P, custom_command: Option<&Vec<String>>) -> Result<()> {
    if let Some(command) = custom_command {
        set_wallpaper_custom_command(path, command)
    } else {
        let abs_path = path.as_ref().canonicalize()?;
        wallpaper::set_from_path(abs_path.to_str().unwrap())
            .map_err(|e| anyhow!("could not set wallpaper: {}", e))
    }
}

fn set_wallpaper_custom_command<P: AsRef<Path>>(path: P, command_str: &[String]) -> Result<()> {
    let path_str = path.as_ref().to_str().unwrap();
    let mut command = command_str.iter().map(|item| item.replace("%f", path_str));

    let mut process_command = Command::new(command.next().unwrap());
    process_command.args(command);
    debug!("running custom command: {process_command:?}");

    let command_status = process_command
        .status()
        .with_context(|| format!("failed to run custom command"))?;

    match command_status.success() {
        true => Ok(()),
        false => Err(anyhow!("custom command process failed")),
    }
}
