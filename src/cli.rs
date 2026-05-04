use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

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
    Scan(ScanOpts),

    /// Run a focus timer session and log it locally
    Focus(FocusOpts),

    /// Uninstall apps from /Applications and clean ~/Library leftovers
    Uninstall(UninstallOpts),

    /// Interactive cleanup of developer caches and recoverable data
    Clean(CleanOpts),
}

#[derive(Args, Debug)]
pub struct ScanOpts {
    /// Report files at or above this size in megabytes
    #[arg(long, default_value_t = 100)]
    pub min_size_mb: u64,

    /// Report files older than this many days
    #[arg(long, default_value_t = 90)]
    pub older_than_days: u64,

    /// Custom directory to scan. Repeatable. Overrides the default
    /// Downloads/Desktop/Documents set when provided at least once.
    #[arg(long, value_name = "DIR")]
    pub path: Vec<PathBuf>,

    /// Emit a machine-readable JSON report on stdout instead of text.
    #[arg(long)]
    pub json: bool,

    /// Maximum number of items to display per section (text mode).
    #[arg(long, default_value_t = 20)]
    pub limit: usize,

    /// Sort order applied to file listings.
    #[arg(long, value_enum, default_value_t = ScanSort::Size)]
    pub sort: ScanSort,

    /// Group duplicate files by (size + filename).
    #[arg(long)]
    pub duplicates: bool,

    /// Verify duplicate groups by hashing file contents. Slower but accurate.
    #[arg(long, requires = "duplicates")]
    pub hash: bool,

    /// Print a per-extension breakdown (count + total size).
    #[arg(long)]
    pub by_ext: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScanSort {
    /// Largest files first
    Size,
    /// Oldest files first
    Age,
    /// Lexicographic path order
    Path,
}

#[derive(Args, Debug)]
pub struct UninstallOpts {
    /// App name (e.g. "Cursor"). If omitted, an interactive picker is shown.
    pub name: Option<String>,

    /// Only show the report and exit. Does not prompt for action.
    #[arg(long)]
    pub dry_run: bool,

    /// Skip the action menu and execute immediately (Trash, or rm -rf if --hard).
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Only remove /Applications/<Name>.app, skip ~/Library cleanup.
    #[arg(long, conflicts_with = "leftovers_only")]
    pub shallow: bool,

    /// Only clean ~/Library leftovers, keep /Applications/<Name>.app.
    #[arg(long, conflicts_with = "shallow")]
    pub leftovers_only: bool,

    /// rm -rf instead of moving to Trash. NOT recoverable.
    #[arg(long)]
    pub hard: bool,

    /// Sort order in the interactive picker.
    #[arg(long, value_enum, default_value_t = SortBy::LastUsed)]
    pub sort: SortBy,

    /// Allow uninstalling Homebrew casks (default: warn and refuse).
    #[arg(long)]
    pub force: bool,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum SortBy {
    /// Least recently used first (default)
    LastUsed,
    /// Largest size first
    Size,
    /// Alphabetical
    Name,
}

#[derive(Args, Debug)]
pub struct CleanOpts {
    /// Show the report and exit without prompting for cleanup.
    #[arg(long, conflicts_with = "yes")]
    pub dry_run: bool,

    /// Skip the picker and the action menu. Requires --category.
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Use permanent deletion instead of Move to Trash. NOT recoverable.
    #[arg(long)]
    pub hard: bool,

    /// Restrict discovery and selection to the named category. Repeatable.
    #[arg(long, action = clap::ArgAction::Append)]
    pub category: Vec<String>,

    /// Show review-risk categories in the picker (unchecked).
    #[arg(long)]
    pub include_review: bool,

    /// Show destructive categories such as Trash in the picker (unchecked).
    #[arg(long)]
    pub include_destructive: bool,
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
