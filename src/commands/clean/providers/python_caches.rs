use std::path::{Path, PathBuf};

use anyhow::Result;

use super::{dev_search_roots, execute_per_item, is_idle, CleanProvider};
use crate::commands::clean::fs_safe::{dir_size_safe, walk_with};
use crate::commands::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "python-caches";
const LABEL: &str = "Python __pycache__ / venv (idle)";

const VENV_NAMES: &[&str] = &["venv", ".venv", "env"];

pub struct PythonCaches {
    pub idle_days: u64,
    pub search_roots: Vec<PathBuf>,
}

impl PythonCaches {
    pub fn new(idle_days: u64) -> Self {
        Self {
            idle_days,
            search_roots: dev_search_roots(),
        }
    }
}

impl CleanProvider for PythonCaches {
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
            for found in find_pycache(root) {
                let size = dir_size_safe(&found);
                items.push(CleanItem {
                    category_id: ID.to_string(),
                    category_label: LABEL.to_string(),
                    path: found,
                    size,
                    risk: RiskLevel::Review,
                });
            }
            for found in find_orphan_venv(root, self.idle_days) {
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

/// Walks `root` symlink-safe and returns every `__pycache__` dir. Manifest
/// check NOT required — pycache is always safe to delete.
pub fn find_pycache(root: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    walk_with(root, |path, meta| {
        if !meta.file_type().is_dir() {
            return false;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("__pycache__") {
            found.push(path.to_path_buf());
            return false; // never recurse into pycache
        }
        true
    });
    found
}

/// Walks `root` symlink-safe and returns every venv-style dir whose parent
/// has a python manifest (pyproject.toml / setup.py / requirements.txt) AND
/// the manifest is idle.
pub fn find_orphan_venv(root: &Path, idle_days: u64) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    walk_with(root, |path, meta| {
        if !meta.file_type().is_dir() {
            return false;
        }
        let name = path.file_name().and_then(|n| n.to_str());
        let is_venv = matches!(name, Some(n) if VENV_NAMES.contains(&n));
        if is_venv {
            if let Some(parent) = path.parent() {
                if let Some(manifest) = python_manifest(parent) {
                    if is_idle(&manifest, idle_days) {
                        found.push(path.to_path_buf());
                    }
                }
            }
            return false; // never recurse into venv
        }
        true
    });
    found
}

/// Returns the first existing python manifest (pyproject.toml / setup.py /
/// requirements.txt) under `dir`, or None.
pub fn python_manifest(dir: &Path) -> Option<PathBuf> {
    for name in ["pyproject.toml", "setup.py", "requirements.txt"] {
        let p = dir.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::clean::providers::known_category_ids;
    use std::fs;

    fn tempdir(label: &str) -> PathBuf {
        let mut base = std::env::temp_dir();
        base.push(format!(
            "tiny-clean-py-{}-{}",
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
    fn python_caches_id_in_known_categories() {
        assert!(known_category_ids().contains(&ID));
    }

    #[test]
    fn pycache_found_anywhere_no_manifest_required() {
        let root = tempdir("pyc");
        let nested = root.join("a/b/__pycache__");
        fs::create_dir_all(&nested).unwrap();
        let found = find_pycache(&root);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], nested);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn venv_requires_python_manifest() {
        let root = tempdir("orphanvenv");
        let proj = root.join("ghost");
        fs::create_dir_all(proj.join("venv")).unwrap();
        // No manifest → must not be flagged.
        let found = find_orphan_venv(&root, 0);
        assert!(found.is_empty());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn venv_idle_threshold_applied() {
        let root = tempdir("freshvenv");
        let proj = root.join("alive");
        fs::create_dir_all(proj.join(".venv")).unwrap();
        let manifest = proj.join("pyproject.toml");
        fs::write(&manifest, b"[project]\nname=\"x\"\n").unwrap();
        let fresh = find_orphan_venv(&root, 30);
        assert!(fresh.is_empty(), "fresh manifest must not flag");
        let old = std::time::SystemTime::now()
            - std::time::Duration::from_secs(31 * 86_400);
        std::fs::File::open(&manifest)
            .unwrap()
            .set_modified(old)
            .unwrap();
        let stale = find_orphan_venv(&root, 30);
        assert_eq!(stale.len(), 1);
        let _ = fs::remove_dir_all(&root);
    }
}
