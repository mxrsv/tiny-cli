use std::path::{Path, PathBuf};

use crate::error::Result;

use super::{dir_roots_as_items, execute_per_item, CleanProvider};
use crate::clean::scan_context::ScanContext;
use crate::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "swift-packages";
const LABEL: &str = "Swift package caches";
const APP: &str = "Xcode";

/// Children of `~/Library/Caches` this category owns; `user-caches` skips them.
/// `~/Library/org.swift.swiftpm` is left alone: it holds SwiftPM's
/// configuration (mirrors, registries) and security fingerprints, not cache.
pub(crate) const LIBRARY_CACHES: &[&str] =
    &["org.swift.swiftpm", "CocoaPods", "org.carthage.CarthageKit"];

pub struct SwiftPackages;

impl SwiftPackages {
    fn roots(home: &Path) -> Vec<PathBuf> {
        let caches = home.join("Library/Caches");
        LIBRARY_CACHES
            .iter()
            .map(|name| caches.join(name))
            .collect()
    }

    pub(crate) fn discover_in(&self, ctx: &ScanContext<'_>, home: &Path) -> Vec<CleanItem> {
        dir_roots_as_items(ctx, &Self::roots(home), ID, LABEL, RiskLevel::Safe)
    }
}

impl CleanProvider for SwiftPackages {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn inclusion_reason(&self) -> String {
        "Package downloads SwiftPM, CocoaPods and Carthage fetch again when a project resolves"
            .into()
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Safe
    }
    fn desktop_trash_paths(&self) -> bool {
        true
    }
    fn requires_app_quit(&self) -> Option<&'static str> {
        Some(APP)
    }
    fn discover(&self, ctx: &ScanContext<'_>) -> Result<Vec<CleanItem>> {
        Ok(super::home()
            .map(|h| self.discover_in(ctx, &h))
            .unwrap_or_default())
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, ID)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clean::providers::test_home::TestHome;

    #[test]
    fn offers_the_package_caches_but_never_swiftpm_configuration() {
        let home = TestHome::new("swift-packages");
        home.dir("Library/Caches/org.swift.swiftpm/repositories");
        home.dir("Library/Caches/CocoaPods/Pods");
        home.dir("Library/Caches/org.carthage.CarthageKit/dependencies");
        home.file("Library/org.swift.swiftpm/configuration/mirrors.json", 2);
        home.dir("Library/org.swift.swiftpm/security/fingerprints");
        let items = SwiftPackages.discover_in(&ScanContext::unchecked(), home.path());
        assert_eq!(
            home.relative(&items),
            [
                "Library/Caches/org.swift.swiftpm",
                "Library/Caches/CocoaPods",
                "Library/Caches/org.carthage.CarthageKit"
            ]
        );
    }

    #[test]
    fn waits_for_xcode_to_quit() {
        assert_eq!(SwiftPackages.quit_apps(), vec!["Xcode"]);
    }
}
