use std::path::{Path, PathBuf};

use crate::error::Result;

use super::{dir_roots_as_items, execute_per_item, CleanProvider};
use crate::clean::scan_context::ScanContext;
use crate::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "js-tool-caches";
const LABEL: &str = "JavaScript tool caches";

/// Children of `~/Library/Caches` this category owns; `user-caches` skips them.
pub(crate) const LIBRARY_CACHES: &[&str] = &["electron", "node-gyp", "deno"];
const BUN_CACHE: &str = ".bun/install/cache";

pub struct JsToolCaches;

impl JsToolCaches {
    fn roots(home: &Path) -> Vec<PathBuf> {
        let caches = home.join("Library/Caches");
        LIBRARY_CACHES
            .iter()
            .map(|name| caches.join(name))
            .chain([home.join(BUN_CACHE)])
            .collect()
    }

    pub(crate) fn discover_in(&self, ctx: &ScanContext<'_>, home: &Path) -> Vec<CleanItem> {
        dir_roots_as_items(ctx, &Self::roots(home), ID, LABEL, RiskLevel::Safe)
    }
}

impl CleanProvider for JsToolCaches {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn inclusion_reason(&self) -> String {
        "Downloads cached by Electron, node-gyp, Deno and Bun; the tools fetch them again".into()
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
    fn offers_the_tool_caches_and_only_bun_s_cache_folder() {
        let home = TestHome::new("js-tool-caches");
        home.dir("Library/Caches/electron/v30");
        home.dir("Library/Caches/node-gyp/20.0.0");
        home.dir("Library/Caches/deno/deps");
        home.dir(".bun/install/cache/react@18");
        home.file(".bun/bin/bun", 4);
        let items = JsToolCaches.discover_in(&ScanContext::unchecked(), home.path());
        assert_eq!(
            home.relative(&items),
            [
                "Library/Caches/electron",
                "Library/Caches/node-gyp",
                "Library/Caches/deno",
                ".bun/install/cache"
            ]
        );
    }
}
