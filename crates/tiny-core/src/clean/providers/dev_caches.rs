use crate::error::Result;
use std::path::PathBuf;
use std::sync::Arc;

use super::{execute_per_item, root_as_item, run_tool, CleanProvider};
use crate::clean::scan_context::ScanContext;
use crate::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};
use crate::runner::CommandRunner;

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Cache directory printed by a package manager. A tool that cannot run is
/// an error; a non-zero exit or empty output means "no cache".
fn tool_path(runner: &dyn CommandRunner, bin: &str, args: &[&str]) -> Result<Option<PathBuf>> {
    let output = run_tool(runner, bin, args)?;
    let s = output.stdout.trim();
    if !output.success() || s.is_empty() {
        return Ok(None);
    }
    Ok(Some(PathBuf::from(s)))
}

// ---------- Cargo ----------

const CARGO_ID: &str = "cargo";
const CARGO_LABEL: &str = "Cargo cache";

/// Cargo subdirs we will touch — pinned. Anything else under `~/.cargo`
/// (especially `bin/`, `config.toml`, `credentials*`) is off-limits.
const CARGO_SUBDIRS: &[&str] = &["registry/cache", "registry/src", "git/db", "git/checkouts"];

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
    fn discover(&self, ctx: &ScanContext<'_>) -> Result<Vec<CleanItem>> {
        let h = match home() {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        let cargo = h.join(".cargo");
        let mut items = Vec::new();
        for sub in CARGO_SUBDIRS {
            let path = cargo.join(sub);
            items.extend(root_as_item(
                ctx,
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

pub struct NpmCache {
    runner: Arc<dyn CommandRunner>,
}

impl NpmCache {
    pub fn with_runner(runner: Arc<dyn CommandRunner>) -> Self {
        Self { runner }
    }
}

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
        self.runner.which("npm")
    }
    fn required_tool(&self) -> Option<&'static str> {
        Some("npm")
    }
    fn discover(&self, ctx: &ScanContext<'_>) -> Result<Vec<CleanItem>> {
        let path = match tool_path(self.runner.as_ref(), "npm", &["config", "get", "cache"])? {
            Some(p) => p,
            None => return Ok(Vec::new()),
        };
        Ok(root_as_item(
            ctx,
            &path,
            NPM_ID,
            NPM_LABEL,
            RiskLevel::Review,
        ))
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, NPM_ID)
    }
}

// ---------- pnpm ----------

const PNPM_ID: &str = "pnpm";
const PNPM_LABEL: &str = "pnpm store";

pub struct PnpmStore {
    runner: Arc<dyn CommandRunner>,
}

impl PnpmStore {
    pub fn with_runner(runner: Arc<dyn CommandRunner>) -> Self {
        Self { runner }
    }
}

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
        self.runner.which("pnpm")
    }
    fn required_tool(&self) -> Option<&'static str> {
        Some("pnpm")
    }
    fn discover(&self, ctx: &ScanContext<'_>) -> Result<Vec<CleanItem>> {
        let path = match tool_path(self.runner.as_ref(), "pnpm", &["store", "path"])? {
            Some(p) => p,
            None => return Ok(Vec::new()),
        };
        Ok(root_as_item(
            ctx,
            &path,
            PNPM_ID,
            PNPM_LABEL,
            RiskLevel::Review,
        ))
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, PNPM_ID)
    }
}

// ---------- yarn ----------

const YARN_ID: &str = "yarn";
const YARN_LABEL: &str = "yarn cache";

pub struct YarnCache {
    runner: Arc<dyn CommandRunner>,
}

impl YarnCache {
    pub fn with_runner(runner: Arc<dyn CommandRunner>) -> Self {
        Self { runner }
    }
}

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
        self.runner.which("yarn")
    }
    fn required_tool(&self) -> Option<&'static str> {
        Some("yarn")
    }
    fn discover(&self, ctx: &ScanContext<'_>) -> Result<Vec<CleanItem>> {
        let path = match tool_path(self.runner.as_ref(), "yarn", &["cache", "dir"])? {
            Some(p) => p,
            None => return Ok(Vec::new()),
        };
        Ok(root_as_item(
            ctx,
            &path,
            YARN_ID,
            YARN_LABEL,
            RiskLevel::Review,
        ))
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
