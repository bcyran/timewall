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
mod pidfile;
mod schedule;
mod setter;
mod signals;
mod wallpaper;

use anyhow::Result;
use clap::Parser;
use signal_hook::{
    consts::signal::{SIGINT, SIGQUIT, SIGTERM},
    iterator::Signals,
};
use signals::start_signal_handler;

fn main() -> Result<()> {
    let termination_rx = start_signal_handler(Signals::new([SIGINT, SIGTERM, SIGQUIT])?);

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
        } => actions::preview(file, delay, repeat, &termination_rx),
        cli::Action::Unpack { file, output } => actions::unpack(file, output),
        cli::Action::Set {
            file,
            daemon,
            appearance,
        } => actions::set(file.as_ref(), daemon, appearance, &termination_rx),
        cli::Action::Unset => actions::unset(),
        cli::Action::Clear { all } => {
            actions::clear(all);
            Ok(())
        }
    }
}
