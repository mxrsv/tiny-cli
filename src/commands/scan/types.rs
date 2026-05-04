//! Shared data types for the scan command.

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub age_secs: u64,
}

#[derive(Debug, Default)]
pub struct ScanData {
    pub roots: Vec<PathBuf>,
    pub all_files: Vec<FileEntry>,
    pub large: Vec<FileEntry>,
    pub old: Vec<FileEntry>,
}
