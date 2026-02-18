use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::{env, path::Path};

use anyhow::Result;
use anyhow::{anyhow, bail, Context};
use chrono::prelude::*;
use log::debug;

use crate::appearance::{get_system_appearance, Appearance};
use crate::cache::{CachedCall, CachedCallRetval};
use crate::config::{Config, Geoclue};
use crate::geo::Coords;
use crate::geoclue;
use crate::heif;
use crate::info::ImageInfo;
use crate::loader::WallpaperLoader;
use crate::schedule::{
    current_image_index_h24, current_image_index_solar, get_image_index_order_appearance,
    get_image_index_order_h24, get_image_index_order_solar,
};
use crate::setter::{set_wallpaper, unset_wallpaper};
use crate::signals::{interruptible_sleep, WakeEvent};
use crate::wallpaper::{self, properties::Properties, Wallpaper};
use crate::{cache::LastWallpaper, schedule::current_image_index_appearance};

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
    wake_rx: &Receiver<WakeEvent>,
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
            set_wallpaper(current_image_path, config.setter.as_ref())?;

            if !daemon {
                eprintln!("Wallpaper set!");
                break;
            }
        }

        let update_interval_seconds = config.daemon.update_interval_seconds;
        debug!("sleeping for {update_interval_seconds} seconds");
        match interruptible_sleep(Duration::from_secs(update_interval_seconds), wake_rx)? {
            Some(WakeEvent::Terminated) => {
                unset_wallpaper()?;
                break;
            }
            Some(WakeEvent::ThemeChanged) => {
                debug!("woke up due to theme change, re-evaluating wallpaper");
            }
            None => {}
        }
    }

    Ok(())
}

pub fn unset() -> Result<()> {
    let did_unset = unset_wallpaper()?;
    if did_unset {
        eprintln!("Wallpaper unset!");
    } else {
        eprintln!("No setter process found. Can't unset.");
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

pub fn preview<P: AsRef<Path>>(
    path: P,
    delay: u64,
    repeat: bool,
    wake_rx: &Receiver<WakeEvent>,
) -> Result<()> {
    let config = Config::find()?;
    validate_wallpaper_file(&path)?;
    let wallpaper = WallpaperLoader::new().load(&path);
    let image_order = match wallpaper.properties {
        Properties::H24(ref props) => get_image_index_order_h24(&props.time_info),
        Properties::Solar(ref props) => get_image_index_order_solar(&props.solar_info),
        Properties::Appearance(ref props) => get_image_index_order_appearance(props),
    };

    let mut should_terminate = false;
    while !should_terminate {
        for image_index in &image_order {
            if should_terminate {
                break;
            }

            let image_path = wallpaper.images.get(*image_index).unwrap();
            set_wallpaper(image_path, config.setter.as_ref())?;

            should_terminate = matches!(
                interruptible_sleep(Duration::from_millis(delay), wake_rx)?,
                Some(WakeEvent::Terminated)
            );
        }

        if !repeat {
            break;
        }
    }

    unset_wallpaper()?;
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
                user_appearance.unwrap(),
            )),
            None => bail!("wallpaper missing appearance metadata"),
        },
        Properties::Appearance(ref appearance_props) => {
            let appearance = resolve_appearance(user_appearance);
            Ok(current_image_index_appearance(appearance_props, appearance))
        }
        Properties::H24(ref props) => current_image_index_h24(&props.time_info, now.time()),
        Properties::Solar(ref props) => {
            current_image_index_solar(&props.solar_info, &now, &try_get_location(config)?)
        }
    }
}

fn resolve_appearance(user_appearance: Option<Appearance>) -> Appearance {
    match user_appearance {
        Some(appearance) => appearance,
        None => match get_system_appearance() {
            Ok(Some(appearance)) => appearance,
            Ok(None) => {
                debug!("system has no appearance preference, falling back to light");
                Appearance::Light
            }
            Err(e) => {
                log::warn!("failed to detect system appearance, falling back to light: {e}");
                Appearance::Light
            }
        },
    }
}

fn try_get_location(config: &Config) -> Result<Coords> {
    let maybe_location = match (config.geoclue.enable, config.geoclue.prefer) {
        (true, true) => match try_get_geoclue_location(&config.geoclue) {
            geoclue_ok @ Ok(_) => geoclue_ok,
            Err(e) => {
                debug!("GeoClue failed, falling back to config location: {e}");
                match config.try_get_location() {
                    config_ok @ Ok(_) => config_ok,
                    Err(_) => Err(e).context("failed to get location from GeoClue and config"),
                }
            }
        },
        (true, false) => match config.try_get_location() {
            config_ok @ Ok(_) => config_ok,
            Err(e) => {
                debug!("Config location failed, falling back to GeoClue: {e}");
                match try_get_geoclue_location(&config.geoclue) {
                    goeclue_ok @ Ok(_) => goeclue_ok,
                    geoclue_err @ Err(_) => {
                        geoclue_err.context("failed to get location from config and GeoClue")
                    }
                }
            }
        },
        (false, _) => config
            .try_get_location()
            .context("GeoClue is disabled and failed to get location from config"),
    };

    maybe_location.with_context(|| {
        format!(
            concat!(
                "Using wallpapers with solar schedule requires your approximate location information. ",
                "Please enable GeoClue 2 or provide the location manually in the configuration file at {}."
            ),
            Config::find_path().unwrap().display()
        )
    })
}

fn try_get_geoclue_location(geoclue_config: &Geoclue) -> Result<Coords> {
    let geoclue_timeout = Duration::from_millis(geoclue_config.timeout);
    let get_location = || geoclue::get_location(geoclue_timeout);

    if geoclue_config.cache_fallback {
        let cache = CachedCall::find("location");
        let location = cache.call_with_fallback(get_location)?;
        match location {
            CachedCallRetval::Fresh(value) => Ok(value),
            CachedCallRetval::Cached(value) => {
                debug!("GeoClue failed but cached value present: {value:?}, falling back",);
                Ok(value)
            }
        }
    } else {
        get_location()
    }
}
