use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};

use crate::cli::CleanOpts;
use crate::util::format_bytes;

const TARGET_FOLDERS: [&str; 3] = ["Downloads", "Desktop", "Documents"];
const MAX_DEPTH: usize = 12;
const SKIP_DIR_NAMES: &[&str] = &[
    // VCS
    ".git", ".svn", ".hg",
    // Package / build outputs that explode in size and noise
    "node_modules", "vendor", "target", "build", "dist", ".next", ".nuxt",
    ".turbo", ".cache", "__pycache__", ".venv", "venv",
    // macOS metadata
    ".Trash", ".Spotlight-V100", ".fseventsd", ".DocumentRevisions-V100",
    ".TemporaryItems", ".DS_Store",
];

fn should_skip_dir(name: &str) -> bool {
    SKIP_DIR_NAMES.iter().any(|s| *s == name)
}

pub fn run(opts: CleanOpts) -> Result<()> {
    let home = home_dir().context("could not resolve user home directory")?;
    let min_size_bytes = opts.min_size_mb.saturating_mul(1024 * 1024);
    let older_than_secs = opts.older_than_days.saturating_mul(86_400);

    println!("Scan target folders under: {}", home.display());
    println!(
        "Thresholds: size >= {} MB, age >= {} days",
        opts.min_size_mb, opts.older_than_days
    );
    println!("(read-only — no files are deleted)");
    println!();

    let mut large: Vec<FileEntry> = Vec::new();
    let mut old: Vec<FileEntry> = Vec::new();

    for folder in TARGET_FOLDERS {
        let dir = home.join(folder);
        if !dir.is_dir() {
            continue;
        }
        scan_dir(
            &dir,
            min_size_bytes,
            older_than_secs,
            0,
            &mut large,
            &mut old,
        );
    }

    large.sort_by(|a, b| b.size.cmp(&a.size));
    old.sort_by(|a, b| b.age_secs.cmp(&a.age_secs));

    println!("== Large files ({}) ==", large.len());
    for entry in large.iter().take(20) {
        println!("{:>10}  {}", format_bytes(entry.size), entry.path.display());
    }
    if large.len() > 20 {
        println!("... and {} more", large.len() - 20);
    }

    println!();
    println!("== Old files ({}) ==", old.len());
    for entry in old.iter().take(20) {
        println!(
            "{:>5}d  {}",
            entry.age_secs / 86_400,
            entry.path.display()
        );
    }
    if old.len() > 20 {
        println!("... and {} more", old.len() - 20);
    }

    Ok(())
}

struct FileEntry {
    path: PathBuf,
    size: u64,
    age_secs: u64,
}

fn scan_dir(
    dir: &Path,
    min_size: u64,
    older_than_secs: u64,
    depth: usize,
    large: &mut Vec<FileEntry>,
    old: &mut Vec<FileEntry>,
) {
    if depth >= MAX_DEPTH {
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(it) => it,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if metadata.is_dir() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if should_skip_dir(&name_str) {
                continue;
            }
            scan_dir(&path, min_size, older_than_secs, depth + 1, large, old);
            continue;
        }
        if !metadata.is_file() {
            continue;
        }

        let size = metadata.len();
        let age_secs = metadata
            .modified()
            .ok()
            .and_then(|t| SystemTime::now().duration_since(t).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if size >= min_size {
            large.push(FileEntry {
                path: path.clone(),
                size,
                age_secs,
            });
        }
        if age_secs >= older_than_secs {
            old.push(FileEntry {
                path,
                size,
                age_secs,
            });
        }
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

