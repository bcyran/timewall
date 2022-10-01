use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// All-in-one tool for Apple dynamic HEIC wallpapers on GNU/Linux
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Action subcommand
    #[clap(subcommand)]
    pub action: Action,
}

/// Main action subcommands
#[derive(Subcommand, Debug)]
pub enum Action {
    /// Print out image info
    Info {
        /// Path to HEIC file
        #[clap(parse(from_os_str))]
        file: PathBuf,
    },
    /// Extract all images and metadata from HEIC file to a directory
    Unpack {
        /// Path to HEIC file
        #[clap(parse(from_os_str))]
        file: PathBuf,
        /// Path to directory to output directory
        #[clap(parse(from_os_str))]
        output: PathBuf,
    },
    /// Set the wallpaper
    Set {
        /// Path to HEIC file
        #[clap(parse(from_os_str))]
        file: PathBuf,
    },
}
