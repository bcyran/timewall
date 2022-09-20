use anyhow::{Ok, Result};
use cache::Cache;
use clap::Parser;
use std::path::Path;

mod cache;
mod cli;
mod heic;
mod helpers;
mod metadata;
mod properties;
mod wallpaper;

use metadata::ImageInfo;

fn main() -> Result<()> {
    env_logger::init();

    let args = cli::Args::parse();

    match args.action {
        cli::Action::Info { file } => {
            println!("{}", ImageInfo::from_image(file)?);
            Ok(())
        }
        cli::Action::Unpack { file, output } => wallpaper::unpack_heic(file, output),
        cli::Action::Set { file } => set(file),
    }
}

pub fn set<P: AsRef<Path>>(file: P) -> Result<()> {
    let mut cache = Cache::find()?;
    println!("{cache:?}");
    let cache_dir = cache.entry_dir(file)?;
    println!("{}", cache_dir.display());

    Ok(())
}
