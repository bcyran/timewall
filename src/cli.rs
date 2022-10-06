use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

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
    /// Quickly cycle through all images in the wallpaper
    Preview {
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
