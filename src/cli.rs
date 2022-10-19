use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

/// All-in-one tool for Apple dynamic HEIF wallpapers on GNU/Linux
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
        /// Path to HEIF file
        #[clap(parse(from_os_str))]
        file: PathBuf,
    },
    /// Quickly cycle through all images in the wallpaper
    Preview {
        /// Path to HEIF file
        #[clap(parse(from_os_str))]
        file: PathBuf,
        /// Delay between wallpaper changes in milliseconds.
        #[clap(short, long, default_value_t = 500)]
        delay: u64,
        /// Repeat the preview in a loop until killed
        #[clap(short, long, action)]
        repeat: bool,
    },
    /// Extract all images and metadata from HEIF file to a directory
    Unpack {
        /// Path to HEIF file
        #[clap(parse(from_os_str))]
        file: PathBuf,
        /// Path to directory to output directory
        #[clap(parse(from_os_str))]
        output: PathBuf,
    },
    /// Set the wallpaper
    Set {
        /// Path to HEIF file
        #[clap(parse(from_os_str))]
        file: Option<PathBuf>,
        /// Run continuously and update the wallpaper as time passes
        #[clap(short, long, action)]
        daemon: bool,
        /// Use light or dark variant
        #[clap(short, long, action, arg_enum, value_parser)]
        appearance: Option<Appearance>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Appearance {
    Light,
    Dark,
}
