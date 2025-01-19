#[macro_use]
mod macros;
mod actions;
mod cache;
mod cli;
mod config;
mod constants;
mod geo;
mod geoclue;
mod heif;
mod info;
mod loader;
mod schedule;
mod setter;
mod wallpaper;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = cli::Args::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

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
            delay,
        } => actions::set(file.as_ref(), daemon, appearance, delay),
        cli::Action::Clear { all } => {
            actions::clear(all);
            Ok(())
        }
    }
}
