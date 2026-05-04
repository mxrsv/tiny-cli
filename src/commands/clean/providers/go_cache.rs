use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;

use super::{execute_per_item, root_as_item, CleanProvider};
use crate::commands::clean::runner::{CommandRunner, RealRunner};
use crate::commands::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "go-cache";
const LABEL: &str = "Go build/mod caches";

pub struct GoCache {
    runner: Arc<dyn CommandRunner>,
}

impl GoCache {
    pub fn new() -> Self {
        Self {
            runner: Arc::new(RealRunner),
        }
    }
    #[cfg(test)]
    pub fn with_runner(runner: Arc<dyn CommandRunner>) -> Self {
        Self { runner }
    }
}

impl Default for GoCache {
    fn default() -> Self {
        Self::new()
    }
}

impl CleanProvider for GoCache {
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
        self.runner.which("go")
    }
    fn discover(&self) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();
        for var in ["GOCACHE", "GOMODCACHE"] {
            let out = self.runner.run("go", &["env", var]);
            if !out.success {
                continue;
            }
            let path = out.stdout.trim();
            if path.is_empty() {
                continue;
            }
            items.extend(root_as_item(
                &PathBuf::from(path),
                ID,
                LABEL,
                RiskLevel::Review,
            ));
        }
        Ok(items)
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
    fn go_cache_id_in_known_categories() {
        assert!(known_category_ids().contains(&ID));
    }

    #[test]
    fn go_cache_unavailable_when_cli_missing() {
        let runner = Arc::new(MockRunner::new()); // no which("go")
        let p = GoCache::with_runner(runner);
        assert!(!p.available());
    }

    #[test]
    fn go_cache_available_when_which_succeeds() {
        let runner = Arc::new(MockRunner::new().with_which("go"));
        let p = GoCache::with_runner(runner);
        assert!(p.available());
    }

    #[test]
    fn go_cache_skips_var_when_subprocess_fails() {
        // GOCACHE returns success+empty path → skip; GOMODCACHE bombs → skip.
        // discover() must not panic, returns empty vec.
        let runner = Arc::new(
            MockRunner::new()
                .with_which("go")
                .with_response("go", &["env", "GOCACHE"], true, "")
                .with_response("go", &["env", "GOMODCACHE"], false, ""),
        );
        let p = GoCache::with_runner(runner);
        let items = p.discover().unwrap();
        assert!(items.is_empty());
    }
}
