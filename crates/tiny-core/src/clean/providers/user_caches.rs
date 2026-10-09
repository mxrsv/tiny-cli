use crate::error::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::{execute_per_item, CleanProvider};
use crate::clean::fs_safe::{dir_size_checked, list_children};
use crate::clean::scan_context::ScanContext;
use crate::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};
use crate::runner::CommandRunner;

const ID: &str = "user-caches";
const LABEL: &str = "User caches";

pub struct UserCaches {
    #[allow(dead_code)] // read by the bundle-id lookup (plan T4)
    runner: Arc<dyn CommandRunner>,
}

impl UserCaches {
    pub fn with_runner(runner: Arc<dyn CommandRunner>) -> Self {
        Self { runner }
    }
}

impl CleanProvider for UserCaches {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn inclusion_reason(&self) -> String {
        "Per-app caches in ~/Library/Caches that apps rebuild".into()
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn desktop_trash_paths(&self) -> bool {
        true
    }
    fn item_apps(&self, path: &Path) -> Vec<String> {
        owning_app(path).map(String::from).into_iter().collect()
    }
    fn discover(&self, ctx: &ScanContext<'_>) -> Result<Vec<CleanItem>> {
        let h = match home() {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        list_caches(ctx, &h.join("Library/Caches"))
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, ID)
    }
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Heuristic: subdirectory names are typically reverse-DNS bundle ids
/// (e.g. `com.apple.Safari`). The *last* segment is the app checked with
/// `pgrep -x` — best-effort, not airtight.
pub fn owning_app(path: &Path) -> Option<&str> {
    let name = path.file_name()?.to_str()?;
    let last_segment = name.rsplit('.').next().unwrap_or(name);
    (!last_segment.is_empty()).then_some(last_segment)
}

fn list_caches(ctx: &ScanContext<'_>, root: &Path) -> Result<Vec<CleanItem>> {
    let mut out = Vec::new();
    for path in list_children(root, ctx) {
        if path.file_name().and_then(|s| s.to_str()).is_none() {
            continue;
        }
        if let Some(app) = owning_app(&path) {
            if ctx.known_running(app) {
                continue;
            }
        }
        let size = dir_size_checked(&path, ctx);
        out.push(CleanItem {
            category_id: ID.to_string(),
            category_label: LABEL.to_string(),
            path,
            size,
            risk: RiskLevel::Review,
            evidence: Vec::new(),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clean::process::test_support::MockChecker;

    #[test]
    fn caches_of_running_apps_are_skipped() {
        let root = std::env::temp_dir().join(format!("tiny-user-caches-{}", std::process::id()));
        std::fs::create_dir_all(root.join("com.apple.Safari")).unwrap();
        std::fs::create_dir_all(root.join("com.example.Idle")).unwrap();
        let m = MockChecker::with_running(["Safari"]);
        let ctx = ScanContext::new(None, &m);
        let items = list_caches(&ctx, &root).unwrap();
        let paths: Vec<_> = items.iter().map(|i| i.path.clone()).collect();
        assert_eq!(paths, vec![root.join("com.example.Idle")]);
        assert!(ctx.take_findings().roots.contains(&root));
        let _ = crate::clean::fs_safe::remove_recursive_safe(&root);
    }

    #[test]
    fn a_failed_probe_keeps_the_path_for_execution_to_recheck() {
        use crate::clean::process::test_support::FailingProbe;
        let root = std::env::temp_dir().join(format!("tiny-user-caches-f-{}", std::process::id()));
        std::fs::create_dir_all(root.join("com.apple.Safari")).unwrap();
        let ctx = ScanContext::new(None, &FailingProbe);
        assert_eq!(list_caches(&ctx, &root).unwrap().len(), 1);
        let _ = crate::clean::fs_safe::remove_recursive_safe(&root);
    }
}
