//! Detect leftover `~/Library/Application Support/<bundle-id>` directories
//! whose owning app has been uninstalled.
//!
//! Approach: build a set of installed bundle IDs via Spotlight
//! (`mdfind kMDItemContentType == 'com.apple.application-bundle'`), then
//! flag any direct child of `Application Support/` whose name doesn't
//! match. Risk is **always Review** — false positives are inevitable
//! because some apps store their data under a different key than their
//! bundle ID.
//!
//! Spotlight tắt → `mdfind` returns 0 lines → we MUST refuse to discover
//! (otherwise every dir would look orphan). The handoff calls this out
//! explicitly: "if mdfind returns 0 result, provider refuse discover".

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;

use super::{execute_per_item, top_level_entries, CleanProvider};
use crate::commands::clean::runner::{CommandRunner, RealRunner};
use crate::commands::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "app-orphans";
const LABEL: &str = "Orphaned Application Support dirs";

const MDFIND_QUERY: &str = "kMDItemContentType == 'com.apple.application-bundle'";
const APP_SUPPORT: &str = "Library/Application Support";

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub struct AppOrphans {
    runner: Arc<dyn CommandRunner>,
}

impl AppOrphans {
    pub fn new() -> Self {
        Self {
            runner: Arc::new(RealRunner),
        }
    }
    #[cfg(test)]
    pub fn with_runner(runner: Arc<dyn CommandRunner>) -> Self {
        Self { runner }
    }

    /// Builds the set of `CFBundleIdentifier` values for every `.app`
    /// bundle Spotlight knows about. Empty result → caller must refuse
    /// to flag anything.
    fn installed_bundle_ids(&self) -> HashSet<String> {
        let out = self.runner.run("mdfind", &[MDFIND_QUERY]);
        if !out.success {
            return HashSet::new();
        }
        let mut ids: HashSet<String> = HashSet::new();
        for line in out.stdout.lines() {
            let app_path = line.trim();
            if app_path.is_empty() {
                continue;
            }
            // `defaults read <app>/Contents/Info CFBundleIdentifier`
            // (no .plist suffix — defaults reads .plist by convention).
            let plist_arg = format!("{}/Contents/Info", app_path);
            let id_out = self
                .runner
                .run("defaults", &["read", &plist_arg, "CFBundleIdentifier"]);
            if !id_out.success {
                continue;
            }
            let id = id_out.stdout.trim();
            if !id.is_empty() {
                ids.insert(id.to_string());
            }
        }
        ids
    }
}

impl Default for AppOrphans {
    fn default() -> Self {
        Self::new()
    }
}

impl CleanProvider for AppOrphans {
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
        let h = match home() {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        let installed = self.installed_bundle_ids();
        if installed.is_empty() {
            // Spotlight off / not indexed yet → refuse rather than flag everything.
            eprintln!(
                "warn: mdfind returned 0 .app bundles; Spotlight likely off. \
                 app_orphans skipped to avoid false positives."
            );
            return Ok(Vec::new());
        }
        let support_dir = h.join(APP_SUPPORT);
        let candidates = top_level_entries(&support_dir, ID, LABEL, RiskLevel::Review);
        // Keep only entries whose dir name doesn't match any installed bundle id.
        let orphans: Vec<CleanItem> = candidates
            .into_iter()
            .filter(|item| {
                let name = match item.path.file_name().and_then(|n| n.to_str()) {
                    Some(n) => n,
                    None => return false,
                };
                !installed.contains(name)
            })
            .collect();
        Ok(orphans)
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, ID)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::clean::providers::known_category_ids;
    use crate::commands::clean::runner::test_support::MockRunner;

    #[test]
    fn app_orphans_id_in_known_categories() {
        assert!(known_category_ids().contains(&ID));
    }

    #[test]
    fn app_orphans_review_risk() {
        assert_eq!(AppOrphans::new().risk(), RiskLevel::Review);
    }

    #[test]
    fn empty_mdfind_result_returns_empty_safely() {
        // mdfind returns success but zero lines → installed set empty →
        // discover() must return empty (not flag everything in Application Support).
        let runner = Arc::new(MockRunner::new().with_response("mdfind", &[MDFIND_QUERY], true, ""));
        let p = AppOrphans::with_runner(runner);
        let items = p.discover().unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn mdfind_failure_returns_empty_safely() {
        // mdfind exits non-zero → also empty.
        let runner = Arc::new(MockRunner::new().with_response("mdfind", &[MDFIND_QUERY], false, ""));
        let p = AppOrphans::with_runner(runner);
        let items = p.discover().unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn installed_bundle_ids_collects_from_each_app() {
        let runner = Arc::new(
            MockRunner::new()
                .with_response(
                    "mdfind",
                    &[MDFIND_QUERY],
                    true,
                    "/Applications/Safari.app\n/Applications/Music.app\n",
                )
                .with_response(
                    "defaults",
                    &["read", "/Applications/Safari.app/Contents/Info", "CFBundleIdentifier"],
                    true,
                    "com.apple.Safari\n",
                )
                .with_response(
                    "defaults",
                    &["read", "/Applications/Music.app/Contents/Info", "CFBundleIdentifier"],
                    true,
                    "com.apple.Music\n",
                ),
        );
        let p = AppOrphans::with_runner(runner);
        let ids = p.installed_bundle_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains("com.apple.Safari"));
        assert!(ids.contains("com.apple.Music"));
    }

    #[test]
    fn installed_bundle_ids_skips_apps_with_failed_defaults() {
        // One app's defaults call fails — that app is silently skipped,
        // others still collected.
        let runner = Arc::new(
            MockRunner::new()
                .with_response(
                    "mdfind",
                    &[MDFIND_QUERY],
                    true,
                    "/Applications/Good.app\n/Applications/Bad.app\n",
                )
                .with_response(
                    "defaults",
                    &["read", "/Applications/Good.app/Contents/Info", "CFBundleIdentifier"],
                    true,
                    "com.example.good",
                )
                .with_response(
                    "defaults",
                    &["read", "/Applications/Bad.app/Contents/Info", "CFBundleIdentifier"],
                    false,
                    "",
                ),
        );
        let p = AppOrphans::with_runner(runner);
        let ids = p.installed_bundle_ids();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains("com.example.good"));
    }
}
