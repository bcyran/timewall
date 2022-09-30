use anyhow::{Ok, Result};
use clap::Parser;
use loader::WallpaperLoader;
use std::path::Path;

mod cache;
mod cli;
mod heic;
mod loader;
mod metadata;
mod properties;
mod wallpaper;
mod selection;

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

pub fn set<P: AsRef<Path>>(path: P) -> Result<()> {
    let mut loader = WallpaperLoader::new();
    println!("{loader:?}");
    let wallpaper = loader.load(path);
    println!("{wallpaper:?}");

    Ok(())
}
