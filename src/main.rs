use anyhow::Result;
use clap::Parser;

mod cli;
mod heic;
mod metadata;
mod properties;
mod wallpaper;

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
    }
}
