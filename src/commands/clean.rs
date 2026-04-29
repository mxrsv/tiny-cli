use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};

use crate::cli::CleanOpts;

const TARGET_FOLDERS: [&str; 3] = ["Downloads", "Desktop", "Documents"];

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
        scan_dir(&dir, min_size_bytes, older_than_secs, &mut large, &mut old);
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
    large: &mut Vec<FileEntry>,
    old: &mut Vec<FileEntry>,
) {
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
            scan_dir(&path, min_size, older_than_secs, large, old);
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

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{:.1} {}", value, UNITS[unit])
    }
}
