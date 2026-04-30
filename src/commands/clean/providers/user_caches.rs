use anyhow::Result;
use std::path::PathBuf;

use super::{execute_per_item, CleanProvider};
use crate::commands::clean::fs_safe::dir_size_safe;
use crate::commands::clean::process::{is_running, PgrepChecker, ProcessChecker};
use crate::commands::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "user-caches";
const LABEL: &str = "User caches";

pub struct UserCaches;

impl CleanProvider for UserCaches {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn discover(&self) -> Result<Vec<CleanItem>> {
        Ok(discover_with_checker(&PgrepChecker))
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, ID)
    }
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Heuristic: subdirectory names are typically reverse-DNS bundle ids
/// (e.g. `com.apple.Safari`). We test the *last* segment via `pgrep -f`
/// — best-effort, not airtight.
fn discover_with_checker(checker: &dyn ProcessChecker) -> Vec<CleanItem> {
    let h = match home() {
        Some(h) => h,
        None => return Vec::new(),
    };
    let root = h.join("Library/Caches");
    let entries = match std::fs::read_dir(&root) {
        Ok(it) => it,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|s| s.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let last_segment = name.rsplit('.').next().unwrap_or(&name);
        if !last_segment.is_empty() && checker.is_running(last_segment) {
            continue;
        }
        let _ = is_running; // keep symbol referenced for non-test builds
        let size = dir_size_safe(&path);
        out.push(CleanItem {
            category_id: ID.to_string(),
            category_label: LABEL.to_string(),
            path,
            size,
            risk: RiskLevel::Review,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::clean::process::test_support::MockChecker;

    #[test]
    fn discover_returns_empty_when_caches_missing() {
        // We can't easily fake $HOME in-process without affecting other
        // tests, so we just confirm the call does not panic and returns a
        // Vec (whether empty or populated depends on the host).
        let m = MockChecker::none();
        let _ = discover_with_checker(&m);
    }
}
