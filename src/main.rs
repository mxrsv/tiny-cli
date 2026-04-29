mod cli;
mod commands;

use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sys => commands::sys::run(),
        Commands::Clean(opts) => commands::clean::run(opts),
        Commands::Focus(opts) => commands::focus::run(opts),
    }
}
