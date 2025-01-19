use std::path::PathBuf;
use std::time::Duration;
use std::{env, path::Path};
use std::{str::FromStr, thread};

use anyhow::Result;
use anyhow::{anyhow, bail, Context};
use chrono::prelude::*;
use log::debug;

use crate::cli::Appearance;
use crate::config::Config;
use crate::geo::Coords;
use crate::geoclue;
use crate::heif;
use crate::info::ImageInfo;
use crate::loader::WallpaperLoader;
use crate::schedule::{
    current_image_index_h24, current_image_index_solar, get_image_index_order_appearance,
    get_image_index_order_h24, get_image_index_order_solar,
};
use crate::setter::set_wallpaper;
use crate::wallpaper::{self, properties::Properties, Wallpaper};
use crate::{cache::LastWallpaper, schedule::current_image_index_appearance};

const GEOCLUE_TIMEOUT: Duration = Duration::from_secs(1);

pub fn info<P: AsRef<Path>>(path: P) -> Result<()> {
    validate_wallpaper_file(&path)?;
    print!("{}", ImageInfo::from_image(&path)?);
    Ok(())
}

pub fn unpack<IP: AsRef<Path>, OP: AsRef<Path>>(source: IP, destination: OP) -> Result<()> {
    validate_wallpaper_file(&source)?;
    wallpaper::unpack(source, destination)
}

pub fn set<P: AsRef<Path>>(
    path: Option<&P>,
    daemon: bool,
    user_appearance: Option<Appearance>,
) -> Result<()> {
    if daemon && user_appearance.is_some() {
        bail!("appearance can't be used in daemon mode!")
    }

    let config = Config::find()?;

    let mut previous_image_index: Option<usize> = None;
    loop {
        let wall_path = get_effective_wall_path(path.as_ref())?;
        let wallpaper = WallpaperLoader::new().load(&wall_path);

        let current_image_index = current_image_index(&wallpaper, &config, user_appearance)?;
        if previous_image_index == Some(current_image_index) {
            debug!("current image is the same as the previous one, skipping update");
        } else {
            previous_image_index.replace(current_image_index);

            let current_image_path = wallpaper
                .images
                .get(current_image_index)
                .with_context(|| "missing image specified by metadata")?;

            debug!("setting wallpaper to {}", current_image_path.display());
            set_wallpaper(current_image_path, config.setter_command())?;

            if !daemon {
                eprintln!("Wallpaper set!");
                break;
            }
        }

        let update_interval_seconds = config.update_interval_seconds();
        debug!("sleeping for {update_interval_seconds} seconds");
        thread::sleep(Duration::from_secs(update_interval_seconds));
    }

    Ok(())
}

fn get_effective_wall_path<P: AsRef<Path>>(given_path: Option<P>) -> Result<PathBuf> {
    let last_wallpaper = LastWallpaper::find();

    if let Some(path) = given_path {
        validate_wallpaper_file(&path)?;
        last_wallpaper.save(&path);
        Ok(path.as_ref().to_path_buf())
    } else if let Some(last_path) = last_wallpaper.get() {
        debug!("last used wallpaper at {}", last_path.display());
        Ok(last_path)
    } else {
        Err(anyhow!("no image to set given"))
    }
}

pub fn preview<P: AsRef<Path>>(path: P, delay: u64, repeat: bool) -> Result<()> {
    let config = Config::find()?;
    validate_wallpaper_file(&path)?;
    let wallpaper = WallpaperLoader::new().load(&path);
    let image_order = match wallpaper.properties {
        Properties::H24(ref props) => get_image_index_order_h24(&props.time_info),
        Properties::Solar(ref props) => get_image_index_order_solar(&props.solar_info),
        Properties::Appearance(ref props) => get_image_index_order_appearance(props),
    };

    loop {
        for image_index in &image_order {
            let image_path = wallpaper.images.get(*image_index).unwrap();
            set_wallpaper(image_path, config.setter_command())?;
            thread::sleep(Duration::from_millis(delay));
        }

        if !repeat {
            break;
        }
    }

    Ok(())
}

pub fn clear(all: bool) {
    let mut loader = WallpaperLoader::new();
    let last_wallpaper = (!all).then(|| LastWallpaper::find().get()).flatten();
    loader.clear_cache(last_wallpaper);
    if all {
        LastWallpaper::find().clear();
    }
}

fn validate_wallpaper_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        bail!("file '{}' is not accessible", path.display());
    }
    if !path.is_file() {
        bail!("'{}' is not a file", path.display());
    }
    heif::validate_file(path)
}

fn get_now_time() -> DateTime<Local> {
    match env::var("TIMEWALL_OVERRIDE_TIME") {
        Err(_) => Local::now(),
        Result::Ok(time_str) => DateTime::from_str(&time_str).unwrap(),
    }
}

fn current_image_index(
    wallpaper: &Wallpaper,
    config: &Config,
    user_appearance: Option<Appearance>,
) -> Result<usize> {
    let now = get_now_time();
    match wallpaper.properties {
        ref any_properties if user_appearance.is_some() => match any_properties.appearance() {
            Some(appearance_props) => Ok(current_image_index_appearance(
                appearance_props,
                user_appearance,
            )),
            None => bail!("wallpaper missing appearance metadata"),
        },
        Properties::Appearance(ref appearance_props) => Ok(current_image_index_appearance(
            appearance_props,
            user_appearance,
        )),
        Properties::H24(ref props) => current_image_index_h24(&props.time_info, now.time()),
        Properties::Solar(ref props) => {
            current_image_index_solar(&props.solar_info, &now, &try_get_location(config)?)
        }
    }
}

fn try_get_location(config: &Config) -> Result<Coords> {
    let maybe_location = match (config.geoclue_enabled(), config.geoclue_preferred()) {
        (true, true) => {
            geoclue::get_location(GEOCLUE_TIMEOUT).or_else(|_| config.try_get_location())
        }
        (true, false) => config
            .try_get_location()
            .or_else(|_| geoclue::get_location(GEOCLUE_TIMEOUT)),
        (false, _) => config.try_get_location(),
    };

    maybe_location.with_context(|| {
        format!(
            concat!(
                "Using wallpapers with solar schedule requires your approximate location information. ",
                "Please enable geoclue2 or provide the location manually in the configuration file at {}."
            ),
            Config::find_path().unwrap().display()
        )
    })
}
