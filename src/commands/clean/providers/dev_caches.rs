use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

use super::{execute_per_item, root_as_item, CleanProvider};
use crate::commands::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn which(bin: &str) -> bool {
    Command::new("which")
        .arg(bin)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_for_path(bin: &str, args: &[&str]) -> Option<PathBuf> {
    let output = Command::new(bin).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if s.is_empty() {
        return None;
    }
    Some(PathBuf::from(s))
}

// ---------- Cargo ----------

const CARGO_ID: &str = "cargo";
const CARGO_LABEL: &str = "Cargo cache";

/// Cargo subdirs we will touch — pinned. Anything else under `~/.cargo`
/// (especially `bin/`, `config.toml`, `credentials*`) is off-limits.
const CARGO_SUBDIRS: &[&str] = &[
    "registry/cache",
    "registry/src",
    "git/db",
    "git/checkouts",
];

pub struct CargoCache;

impl CleanProvider for CargoCache {
    fn id(&self) -> &'static str {
        CARGO_ID
    }
    fn label(&self) -> &'static str {
        CARGO_LABEL
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn available(&self) -> bool {
        home().map(|h| h.join(".cargo").is_dir()).unwrap_or(false)
    }
    fn discover(&self) -> Result<Vec<CleanItem>> {
        let h = match home() {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        let cargo = h.join(".cargo");
        let mut items = Vec::new();
        for sub in CARGO_SUBDIRS {
            let path = cargo.join(sub);
            items.extend(root_as_item(
                &path,
                CARGO_ID,
                CARGO_LABEL,
                RiskLevel::Review,
            ));
        }
        Ok(items)
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        for item in items {
            debug_assert!(
                is_safe_cargo_path(&item.path),
                "cargo provider produced unsafe path: {}",
                item.path.display()
            );
        }
        execute_per_item(items, action, CARGO_ID)
    }
}

/// Guard: returned only true for paths under `~/.cargo/<one of CARGO_SUBDIRS>`.
/// Used as a debug_assert in execute() and as a unit-testable invariant.
pub fn is_safe_cargo_path(path: &std::path::Path) -> bool {
    let h = match home() {
        Some(h) => h,
        None => return false,
    };
    let cargo = h.join(".cargo");
    CARGO_SUBDIRS
        .iter()
        .any(|sub| path == cargo.join(sub).as_path())
}

// ---------- npm ----------

const NPM_ID: &str = "npm";
const NPM_LABEL: &str = "npm cache";

pub struct NpmCache;

impl CleanProvider for NpmCache {
    fn id(&self) -> &'static str {
        NPM_ID
    }
    fn label(&self) -> &'static str {
        NPM_LABEL
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn available(&self) -> bool {
        which("npm")
    }
    fn discover(&self) -> Result<Vec<CleanItem>> {
        let path = match run_for_path("npm", &["config", "get", "cache"]) {
            Some(p) => p,
            None => return Ok(Vec::new()),
        };
        Ok(root_as_item(&path, NPM_ID, NPM_LABEL, RiskLevel::Review))
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, NPM_ID)
    }
}

// ---------- pnpm ----------

const PNPM_ID: &str = "pnpm";
const PNPM_LABEL: &str = "pnpm store";

pub struct PnpmStore;

impl CleanProvider for PnpmStore {
    fn id(&self) -> &'static str {
        PNPM_ID
    }
    fn label(&self) -> &'static str {
        PNPM_LABEL
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn available(&self) -> bool {
        which("pnpm")
    }
    fn discover(&self) -> Result<Vec<CleanItem>> {
        let path = match run_for_path("pnpm", &["store", "path"]) {
            Some(p) => p,
            None => return Ok(Vec::new()),
        };
        Ok(root_as_item(&path, PNPM_ID, PNPM_LABEL, RiskLevel::Review))
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, PNPM_ID)
    }
}

// ---------- yarn ----------

const YARN_ID: &str = "yarn";
const YARN_LABEL: &str = "yarn cache";

pub struct YarnCache;

impl CleanProvider for YarnCache {
    fn id(&self) -> &'static str {
        YARN_ID
    }
    fn label(&self) -> &'static str {
        YARN_LABEL
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn available(&self) -> bool {
        which("yarn")
    }
    fn discover(&self) -> Result<Vec<CleanItem>> {
        let path = match run_for_path("yarn", &["cache", "dir"]) {
            Some(p) => p,
            None => return Ok(Vec::new()),
        };
        Ok(root_as_item(&path, YARN_ID, YARN_LABEL, RiskLevel::Review))
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, YARN_ID)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_safe_path_accepts_pinned_subdirs() {
        let h = match home() {
            Some(h) => h,
            None => return,
        };
        for sub in CARGO_SUBDIRS {
            assert!(is_safe_cargo_path(&h.join(".cargo").join(sub)));
        }
    }

    #[test]
    fn cargo_safe_path_rejects_dangerous_paths() {
        let h = match home() {
            Some(h) => h,
            None => return,
        };
        assert!(!is_safe_cargo_path(&h.join(".cargo/bin")));
        assert!(!is_safe_cargo_path(&h.join(".cargo/config.toml")));
        assert!(!is_safe_cargo_path(&h.join(".cargo/credentials")));
        assert!(!is_safe_cargo_path(&h.join(".cargo/credentials.toml")));
        assert!(!is_safe_cargo_path(&h.join(".rustup")));
        assert!(!is_safe_cargo_path(&h.join(".cargo")));
    }
}
