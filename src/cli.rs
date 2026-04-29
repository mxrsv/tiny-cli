use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "tiny",
    version,
    about = "A small, practical CLI for performance and productivity",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show system information (CPU, memory, disk, uptime)
    Sys,

    /// Scan common folders and report large or old files (read-only)
    Clean(CleanOpts),

    /// Run a focus timer session and log it locally
    Focus(FocusOpts),
}

#[derive(Args, Debug)]
pub struct CleanOpts {
    /// Report files at or above this size in megabytes
    #[arg(long, default_value_t = 100)]
    pub min_size_mb: u64,

    /// Report files older than this many days
    #[arg(long, default_value_t = 90)]
    pub older_than_days: u64,
}

#[derive(Args, Debug)]
pub struct FocusOpts {
    /// Length of the focus session in minutes
    #[arg(long, default_value_t = 25)]
    pub minutes: u64,

    /// Optional label for the session (e.g. "deep work")
    #[arg(long)]
    pub label: Option<String>,
}
