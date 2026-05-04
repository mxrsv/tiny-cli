use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;

use super::{execute_per_item, root_as_item, CleanProvider};
use crate::commands::clean::fs_safe::is_dir_safe;
use crate::commands::clean::process::{PgrepChecker, ProcessChecker};
use crate::commands::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "chat-caches";
const LABEL: &str = "Chat app caches";

const CHAT_PATHS: &[(&str, &str)] = &[
    ("Library/Application Support/Slack/Cache", "Slack"),
    (
        "Library/Application Support/Slack/Service Worker/CacheStorage",
        "Slack",
    ),
    ("Library/Application Support/discord/Cache", "Discord"),
];

/// Telegram path uses a glob pattern: `~/Library/Group Containers/
/// *.ru.keepcoder.Telegram/account-*/postbox/media`. We resolve via
/// read_dir at discover time. Locked by "Telegram".
const TELEGRAM_GROUP_CONTAINERS: &str = "Library/Group Containers";

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub struct ChatCaches {
    checker: Arc<dyn ProcessChecker>,
}

impl ChatCaches {
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

impl Default for ChatCaches {
    fn default() -> Self {
        Self::new()
    }
}

impl CleanProvider for ChatCaches {
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
        for (rel, app) in CHAT_PATHS {
            let path = h.join(rel);
            if !is_dir_safe(&path) {
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
        // Telegram: glob resolution.
        let telegram_running = self.checker.is_running("Telegram");
        for tg_media in telegram_media_dirs(&h.join(TELEGRAM_GROUP_CONTAINERS)) {
            if telegram_running {
                eprintln!(
                    "warn: skipping {} (app 'Telegram' is running)",
                    tg_media.display()
                );
                continue;
            }
            items.extend(root_as_item(&tg_media, ID, LABEL, RiskLevel::Review));
        }
        Ok(items)
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, ID)
    }
}

/// Resolves `<group_containers>/*.ru.keepcoder.Telegram/account-*/postbox/
/// media` for every matching account.
pub fn telegram_media_dirs(group_containers: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(group_containers) {
        Ok(it) => it,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if !name.ends_with(".ru.keepcoder.Telegram") {
            continue;
        }
        let accounts = match std::fs::read_dir(&path) {
            Ok(it) => it,
            Err(_) => continue,
        };
        for acc in accounts.flatten() {
            let acc_path = acc.path();
            let acc_name = match acc_path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };
            if !acc_name.starts_with("account-") {
                continue;
            }
            let media = acc_path.join("postbox/media");
            if is_dir_safe(&media) {
                out.push(media);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::clean::process::test_support::MockChecker;
    use crate::commands::clean::providers::known_category_ids;
    use std::fs;

    fn tempdir(label: &str) -> PathBuf {
        let mut base = std::env::temp_dir();
        base.push(format!(
            "tiny-clean-chat-{}-{}",
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
    fn chat_id_in_known_categories() {
        assert!(known_category_ids().contains(&ID));
    }

    #[test]
    fn chat_paths_skip_running_apps() {
        let mock = Arc::new(MockChecker::with_running(["Slack", "Discord", "Telegram"]));
        let p = ChatCaches::with_checker(mock);
        let items = p.discover().unwrap();
        for item in &items {
            let s = item.path.to_string_lossy();
            assert!(!s.contains("/Slack/"), "Slack path leaked: {}", s);
            assert!(!s.contains("/discord/"), "Discord path leaked: {}", s);
            assert!(
                !s.contains(".ru.keepcoder.Telegram"),
                "Telegram path leaked: {}",
                s
            );
        }
    }

    #[test]
    fn telegram_media_dirs_resolves_account_glob() {
        let root = tempdir("tg");
        let group = root
            .join("6N38VWS5BX.ru.keepcoder.Telegram")
            .join("account-12345");
        fs::create_dir_all(group.join("postbox/media")).unwrap();
        // Wrong-suffix container → must skip.
        fs::create_dir_all(root.join("other.app/account-1/postbox/media")).unwrap();
        // Wrong-prefix account → must skip.
        fs::create_dir_all(
            root.join("X.ru.keepcoder.Telegram/profile-1/postbox/media"),
        )
        .unwrap();
        let found = telegram_media_dirs(&root);
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("account-12345/postbox/media"));
        let _ = fs::remove_dir_all(&root);
    }
}
