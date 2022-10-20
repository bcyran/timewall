use clap::CommandFactory;
use clap_complete::{generate_to, Shell};
use std::io::Error;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let var = std::env::var_os("SHELL_COMPLETIONS_DIR").or_else(|| std::env::var_os("OUT_DIR"));
    let outdir = match var {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = Args::command();
    for shell in [Shell::Bash, Shell::Fish, Shell::Zsh] {
        generate_to(shell, &mut cmd, "timewall", &outdir).unwrap();
    }

    Ok(())
}
