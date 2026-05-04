use std::path::{Path, PathBuf};

use anyhow::Result;

use super::{dev_search_roots, execute_per_item, is_idle, CleanProvider};
use crate::commands::clean::fs_safe::{dir_size_safe, walk_with};
use crate::commands::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "rust-targets";
const LABEL: &str = "Rust target/ (idle)";

pub struct RustTargets {
    pub idle_days: u64,
    pub search_roots: Vec<PathBuf>,
}

impl RustTargets {
    pub fn new(idle_days: u64) -> Self {
        Self {
            idle_days,
            search_roots: dev_search_roots(),
        }
    }
}

impl CleanProvider for RustTargets {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn available(&self) -> bool {
        !self.search_roots.is_empty()
    }
    fn discover(&self) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();
        for root in &self.search_roots {
            for found in find_rust_targets(root, self.idle_days) {
                let size = dir_size_safe(&found);
                items.push(CleanItem {
                    category_id: ID.to_string(),
                    category_label: LABEL.to_string(),
                    path: found,
                    size,
                    risk: RiskLevel::Review,
                });
            }
        }
        Ok(items)
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, ID)
    }
}

/// Walks `root` symlink-safe and returns every `target` dir whose parent
/// has a `Cargo.toml` modified more than `idle_days` ago. Does NOT descend
/// into a `target` dir once found.
pub fn find_rust_targets(root: &Path, idle_days: u64) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    walk_with(root, |path, meta| {
        if !meta.file_type().is_dir() {
            return false;
        }
        let is_target = path.file_name().and_then(|n| n.to_str()) == Some("target");
        if is_target {
            if let Some(parent) = path.parent() {
                let manifest = parent.join("Cargo.toml");
                if manifest.is_file() && is_idle(&manifest, idle_days) {
                    found.push(path.to_path_buf());
                }
            }
            // Never recurse into target/.
            return false;
        }
        true
    });
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::clean::providers::known_category_ids;
    use std::fs;

    fn tempdir(label: &str) -> PathBuf {
        let mut base = std::env::temp_dir();
        base.push(format!(
            "tiny-clean-rust-{}-{}",
            label,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn rust_targets_id_in_known_categories() {
        assert!(known_category_ids().contains(&ID));
    }

    #[test]
    fn refuses_target_without_cargo_toml() {
        let root = tempdir("orphan");
        let proj = root.join("ghost");
        fs::create_dir_all(proj.join("target")).unwrap();
        let found = find_rust_targets(&root, 0);
        assert!(found.is_empty());
        let _ = fs::remove_dir_all(&root);
    }

    fn backdate(p: &Path, days: u64) {
        let old = std::time::SystemTime::now()
            - std::time::Duration::from_secs(days * 86_400);
        std::fs::File::open(p).unwrap().set_modified(old).unwrap();
    }

    #[test]
    fn idle_threshold_filters_recently_modified() {
        let root = tempdir("idle");
        let proj = root.join("alive");
        fs::create_dir_all(proj.join("target")).unwrap();
        let manifest = proj.join("Cargo.toml");
        fs::write(&manifest, b"[package]\nname = \"x\"\n").unwrap();
        let fresh = find_rust_targets(&root, 30);
        assert!(fresh.is_empty(), "fresh Cargo.toml must not flag");
        backdate(&manifest, 31);
        let stale = find_rust_targets(&root, 30);
        assert_eq!(stale.len(), 1);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn does_not_descend_into_target() {
        let root = tempdir("nested");
        let proj = root.join("p");
        let outer = proj.join("target");
        fs::create_dir_all(outer.join("target")).unwrap();
        let manifest = proj.join("Cargo.toml");
        fs::write(&manifest, b"[package]\nname = \"x\"\n").unwrap();
        backdate(&manifest, 40);
        let found = find_rust_targets(&root, 30);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], outer);
        let _ = fs::remove_dir_all(&root);
    }
}
