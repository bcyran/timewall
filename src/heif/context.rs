use anyhow::{Context, Result};
use std::path::Path;

use libheif_rs::{HeifContext, SecurityLimits};

pub fn from_file(path: impl AsRef<Path>) -> Result<HeifContext<'static>> {
    let mut heif_context = HeifContext::new().context("failed to create HEIF context")?;

    // The default limit of 1000 items is too low for some wallpapers.
    let mut security_limits = SecurityLimits::new();
    security_limits.set_max_items(2000);
    heif_context
        .set_security_limits(&security_limits)
        .context("failed to set HEIF context security limits")?;

    heif_context
        .read_file(path.as_ref().to_str().unwrap())
        .context("failed to read HEIF context from file")?;

    Ok(heif_context)
}
