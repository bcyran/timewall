use std::path::Path;
use std::thread;
use std::time::Duration;

use anyhow::Context;
use anyhow::{anyhow, Ok, Result};
use chrono::prelude::*;
use clap::Parser;
use loader::WallpaperLoader;
use log::debug;
use properties::WallpaperProperties;

#[macro_use]
mod macros;
mod cache;
mod cli;
mod config;
mod constants;
mod geo;
mod heic;
mod info;
mod loader;
mod metadata;
mod properties;
mod schedule;
mod setter;
mod time;
mod wallpaper;

use info::ImageInfo;

use crate::cache::LastWallpaper;
use crate::config::Config;
use crate::constants::{PREVIEW_UPDATE_INTERVAL_MILLIS, UPDATE_INTERVAL_MINUTES};
use crate::schedule::{
    current_image_index_h24, current_image_index_solar, get_image_index_order_h24,
    get_image_index_order_solar,
};
use crate::setter::set_wallpaper;

fn main() -> Result<()> {
    env_logger::init();

    let args = cli::Args::parse();

    match args.action {
        cli::Action::Info { file } => info(file),
        cli::Action::Preview { file } => preview(file),
        cli::Action::Unpack { file, output } => wallpaper::unpack_heic(file, output),
        cli::Action::Set { file, daemon } => set(file, daemon),
    }
}

pub fn info<P: AsRef<Path>>(path: P) -> Result<()> {
    print!("{}", ImageInfo::from_image(path)?);
    Ok(())
}

pub fn set<P: AsRef<Path>>(path: Option<P>, daemon: bool) -> Result<()> {
    let config = Config::find()?;
    let last_wallpaper = LastWallpaper::find();

    let wall_path = if let Some(given_path) = path {
        last_wallpaper.save(&given_path);
        given_path.as_ref().to_path_buf()
    } else if let Some(last_path) = last_wallpaper.get() {
        debug!("last used wallpaper at {}", last_path.display());
        last_path
    } else {
        return Err(anyhow!("no image to set given"));
    };

    let wallpaper = WallpaperLoader::new().load(&wall_path);

    loop {
        let now = Local::now();
        let current_image_index = match wallpaper.properties {
            WallpaperProperties::H24(ref props) => {
                current_image_index_h24(&props.time_info, &now.time())
            }
            WallpaperProperties::Solar(ref props) => {
                current_image_index_solar(&props.solar_info, &now, &config.coords)
            }
        }
        .with_context(|| format!("could not determine image to set"))?;

        let current_image_path = wallpaper
            .images
            .get(current_image_index)
            .with_context(|| format!("missing image specified by metadata"))?;

        debug!("setting wallpaper to {}", current_image_path.display());
        set_wallpaper(current_image_path)
            .with_context(|| format!("could not set the wallpaper"))?;

        if !daemon {
            break;
        }

        debug!("sleeping for {UPDATE_INTERVAL_MINUTES} minutes");
        thread::sleep(Duration::from_secs(UPDATE_INTERVAL_MINUTES * 60));
    }

    Ok(())
}

pub fn preview<P: AsRef<Path>>(path: P) -> Result<()> {
    let wallpaper = WallpaperLoader::new().load(&path);
    let image_order = match wallpaper.properties {
        WallpaperProperties::H24(ref props) => get_image_index_order_h24(&props.time_info),
        WallpaperProperties::Solar(ref props) => get_image_index_order_solar(&props.solar_info),
    };

    for image_index in image_order.iter().cycle() {
        let image_path = wallpaper.images.get(*image_index).unwrap();
        set_wallpaper(image_path).with_context(|| format!("could not set the wallpaper"))?;
        thread::sleep(Duration::from_millis(PREVIEW_UPDATE_INTERVAL_MILLIS));
    }

    Ok(())
}
