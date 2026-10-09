use std::path::{Path, PathBuf};

use crate::error::Result;

use super::{dir_roots_as_items, execute_per_item, CleanProvider};
use crate::clean::fs_safe::{is_dir_safe, list_children};
use crate::clean::scan_context::ScanContext;
use crate::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "browser-profiles";
const LABEL: &str = "Browser profile caches";

const APP_SUPPORT: &str = "Library/Application Support";
/// Chromium user-data folders under Application Support and the app that
/// owns each.
const BROWSERS: &[(&str, &str)] = &[
    ("Google/Chrome", "Google Chrome"),
    ("BraveSoftware/Brave-Browser", "Brave Browser"),
    ("Microsoft Edge", "Microsoft Edge"),
    ("Arc/User Data", "Arc"),
];
/// The only folders taken from a profile. Cookies, history, Local Storage,
/// IndexedDB and everything else in the profile stay.
const CACHE_DIRS: &[&str] = &["Cache", "Code Cache"];

pub struct BrowserProfiles;

impl BrowserProfiles {
    /// `Default`, `Profile N` and `Guest Profile`; `System Profile` and
    /// non-profile folders (`Crashpad`, `Safe Browsing`, ...) are skipped.
    fn is_profile(name: &str) -> bool {
        name == "Default" || name == "Guest Profile" || name.starts_with("Profile ")
    }

    pub(crate) fn discover_in(&self, ctx: &ScanContext<'_>, home: &Path) -> Vec<CleanItem> {
        // `browser-caches` already offers these.
        let taken: Vec<PathBuf> = super::browser_caches::BROWSER_CACHE_PATHS
            .iter()
            .map(|(rel, _)| home.join(rel))
            .collect();
        let mut roots = Vec::new();
        for (rel, app) in BROWSERS {
            let user_data = home.join(APP_SUPPORT).join(rel);
            if !is_dir_safe(&user_data) || ctx.known_running(app) {
                continue;
            }
            for profile in list_children(&user_data, ctx) {
                let is_profile = profile
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(Self::is_profile);
                if !is_profile || !is_dir_safe(&profile) {
                    continue;
                }
                roots.extend(
                    CACHE_DIRS
                        .iter()
                        .map(|dir| profile.join(dir))
                        .filter(|path| !taken.contains(path)),
                );
            }
        }
        dir_roots_as_items(ctx, &roots, ID, LABEL, RiskLevel::Review)
    }

    fn owner(home: &Path, path: &Path) -> Option<&'static str> {
        BROWSERS
            .iter()
            .find(|(rel, _)| path.starts_with(home.join(APP_SUPPORT).join(rel)))
            .map(|(_, app)| *app)
    }
}

impl CleanProvider for BrowserProfiles {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn inclusion_reason(&self) -> String {
        "Cache and Code Cache folders inside Chrome, Brave, Edge and Arc profiles (never cookies, history or site data); the browser recreates them"
            .into()
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn desktop_trash_paths(&self) -> bool {
        true
    }
    /// Gated per browser in discovery and here, so one running browser
    /// does not hide the others.
    fn item_apps(&self, path: &Path) -> Vec<String> {
        let owner = super::home().and_then(|h| Self::owner(&h, path));
        // An unrecognised path is gated on every browser rather than none.
        match owner {
            Some(app) => vec![app.to_string()],
            None => BROWSERS.iter().map(|(_, app)| app.to_string()).collect(),
        }
    }
    fn check_item(&self, path: &Path) -> std::result::Result<(), String> {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if CACHE_DIRS.contains(&name) {
            Ok(())
        } else {
            Err(format!("{} is not a browser cache folder", path.display()))
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

    fn browser_home(label: &str) -> TestHome {
        let home = TestHome::new(label);
        let chrome = "Library/Application Support/Google/Chrome";
        for profile in ["Default", "Profile 2", "System Profile"] {
            for dir in ["Cache", "Code Cache", "IndexedDB", "Local Storage"] {
                home.dir(&format!("{chrome}/{profile}/{dir}"));
            }
            home.file(&format!("{chrome}/{profile}/Cookies"), 1);
            home.file(&format!("{chrome}/{profile}/History"), 1);
        }
        home.dir(&format!("{chrome}/Crashpad/Cache"));
        home.dir("Library/Application Support/BraveSoftware/Brave-Browser/Default/Code Cache");
        home.dir("Library/Application Support/Arc/User Data/Default/Cache");
        home.dir("Library/Application Support/Arc/User Data/Default/Code Cache");
        home
    }

    #[test]
    fn offers_only_cache_folders_of_real_profiles() {
        let home = browser_home("browser-profiles");
        let items = BrowserProfiles.discover_in(&ScanContext::unchecked(), home.path());
        let mut found = home.relative(&items);
        found.sort();
        assert_eq!(
            found,
            [
                // Chrome and Arc `Default/Cache` belong to `browser-caches`.
                "Library/Application Support/Arc/User Data/Default/Code Cache",
                "Library/Application Support/BraveSoftware/Brave-Browser/Default/Code Cache",
                "Library/Application Support/Google/Chrome/Default/Code Cache",
                "Library/Application Support/Google/Chrome/Profile 2/Cache",
                "Library/Application Support/Google/Chrome/Profile 2/Code Cache",
            ]
        );
    }

    #[test]
    fn never_offers_profile_data() {
        let home = browser_home("browser-profiles-data");
        let items = BrowserProfiles.discover_in(&ScanContext::unchecked(), home.path());
        for item in &items {
            let name = item.path.file_name().unwrap().to_string_lossy();
            assert!(CACHE_DIRS.contains(&name.as_ref()), "{name}");
        }
        for forbidden in [
            "Cookies",
            "History",
            "Local Storage",
            "IndexedDB",
            "Default",
        ] {
            assert!(BrowserProfiles.check_item(Path::new(forbidden)).is_err());
        }
    }

    #[test]
    fn a_running_browser_hides_only_its_own_profiles() {
        let home = browser_home("browser-profiles-running");
        let chrome = MockChecker::with_running(["Google Chrome"]);
        let ctx = ScanContext::new(None, &chrome);
        let items = BrowserProfiles.discover_in(&ctx, home.path());
        assert!(!items.is_empty());
        for item in &items {
            assert!(!item.path.to_string_lossy().contains("/Google/Chrome/"));
        }
    }

    #[test]
    fn item_apps_names_the_owning_browser() {
        let h = PathBuf::from(std::env::var_os("HOME").unwrap());
        let brave = h
            .join(APP_SUPPORT)
            .join("BraveSoftware/Brave-Browser/Default/Cache");
        assert_eq!(BrowserProfiles.item_apps(&brave), vec!["Brave Browser"]);
        assert_eq!(
            BrowserProfiles.item_apps(Path::new("/x")).len(),
            BROWSERS.len()
        );
    }
}
