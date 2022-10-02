use std::path::Path;

use anyhow::Context;
use anyhow::{anyhow, Ok, Result};
use chrono::prelude::*;
use clap::Parser;
use loader::WallpaperLoader;
use properties::WallpaperProperties;

#[macro_use]
mod macros;
mod cache;
mod cli;
mod config;
mod constants;
mod geo;
mod heic;
mod loader;
mod metadata;
mod properties;
mod selection;
mod setter;
mod wallpaper;

use metadata::ImageInfo;

use crate::cache::LastWallpaper;
use crate::config::Config;
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

pub fn set<P: AsRef<Path>>(path: Option<P>) -> Result<()> {
    let config = Config::find()?;
    let last_wallpaper = LastWallpaper::find();

    let wall_path = if let Some(given_path) = path {
        last_wallpaper.save(&given_path);
        given_path.as_ref().to_path_buf()
    } else if let Some(last_path) = last_wallpaper.get() {
        last_path
    } else {
        return Err(anyhow!("no image to set given"));
    };

    let wallpaper = WallpaperLoader::new().load(&wall_path);

    let now = Local::now();
    let current_image_index = match wallpaper.properties {
        WallpaperProperties::H24(props) => select_image_h24(&props.time_info, &now.time()),
        WallpaperProperties::Solar(props) => {
            select_image_solar(&props.solar_info, &now, &config.coords)
        }
    }
    .with_context(|| format!("could not determine image to set"))?;

    let current_image_path = wallpaper
        .images
        .get(current_image_index)
        .with_context(|| format!("missing image specified by metadata"))?;

    set_wallpaper(current_image_path).with_context(|| format!("could not set the wallpaper"))?;

    Ok(())
}
