use anyhow::{Ok, Result};
use clap::Parser;
use loader::WallpaperLoader;
use std::path::Path;
use chrono::prelude::*;
use chrono::Duration;
use std::f64::consts::PI;

mod cache;
mod cli;
mod heic;
mod loader;
mod metadata;
mod properties;
mod wallpaper;
mod selection;
mod geo;

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

    let lat = 50.16;
    let lon = 19.10;
    let mut time = Local.ymd(2022, 9, 30).and_hms(0, 0, 0);
    let time_end = Local.ymd(2022, 9, 30).and_hms(23, 0, 0);
    while time < time_end {
        time = time + Duration::minutes(30);
        let sun_pos = sun::pos(time.timestamp_millis(), lat, lon);
        println!("{}: {} {}", time, sun_pos.azimuth * 180.0 / PI, sun_pos.altitude);
    }

    Ok(())
}
