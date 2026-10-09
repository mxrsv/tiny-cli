use std::path::{Path, PathBuf};

use crate::error::Result;

use super::{dir_roots_as_items, execute_per_item, CleanProvider};
use crate::clean::scan_context::ScanContext;
use crate::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "browser-automation";
const LABEL: &str = "Browser automation downloads";

/// Children of `~/Library/Caches` this category owns; `user-caches` skips them.
pub(crate) const LIBRARY_CACHES: &[&str] = &["ms-playwright", "ms-playwright-mcp", "Cypress"];
/// Under `~/.cache`.
const DOT_CACHE: &[&str] = &["puppeteer", "selenium"];

pub struct BrowserAutomation;

impl BrowserAutomation {
    fn roots(home: &Path) -> Vec<PathBuf> {
        let caches = home.join("Library/Caches");
        let dot_cache = home.join(".cache");
        LIBRARY_CACHES
            .iter()
            .map(|name| caches.join(name))
            .chain(DOT_CACHE.iter().map(|name| dot_cache.join(name)))
            .collect()
    }

    pub(crate) fn discover_in(&self, ctx: &ScanContext<'_>, home: &Path) -> Vec<CleanItem> {
        dir_roots_as_items(ctx, &Self::roots(home), ID, LABEL, RiskLevel::Safe)
    }
}

impl CleanProvider for BrowserAutomation {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn inclusion_reason(&self) -> String {
        "Browsers downloaded by Playwright, Cypress, Puppeteer and Selenium; their install command fetches them again"
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
    fn offers_each_tool_folder_that_exists_and_skips_symlinks() {
        let home = TestHome::new("browser-automation");
        home.dir("Library/Caches/ms-playwright/chromium-1140");
        home.file("Library/Caches/ms-playwright/chromium-1140/chrome", 10);
        home.dir("Library/Caches/Cypress/13.0.0");
        home.dir(".cache/puppeteer/chrome");
        home.symlink("Library/Caches/ms-playwright-mcp", ".cache/puppeteer");
        let items = BrowserAutomation.discover_in(&ScanContext::unchecked(), home.path());
        assert_eq!(
            home.relative(&items),
            [
                "Library/Caches/ms-playwright",
                "Library/Caches/Cypress",
                ".cache/puppeteer"
            ]
        );
        assert_eq!(items[0].size, 10);
    }
}
