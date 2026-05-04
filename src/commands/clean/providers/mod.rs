use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::process::Command;

use super::fs_safe::{dir_size_safe, remove_recursive_safe};
use super::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

pub mod dev_caches;
pub mod gradle_maven;
pub mod jetbrains;
pub mod trash;
pub mod user_caches;
pub mod user_logs;
pub mod vscode;
pub mod xcode;

/// Top-level grouping for the hierarchical picker. Source of truth lives in
/// `category_family()` below — providers must not declare their own family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Dev,
    UserStorage,
    System,
}

impl Family {
    #[allow(dead_code)]
    pub fn id(&self) -> &'static str {
        match self {
            Family::Dev => "dev",
            Family::UserStorage => "user-storage",
            Family::System => "system",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Family::Dev => "Dev caches",
            Family::UserStorage => "User storage",
            Family::System => "System leftovers",
        }
    }
}

/// Maps a category id to its family. Panics on unknown id — every id in
/// `known_category_ids()` MUST be covered (verified by test below).
pub fn category_family(category_id: &str) -> Family {
    match category_id {
        // Dev family
        "cargo" | "npm" | "pnpm" | "yarn" => Family::Dev,
        "gradle-maven" | "jetbrains" | "vscode" => Family::Dev,
        "xcode-derived" | "xcode-archives" | "xcode-devicesupport" => Family::Dev,
        // System family
        "user-logs" | "user-caches" | "trash" => Family::System,
        other => panic!("unknown category id: {}", other),
    }
}

pub trait CleanProvider {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn risk(&self) -> RiskLevel;

    /// Process name that should NOT be running before discover/execute.
    /// `None` means no app gating.
    fn requires_app_quit(&self) -> Option<&'static str> {
        None
    }

    /// Returns false when the provider is fundamentally unavailable on this
    /// system (e.g. CLI not installed). A provider that returns true here may
    /// still legitimately discover zero items.
    fn available(&self) -> bool {
        true
    }

    fn discover(&self) -> Result<Vec<CleanItem>>;

    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport>;
}

/// Returns every provider in canonical id order. Filtering by risk level,
/// `--category`, and runtime availability happens in `discover.rs`.
///
/// Takes `&CleanOpts` so providers needing tunables (idle_days, search
/// roots, ...) can read them at construction. M0 providers don't yet, but
/// the param exists so M1+ can land without re-threading callers.
pub fn all_providers(_opts: &crate::cli::CleanOpts) -> Vec<Box<dyn CleanProvider>> {
    vec![
        Box::new(user_logs::UserLogs),
        Box::new(xcode::XcodeDerivedData),
        Box::new(user_caches::UserCaches),
        Box::new(xcode::XcodeArchives),
        Box::new(xcode::XcodeDeviceSupport),
        Box::new(dev_caches::CargoCache),
        Box::new(dev_caches::NpmCache),
        Box::new(dev_caches::PnpmStore),
        Box::new(dev_caches::YarnCache),
        Box::new(gradle_maven::GradleMaven),
        Box::new(jetbrains::JetBrains),
        Box::new(vscode::VsCode),
        Box::new(trash::TrashProvider),
    ]
}

/// Canonical list of category ids accepted by `--category`.
pub fn known_category_ids() -> &'static [&'static str] {
    &[
        "user-logs",
        "xcode-derived",
        "user-caches",
        "xcode-archives",
        "xcode-devicesupport",
        "cargo",
        "npm",
        "pnpm",
        "yarn",
        "gradle-maven",
        "jetbrains",
        "vscode",
        "trash",
    ]
}

/// Lists immediate children of `root` as `CleanItem`s, sized via the
/// symlink-safe walk. Returns empty when `root` does not exist.
pub(crate) fn top_level_entries(
    root: &Path,
    category_id: &str,
    category_label: &str,
    risk: RiskLevel,
) -> Vec<CleanItem> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(root) {
        Ok(it) => it,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let size = dir_size_safe(&path);
        out.push(CleanItem {
            category_id: category_id.to_string(),
            category_label: category_label.to_string(),
            path,
            size,
            risk,
        });
    }
    out
}

/// Treats `root` itself as a single CleanItem (used for category-rooted
/// providers like xcode-derived). Returns empty when missing.
pub(crate) fn root_as_item(
    root: &Path,
    category_id: &str,
    category_label: &str,
    risk: RiskLevel,
) -> Vec<CleanItem> {
    if !root.exists() {
        return Vec::new();
    }
    let size = dir_size_safe(root);
    vec![CleanItem {
        category_id: category_id.to_string(),
        category_label: category_label.to_string(),
        path: root.to_path_buf(),
        size,
        risk,
    }]
}

/// Default per-item executor used by every provider except Trash. Rejects
/// `ExecAction::EmptyTrash`.
pub(crate) fn execute_per_item(
    items: &[CleanItem],
    action: ExecAction,
    provider_id: &'static str,
) -> Result<ExecReport> {
    if matches!(action, ExecAction::EmptyTrash) {
        return Err(anyhow!(
            "{} provider does not accept EmptyTrash",
            provider_id
        ));
    }
    let mut report = ExecReport::default();
    for item in items {
        let result = match action {
            ExecAction::Trash => move_to_trash(&item.path),
            ExecAction::HardDelete => remove_recursive_safe(&item.path)
                .map_err(|e| anyhow!("remove {}: {}", item.path.display(), e)),
            ExecAction::EmptyTrash => unreachable!(),
        };
        match result {
            Ok(()) => report.removed_paths.push(item.path.clone()),
            Err(e) => report.failed.push((item.path.clone(), e.to_string())),
        }
    }
    Ok(report)
}

/// Move a path to the user's Trash via Finder. Mirrors the helper in
/// `uninstall.rs` but lives here so the clean module is self-contained.
pub(crate) fn move_to_trash(path: &Path) -> Result<()> {
    let posix = path
        .to_str()
        .ok_or_else(|| anyhow!("non-utf8 path: {}", path.display()))?;
    let script = "on run argv\n\
                  tell application \"Finder\" to delete (POSIX file (item 1 of argv) as alias)\n\
                  end run";
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .arg(posix)
        .output()
        .with_context(|| format!("failed to spawn osascript for {}", path.display()))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow!("osascript failed for {}: {}", path.display(), err));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_known_category_has_family() {
        for id in known_category_ids() {
            // Must not panic. Result discarded; coverage is what we test.
            let _ = category_family(id);
        }
    }

    #[test]
    fn family_id_and_label_stable() {
        assert_eq!(Family::Dev.id(), "dev");
        assert_eq!(Family::UserStorage.id(), "user-storage");
        assert_eq!(Family::System.id(), "system");
        assert_eq!(Family::Dev.label(), "Dev caches");
        assert_eq!(Family::UserStorage.label(), "User storage");
        assert_eq!(Family::System.label(), "System leftovers");
    }

    #[test]
    #[should_panic(expected = "unknown category id")]
    fn category_family_panics_on_unknown() {
        let _ = category_family("definitely-not-a-category");
    }
}
