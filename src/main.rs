use anyhow::Result;
use libheif_rs::{HeifContext, ItemId};
use std::path::PathBuf;

use clap::Parser;

mod heic;
mod metadata;
mod properties;
use metadata::AppleDesktop;
use properties::{Plist, WallpaperPropertiesH24, WallpaperPropertiesSolar};

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
        AppleDesktop::H24(value) => println!("{:?}", WallpaperPropertiesH24::from_base64(value)?),
        AppleDesktop::Solar(value) => {
            println!("{:?}", WallpaperPropertiesSolar::from_base64(value)?)
        }
    }

    let number_of_images = heif_ctx.number_of_top_level_images();
    let mut image_ids = vec![0 as ItemId; number_of_images];
    heif_ctx.top_level_image_ids(&mut image_ids);
    let first_image_id = image_ids.get(0).unwrap();
    let first_image = heif_ctx.image_handle(*first_image_id)?;
    heic::write_as_png(&first_image, "local/test.png")?;

    Ok(())
}
