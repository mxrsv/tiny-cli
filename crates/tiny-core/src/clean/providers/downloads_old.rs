use std::path::{Path, PathBuf};

use crate::error::Result;

use super::{execute_per_item, is_idle, CleanProvider};
use crate::clean::fs_safe::{dir_size_checked, is_dir_safe, list_children};
use crate::clean::scan_context::ScanContext;
use crate::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "downloads-old";
const LABEL: &str = "Downloads (old files)";

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub struct DownloadsOld {
    pub idle_days: u64,
}

impl DownloadsOld {
    pub fn new(idle_days: u64) -> Self {
        Self { idle_days }
    }
}

impl CleanProvider for DownloadsOld {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn inclusion_reason(&self) -> String {
        format!(
            "Files directly in Downloads not modified for {} days",
            self.idle_days
        )
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn desktop_trash_paths(&self) -> bool {
        true
    }
    fn available(&self) -> bool {
        home()
            .map(|h| is_dir_safe(&h.join("Downloads")))
            .unwrap_or(false)
    }
    fn discover(&self, ctx: &ScanContext<'_>) -> Result<Vec<CleanItem>> {
        let h = match home() {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        Ok(list_old_files(ctx, &h.join("Downloads"), self.idle_days))
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, ID)
    }
}

/// Lists every file (not directory) directly inside `dir` whose mtime is
/// older than `idle_days`. Non-recursive — subdirs are not descended (we
/// don't want to recurse into a user's curated download folders).
pub fn list_old_files(ctx: &ScanContext<'_>, dir: &Path, idle_days: u64) -> Vec<CleanItem> {
    let mut out = Vec::new();
    for path in list_children(dir, ctx) {
        let meta = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if !meta.file_type().is_file() {
            continue;
        }
        if !is_idle(&path, idle_days) {
            continue;
        }
        let size = dir_size_checked(&path, ctx);
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
    use crate::clean::providers::known_category_ids;
    use std::fs;

    fn tempdir(label: &str) -> PathBuf {
        let mut base = std::env::temp_dir();
        base.push(format!(
            "tiny-clean-dl-{}-{}",
            label,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&base).unwrap();
        base
    }

    fn backdate(p: &Path, days: u64) {
        let old = std::time::SystemTime::now() - std::time::Duration::from_secs(days * 86_400);
        std::fs::File::open(p).unwrap().set_modified(old).unwrap();
    }

    #[test]
    fn downloads_old_id_in_known_categories() {
        assert!(known_category_ids().contains(&ID));
    }

    #[test]
    fn downloads_filters_by_age() {
        let dir = tempdir("age");
        let fresh = dir.join("fresh.txt");
        let stale = dir.join("stale.txt");
        fs::write(&fresh, b"new").unwrap();
        fs::write(&stale, b"old").unwrap();
        backdate(&stale, 60);
        let found = list_old_files(&ScanContext::unchecked(), &dir, 30);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].path, stale);
        let _ = crate::clean::fs_safe::remove_recursive_safe(&dir);
    }

    #[test]
    fn downloads_skips_subdirectories() {
        let dir = tempdir("subdirs");
        fs::create_dir_all(dir.join("subdir")).unwrap();
        let f = dir.join("subdir/inside.txt");
        fs::write(&f, b"x").unwrap();
        backdate(&f, 60);
        let found = list_old_files(&ScanContext::unchecked(), &dir, 30);
        // Subdirs not descended; even though file is old it's not flagged.
        assert!(found.is_empty());
        let _ = crate::clean::fs_safe::remove_recursive_safe(&dir);
    }
}
