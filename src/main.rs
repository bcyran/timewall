use anyhow::Result;
use std::path::PathBuf;

use clap::Parser;

mod heic;
mod metadata;
mod properties;
mod wallpaper;

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

    let test_path = "local/test";

    wallpaper::unpack_heic(&args.file, test_path)?;
    let wall = wallpaper::Wallpaper::load(test_path)?;
    println!("{wall:?}");

    Ok(())
}
