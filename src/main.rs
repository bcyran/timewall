use anyhow::Result;
use libheif_rs::HeifContext;
use std::path::PathBuf;

use clap::Parser;

mod metadata;
mod wallpaper_plist;
use metadata::AppleDesktop;
use wallpaper_plist::{WallpaperPlistH24, WallpaperPlistSolar};

/// Read Apple dynamic wallpaper metadata from HEIC files
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    ///Path to HEIC file to read
    #[clap(parse(from_os_str))]
    file: PathBuf,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    println!("File: {}", args.file.display());

    let heif_ctx = HeifContext::read_from_file(args.file.to_str().unwrap())?;
    let meta = AppleDesktop::from_heif(&heif_ctx)?;

    match meta {
        AppleDesktop::H24(value) => println!("{:?}", WallpaperPlistH24::from_base64(value)?),
        AppleDesktop::Solar(value) => println!("{:?}", WallpaperPlistSolar::from_base64(value)?),
    }

    Ok(())
}
