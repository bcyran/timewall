use std::path::Path;

use anyhow::{anyhow, Result};

pub fn set_wallpaper<P: AsRef<Path>>(path: P) -> Result<()> {
    let abs_path = path.as_ref().canonicalize()?;
    wallpaper::set_from_path(abs_path.to_str().unwrap())
        .or_else(|e| Err(anyhow!("could not set wallpaper: {}", e)))
}
