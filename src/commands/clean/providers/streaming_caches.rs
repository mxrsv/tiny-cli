use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;

use super::{execute_per_item, root_as_item, CleanProvider};
use crate::commands::clean::process::{PgrepChecker, ProcessChecker};
use crate::commands::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "streaming-caches";
const LABEL: &str = "Streaming app caches";

/// (rel_path_under_home, app_name_locking_it). Skip path when its app is
/// running — Spotify especially keeps PersistentCache locked while open.
const STREAMING_PATHS: &[(&str, &str)] = &[
    ("Library/Caches/com.spotify.client", "Spotify"),
    (
        "Library/Application Support/Spotify/PersistentCache",
        "Spotify",
    ),
    (
        "Library/Containers/com.netflix.Netflix/Data/Library/Caches",
        "Netflix",
    ),
    (
        "Library/Containers/com.apple.Music/Data/Library/Caches",
        "Music",
    ),
];

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub struct StreamingCaches {
    checker: Arc<dyn ProcessChecker>,
}

impl StreamingCaches {
    pub fn new() -> Self {
        Self {
            checker: Arc::new(PgrepChecker),
        }
    }
    #[cfg(test)]
    pub fn with_checker(checker: Arc<dyn ProcessChecker>) -> Self {
        Self { checker }
    }
}

impl Default for StreamingCaches {
    fn default() -> Self {
        Self::new()
    }
}

impl CleanProvider for StreamingCaches {
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
        let mut items = Vec::new();
        for (rel, app) in STREAMING_PATHS {
            let path = h.join(rel);
            if !path.is_dir() {
                continue;
            }
            if self.checker.is_running(app) {
                eprintln!(
                    "warn: skipping {} (app '{}' is running)",
                    path.display(),
                    app
                );
                continue;
            }
            items.extend(root_as_item(&path, ID, LABEL, RiskLevel::Review));
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
    use crate::commands::clean::process::test_support::MockChecker;
    use crate::commands::clean::providers::known_category_ids;

    #[test]
    fn streaming_id_in_known_categories() {
        assert!(known_category_ids().contains(&ID));
    }

    #[test]
    fn streaming_paths_filtered_by_running_app() {
        // No paths exist on this machine necessarily, but we can prove
        // the gate logic by injecting a checker. Build minimum that we
        // can: assert that with Spotify running, discover() never
        // includes a Spotify-tagged CleanItem (size could be 0 if dir
        // missing, but the path string itself must not appear).
        let mock = Arc::new(MockChecker::with_running(["Spotify"]));
        let p = StreamingCaches::with_checker(mock);
        let items = p.discover().unwrap();
        for item in &items {
            let s = item.path.to_string_lossy();
            assert!(
                !s.contains("com.spotify.client") && !s.contains("Spotify/PersistentCache"),
                "Spotify path leaked while Spotify running: {}",
                s
            );
        }
    }
}
