use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

/// All-in-one tool for Apple dynamic HEIF wallpapers on GNU/Linux.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Action subcommand
    #[command(subcommand)]
    pub action: Action,
}

/// Main action subcommands
#[derive(Subcommand, Debug)]
pub enum Action {
    /// Print out wallpaper info
    Info {
        /// Path to HEIF wallpaper file
        file: PathBuf,
    },
    /// Quickly cycle through all images in the wallpaper
    Preview {
        /// Path to HEIF wallpaper file
        file: PathBuf,
        /// Delay between wallpaper changes in milliseconds
        #[arg(short, long, default_value_t = 500)]
        delay: u64,
        /// Repeat the preview in a loop until killed
        #[arg(short, long, action)]
        repeat: bool,
    },
    /// Extract all images and metadata from HEIF wallpaper to a directory
    Unpack {
        /// Path to HEIF wallpaper file
        file: PathBuf,
        /// Path to output directory
        output: PathBuf,
    },
    /// Set the wallpaper
    Set {
        /// Path to HEIF wallpaper file
        file: Option<PathBuf>,
        /// Run continuously and update the wallpaper as time passes
        #[arg(short, long, action)]
        daemon: bool,
        /// Use light or dark variant
        #[arg(short, long, value_enum)]
        appearance: Option<Appearance>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Appearance {
    Light,
    Dark,
}
