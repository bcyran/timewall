use std::path::Path;
use std::thread;
use std::time::Duration;

use anyhow::{bail, Context};
use anyhow::{Ok, Result};
use chrono::prelude::*;
use clap::Parser;
use cli::Appearance;
use loader::WallpaperLoader;
use log::debug;
use properties::Properties;

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
use schedule::get_image_index_order_appearance;

use crate::config::Config;
use crate::constants::{PREVIEW_UPDATE_INTERVAL_MILLIS, UPDATE_INTERVAL_MINUTES};
use crate::schedule::{
    current_image_index_h24, current_image_index_solar, get_image_index_order_h24,
    get_image_index_order_solar,
};
use crate::setter::set_wallpaper;
use crate::wallpaper::Wallpaper;
use crate::{cache::LastWallpaper, schedule::current_image_index_appearance};

fn main() -> Result<()> {
    env_logger::init();

    let args = cli::Args::parse();

    match args.action {
        cli::Action::Info { file } => info(file),
        cli::Action::Preview { file } => preview(file),
        cli::Action::Unpack { file, output } => wallpaper::unpack_heic(file, output),
        cli::Action::Set {
            file,
            daemon,
            appearance,
        } => set(file, daemon, appearance),
    }
}

pub fn info<P: AsRef<Path>>(path: P) -> Result<()> {
    print!("{}", ImageInfo::from_image(path)?);
    Ok(())
}

pub fn set<P: AsRef<Path>>(
    path: Option<P>,
    daemon: bool,
    user_appearance: Option<Appearance>,
) -> Result<()> {
    if daemon && user_appearance.is_some() {
        bail!("appearance can't be used in daemon mode!")
    }

    let config = Config::find()?;
    let last_wallpaper = LastWallpaper::find();

    let wall_path = if let Some(given_path) = path {
        last_wallpaper.save(&given_path);
        given_path.as_ref().to_path_buf()
    } else if let Some(last_path) = last_wallpaper.get() {
        debug!("last used wallpaper at {}", last_path.display());
        last_path
    } else {
        bail!("no image to set given");
    };

    let wallpaper = WallpaperLoader::new().load(&wall_path);

    loop {
        let current_image_index = current_image_index(&wallpaper, &config, user_appearance)?;
        let current_image_path = wallpaper
            .images
            .get(current_image_index)
            .with_context(|| "missing image specified by metadata")?;

        debug!("setting wallpaper to {}", current_image_path.display());
        set_wallpaper(current_image_path, config.setter_command())?;

        if !daemon {
            break;
        }

        debug!("sleeping for {UPDATE_INTERVAL_MINUTES} minutes");
        thread::sleep(Duration::from_secs(UPDATE_INTERVAL_MINUTES * 60));
    }

    Ok(())
}

fn current_image_index(
    wallpaper: &Wallpaper,
    config: &Config,
    user_appearance: Option<Appearance>,
) -> Result<usize> {
    let now = Local::now();
    match wallpaper.properties {
        ref any_properties if user_appearance.is_some() => match any_properties.appearance() {
            Some(appearance_props) => {
                current_image_index_appearance(appearance_props, user_appearance)
            }
            None => bail!("wallpaper missing appearance metadata"),
        },
        Properties::Appearance(ref appearance_props) => {
            current_image_index_appearance(appearance_props, user_appearance)
        }
        Properties::H24(ref props) => current_image_index_h24(&props.time_info, &now.time()),
        Properties::Solar(ref props) => {
            current_image_index_solar(&props.solar_info, &now, &config.location)
        }
    }
}

pub fn preview<P: AsRef<Path>>(path: P) -> Result<()> {
    let config = Config::find()?;
    let wallpaper = WallpaperLoader::new().load(&path);
    let image_order = match wallpaper.properties {
        Properties::H24(ref props) => get_image_index_order_h24(&props.time_info),
        Properties::Solar(ref props) => get_image_index_order_solar(&props.solar_info),
        Properties::Appearance(ref props) => get_image_index_order_appearance(&props),
    };

    for image_index in image_order.iter().cycle() {
        let image_path = wallpaper.images.get(*image_index).unwrap();
        set_wallpaper(image_path, config.setter_command())?;
        thread::sleep(Duration::from_millis(PREVIEW_UPDATE_INTERVAL_MILLIS));
    }

    Ok(())
}
