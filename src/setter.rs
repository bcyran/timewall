use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub fn set_wallpaper<P: AsRef<Path>>(path: P) -> Result<()> {
    let abs_path = path.as_ref().canonicalize()?;
    Command::new("feh")
        .arg("--bg-fill")
        .arg(abs_path.as_os_str())
        .status()
        .with_context(|| format!("failed to set wallpaper"))?;
    Ok(())
}
