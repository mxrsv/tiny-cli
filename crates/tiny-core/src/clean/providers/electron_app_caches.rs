use std::path::Path;

use crate::error::Result;

use super::{dir_roots_as_items, execute_per_item, CleanProvider};
use crate::clean::fs_safe::is_dir_safe;
use crate::clean::scan_context::ScanContext;
use crate::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "electron-app-caches";
const LABEL: &str = "Electron app caches";

const APP_SUPPORT: &str = "Library/Application Support";
/// Allow-listed apps: folder under Application Support and process name.
/// VS Code (`Code`), Slack and Discord have their own categories.
const APPS: &[(&str, &str)] = &[
    ("Notion", "Notion"),
    ("Figma", "Figma"),
    ("obsidian", "Obsidian"),
    ("Postman", "Postman"),
    ("Cursor", "Cursor"),
    ("Claude", "Claude"),
    ("Linear", "Linear"),
];
/// Chromium cache folders at the top of an app's folder. Never
/// `globalStorage`, `Local Storage`, `IndexedDB` or anything else.
const CACHE_DIRS: &[&str] = &["Cache", "Code Cache", "GPUCache"];

pub struct ElectronAppCaches;

impl ElectronAppCaches {
    pub(crate) fn discover_in(&self, ctx: &ScanContext<'_>, home: &Path) -> Vec<CleanItem> {
        let mut roots = Vec::new();
        for (folder, app) in APPS {
            let base = home.join(APP_SUPPORT).join(folder);
            if !is_dir_safe(&base) || ctx.known_running(app) {
                continue;
            }
            roots.extend(CACHE_DIRS.iter().map(|dir| base.join(dir)));
        }
        dir_roots_as_items(ctx, &roots, ID, LABEL, RiskLevel::Review)
    }

    fn owner(home: &Path, path: &Path) -> Option<&'static str> {
        APPS.iter()
            .find(|(folder, _)| path.parent() == Some(&home.join(APP_SUPPORT).join(folder)))
            .map(|(_, app)| *app)
    }
}

impl CleanProvider for ElectronAppCaches {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn inclusion_reason(&self) -> String {
        "Chromium cache folders of Notion, Figma, Obsidian and other listed apps (never their settings or stored data); the app recreates them"
            .into()
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn desktop_trash_paths(&self) -> bool {
        true
    }
    /// Gated per app in discovery and here, so one running app does not
    /// hide the others.
    fn item_apps(&self, path: &Path) -> Vec<String> {
        match super::home().and_then(|h| Self::owner(&h, path)) {
            Some(app) => vec![app.to_string()],
            None => APPS.iter().map(|(_, app)| app.to_string()).collect(),
        }
    }
    fn check_item(&self, path: &Path) -> std::result::Result<(), String> {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if CACHE_DIRS.contains(&name) {
            Ok(())
        } else {
            Err(format!("{} is not an app cache folder", path.display()))
        }
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
    use crate::clean::process::test_support::MockChecker;
    use crate::clean::providers::test_home::TestHome;
    use std::path::PathBuf;

    fn app_home(label: &str) -> TestHome {
        let home = TestHome::new(label);
        for dir in [
            "Cache",
            "Code Cache",
            "GPUCache",
            "IndexedDB",
            "Local Storage",
        ] {
            home.dir(&format!("Library/Application Support/Notion/{dir}"));
        }
        home.dir("Library/Application Support/Cursor/User/globalStorage");
        home.dir("Library/Application Support/Cursor/Cache");
        home.dir("Library/Application Support/Code/Cache");
        home.dir("Library/Application Support/SomeOtherApp/Cache");
        home
    }

    #[test]
    fn offers_only_allow_listed_cache_folders() {
        let home = app_home("electron");
        let items = ElectronAppCaches.discover_in(&ScanContext::unchecked(), home.path());
        assert_eq!(
            home.relative(&items),
            [
                "Library/Application Support/Notion/Cache",
                "Library/Application Support/Notion/Code Cache",
                "Library/Application Support/Notion/GPUCache",
                "Library/Application Support/Cursor/Cache",
            ]
        );
    }

    #[test]
    fn never_global_storage_or_site_data() {
        for (folder, _) in APPS {
            assert!(!["Code", "Slack", "discord"].contains(folder), "{folder}");
        }
        for dir in CACHE_DIRS {
            let lower = dir.to_lowercase();
            assert!(!lower.contains("storage") && !lower.contains("indexeddb"));
        }
        assert!(ElectronAppCaches
            .check_item(Path::new("/a/Cursor/User/globalStorage"))
            .is_err());
    }

    #[test]
    fn a_running_app_hides_only_its_own_caches() {
        let home = app_home("electron-running");
        let notion = MockChecker::with_running(["Notion"]);
        let ctx = ScanContext::new(None, &notion);
        let items = ElectronAppCaches.discover_in(&ctx, home.path());
        assert_eq!(
            home.relative(&items),
            ["Library/Application Support/Cursor/Cache"]
        );
    }

    #[test]
    fn item_apps_names_the_owning_app() {
        let h = PathBuf::from(std::env::var_os("HOME").unwrap());
        let figma = h.join(APP_SUPPORT).join("Figma/GPUCache");
        assert_eq!(ElectronAppCaches.item_apps(&figma), vec!["Figma"]);
        assert_eq!(
            ElectronAppCaches.item_apps(Path::new("/x/Cache")).len(),
            APPS.len()
        );
    }
}
