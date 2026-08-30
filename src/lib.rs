mod app;
mod browser;
mod cargo;
mod cli;
mod error;
mod git;
mod providers;
mod remote;
mod ssh_config;

use std::process::ExitCode;

use clap::Parser;

use crate::cli::Cli;

pub fn run_from_env() -> ExitCode {
    let cli = Cli::parse();
    run(cli)
}

fn run(cli: Cli) -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Failed to determine current directory: {error}");
            return ExitCode::from(1);
        }
    };

    match app::run(cli, &cwd) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}
