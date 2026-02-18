#[macro_use]
mod macros;
mod actions;
mod appearance;
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

use std::sync::mpsc::channel;

use anyhow::Result;
use clap::Parser;
use signal_hook::{
    consts::signal::{SIGINT, SIGQUIT, SIGTERM},
    iterator::Signals,
};
use signals::{start_appearance_change_handler, start_signal_handler, WakeEvent};

impl From<cli::CliAppearance> for appearance::Appearance {
    fn from(cli: cli::CliAppearance) -> Self {
        match cli {
            cli::CliAppearance::Light => Self::Light,
            cli::CliAppearance::Dark => Self::Dark,
        }
    }
}

fn main() -> Result<()> {
    let (wake_tx, wake_rx) = channel::<WakeEvent>();

    start_signal_handler(Signals::new([SIGINT, SIGTERM, SIGQUIT])?, wake_tx.clone());

    let args = cli::Args::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    if matches!(args.action, cli::Action::Set { daemon: true, .. }) {
        start_appearance_change_handler(wake_tx);
    }

    match args.action {
        cli::Action::Info { file } => actions::info(file),
        cli::Action::Preview {
            file,
            delay,
            repeat,
        } => actions::preview(file, delay, repeat, &wake_rx),
        cli::Action::Unpack { file, output } => actions::unpack(file, output),
        cli::Action::Set {
            file,
            daemon,
            appearance,
        } => actions::set(file.as_ref(), daemon, appearance.map(Into::into), &wake_rx),
        cli::Action::Unset => actions::unset(),
        cli::Action::Clear { all } => {
            actions::clear(all);
            Ok(())
        }
    }
}
