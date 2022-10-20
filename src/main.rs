#[macro_use]
mod macros;
mod actions;
mod cache;
mod cli;
mod config;
mod constants;
mod geo;
mod heif;
mod info;
mod loader;
mod schedule;
mod setter;
mod wallpaper;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    env_logger::init();

    let args = cli::Args::parse();

    match args.action {
        cli::Action::Info { file } => actions::info(file),
        cli::Action::Preview {
            file,
            delay,
            repeat,
        } => actions::preview(file, delay, repeat),
        cli::Action::Unpack { file, output } => actions::unpack(file, output),
        cli::Action::Set {
            file,
            daemon,
            appearance,
        } => actions::set(file, daemon, appearance),
    }
}
