use anyhow::{Context, Result};
use std::path::Path;

use libheif_rs::{HeifContext, SecurityLimits};

pub fn from_file(path: impl AsRef<Path>) -> Result<HeifContext<'static>> {
    let mut heif_context = HeifContext::new().context("failed to create HEIF context")?;

    // The default limits are too low for some big wallpaper files.
    let mut security_limits = SecurityLimits::new();
    security_limits.set_max_items(6000);
    security_limits.set_max_children_per_box(500);
    heif_context
        .set_security_limits(&security_limits)
        .context("failed to set HEIF context security limits")?;

    heif_context
        .read_file(path.as_ref().to_str().unwrap())
        .context("failed to read HEIF context from file")?;

    Ok(heif_context)
}
