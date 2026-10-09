use std::path::Path;

use crate::error::Result;

use super::{dir_roots_as_items, execute_per_item, CleanProvider};
use crate::clean::scan_context::ScanContext;
use crate::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "homebrew-cache";
const LABEL: &str = "Homebrew downloads";

/// Children of `~/Library/Caches` this category owns; `user-caches` skips them.
/// Old kegs in the Cellar are not offered: only `brew cleanup` removes them
/// without breaking links.
pub(crate) const LIBRARY_CACHES: &[&str] = &["Homebrew"];

pub struct HomebrewCache;

impl HomebrewCache {
    pub(crate) fn discover_in(&self, ctx: &ScanContext<'_>, home: &Path) -> Vec<CleanItem> {
        let caches = home.join("Library/Caches");
        let roots: Vec<_> = LIBRARY_CACHES.iter().map(|n| caches.join(n)).collect();
        dir_roots_as_items(ctx, &roots, ID, LABEL, RiskLevel::Safe)
    }
}

impl CleanProvider for HomebrewCache {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn inclusion_reason(&self) -> String {
        "Bottles and source archives Homebrew already installed; it downloads them again if needed"
            .into()
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Safe
    }
    fn desktop_trash_paths(&self) -> bool {
        true
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
    fn offers_the_download_cache_as_one_item() {
        let home = TestHome::new("homebrew-cache");
        home.file(
            "Library/Caches/Homebrew/downloads/abc--wget.bottle.tar.gz",
            7,
        );
        home.file("Library/Caches/Homebrew/api/formula.jws.json", 3);
        let items = HomebrewCache.discover_in(&ScanContext::unchecked(), home.path());
        assert_eq!(home.relative(&items), ["Library/Caches/Homebrew"]);
        assert_eq!(items[0].size, 10);
    }

    #[test]
    fn a_missing_cache_offers_nothing() {
        let home = TestHome::new("homebrew-none");
        assert!(HomebrewCache
            .discover_in(&ScanContext::unchecked(), home.path())
            .is_empty());
    }
}
