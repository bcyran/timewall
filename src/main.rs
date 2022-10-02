use std::path::Path;

use anyhow::{Ok, Result};
use chrono::prelude::*;
use clap::Parser;
use geo::Coords;
use loader::WallpaperLoader;
use properties::WallpaperProperties;

#[macro_use]
mod macros;
mod cache;
mod cli;
mod geo;
mod heic;
mod loader;
mod metadata;
mod properties;
mod selection;
mod setter;
mod wallpaper;

use metadata::ImageInfo;

use crate::selection::select_image_h24;
use crate::selection::select_image_solar;
use crate::setter::set_wallpaper;

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

    let coords = Coords {
        lat: 50.16,
        lon: 19.10,
    };
    let now = Local::now();
    let current_index = match wallpaper.properties {
        WallpaperProperties::H24(props) => select_image_h24(&props.time_info, &now.time()),
        WallpaperProperties::Solar(props) => select_image_solar(&props.solar_info, &now, &coords),
    }?;
    let current_image_path = wallpaper.images.get(current_index).unwrap();

    println!("image index: {}", current_index);

    set_wallpaper(current_image_path)?;

    Ok(())
}
