use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::cli::FocusOpts;

#[derive(Serialize, Deserialize, Debug)]
struct FocusSession {
    started_at_unix: u64,
    finished_at_unix: u64,
    minutes: u64,
    label: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct FocusLog {
    sessions: Vec<FocusSession>,
}

pub fn run(opts: FocusOpts) -> Result<()> {
    if opts.minutes == 0 {
        println!("minutes must be > 0");
        return Ok(());
    }

    let started_at = SystemTime::now();
    let label = opts.label.as_deref().unwrap_or("focus");
    println!(
        "Starting {} session: {} minute(s)",
        label, opts.minutes
    );
    println!("Press Ctrl+C to abort.");

    let total_secs = opts.minutes * 60;
    run_timer(total_secs);

    let finished_at = SystemTime::now();
    let session = FocusSession {
        started_at_unix: to_unix(started_at),
        finished_at_unix: to_unix(finished_at),
        minutes: opts.minutes,
        label: opts.label,
    };

    let log_path = log_path()?;
    append_session(&log_path, session)?;
    println!();
    println!("Session complete. Logged to {}", log_path.display());
    Ok(())
}

fn run_timer(total_secs: u64) {
    let bar_width: u64 = 30;
    for elapsed in 0..=total_secs {
        let remaining = total_secs - elapsed;
        let filled = if total_secs == 0 {
            bar_width
        } else {
            (elapsed * bar_width) / total_secs
        };
        let empty = bar_width - filled;
        let bar = format!("{}{}", "#".repeat(filled as usize), "-".repeat(empty as usize));
        let mins = remaining / 60;
        let secs = remaining % 60;
        print!("\r[{}] {:02}:{:02} remaining", bar, mins, secs);
        let _ = io::stdout().flush();

        if elapsed < total_secs {
            thread::sleep(Duration::from_secs(1));
        }
    }
}

fn to_unix(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn log_path() -> Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .context("could not resolve user home directory")?;
    let dir = home.join(".tiny-cli");
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create {}", dir.display()))?;
    Ok(dir.join("focus-sessions.json"))
}

fn append_session(path: &PathBuf, session: FocusSession) -> Result<()> {
    let mut log: FocusLog = if path.exists() {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if raw.trim().is_empty() {
            FocusLog::default()
        } else {
            serde_json::from_str(&raw).unwrap_or_default()
        }
    } else {
        FocusLog::default()
    };

    log.sessions.push(session);
    let serialized = serde_json::to_string_pretty(&log)?;
    fs::write(path, serialized)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}
