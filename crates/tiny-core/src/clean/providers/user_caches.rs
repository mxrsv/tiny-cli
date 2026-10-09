use crate::error::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::{execute_per_item, run_tool, CleanProvider};
use crate::clean::fs_safe::{dir_size_checked, list_children};
use crate::clean::scan_context::ScanContext;
use crate::clean::types::{CleanItem, Evidence, ExecAction, ExecReport, RiskLevel};
use crate::runner::CommandRunner;

const ID: &str = "user-caches";
const LABEL: &str = "User caches";

/// Absolute paths: a Finder-launched app has a minimal `PATH`.
const MDFIND: &str = "/usr/bin/mdfind";
const PLUTIL: &str = "/usr/bin/plutil";
const BUNDLE_ID_ATTR: &str = "kMDItemCFBundleIdentifier";
const APPLE_PREFIX: &str = "com.apple.";

pub struct UserCaches {
    runner: Arc<dyn CommandRunner>,
    /// Bundle id -> executable name of the app that owns it (`None`: no app
    /// resolved). Filled once per discovery, so `item_apps` at execution
    /// time does not spawn again.
    owners: Mutex<HashMap<String, Option<String>>>,
}

impl UserCaches {
    pub fn with_runner(runner: Arc<dyn CommandRunner>) -> Self {
        Self {
            runner,
            owners: Mutex::new(HashMap::new()),
        }
    }

    fn cached_owner(&self, id: &str) -> Option<Option<String>> {
        let owners = self.owners.lock().unwrap_or_else(|e| e.into_inner());
        owners.get(id).cloned()
    }

    /// Resolves every id with one Spotlight query plus one `plutil` per
    /// matched app, and replaces their cache entries. A failed tool leaves
    /// the affected ids unresolved; it never fails the category.
    fn resolve(&self, ids: &[&str]) {
        let mut found = self.find_apps(ids);
        let mut owners = self.owners.lock().unwrap_or_else(|e| e.into_inner());
        for id in ids {
            let apps = found.remove(*id).unwrap_or_default();
            let exe = apps.iter().find_map(|app| self.executable_of(app));
            owners.insert((*id).to_string(), exe);
        }
    }

    /// Bundle id -> paths of the `.app` bundles Spotlight lists for it.
    /// `ids` are validated by the caller, so they are safe to put in the query.
    fn find_apps(&self, ids: &[&str]) -> HashMap<String, Vec<String>> {
        let mut found: HashMap<String, Vec<String>> = HashMap::new();
        if ids.is_empty() {
            return found;
        }
        let query = ids
            .iter()
            .map(|id| format!("{BUNDLE_ID_ATTR} == '{id}'"))
            .collect::<Vec<_>>()
            .join(" || ");
        let out = match run_tool(
            self.runner.as_ref(),
            MDFIND,
            &["-attr", BUNDLE_ID_ATTR, &query],
        ) {
            Ok(out) if out.success() => out,
            _ => return found,
        };
        let marker = format!(" {BUNDLE_ID_ATTR} = ");
        for line in out.stdout.lines() {
            let Some((app, id)) = line.rsplit_once(&marker) else {
                continue;
            };
            let id = id.trim();
            if ids.contains(&id) {
                found
                    .entry(id.to_string())
                    .or_default()
                    .push(app.trim().to_string());
            }
        }
        found
    }

    /// `CFBundleExecutable`: the process name `pgrep -x` matches (VS Code is
    /// `Code`, not `VSCode`).
    fn executable_of(&self, app: &str) -> Option<String> {
        let plist = format!("{app}/Contents/Info.plist");
        let out = run_tool(
            self.runner.as_ref(),
            PLUTIL,
            &["-extract", "CFBundleExecutable", "raw", "-o", "-", &plist],
        )
        .ok()?;
        let exe = out.stdout.trim();
        (out.success() && !exe.is_empty()).then(|| exe.to_string())
    }

    /// The resolved executable for a cache folder name, resolving lazily
    /// when discovery has not seen it (a fresh provider at execution time).
    fn owner(&self, name: &str) -> Option<String> {
        if !is_bundle_id(name) {
            return None;
        }
        if let Some(cached) = self.cached_owner(name) {
            return cached;
        }
        self.resolve(&[name]);
        self.cached_owner(name).flatten()
    }

    /// Process names to check for a cache folder: the resolved executable,
    /// else the last-segment guess so the gate is never empty.
    fn gate_apps(&self, name: &str) -> Vec<String> {
        self.owner(name)
            .or_else(|| guess_app(name).map(String::from))
            .into_iter()
            .collect()
    }

    fn list_caches(&self, ctx: &ScanContext<'_>, root: &Path) -> Result<Vec<CleanItem>> {
        let paths: Vec<(PathBuf, String)> = list_children(root, ctx)
            .into_iter()
            .filter_map(|path| {
                let name = path.file_name()?.to_str()?.to_string();
                (!is_apple_cache(&name)).then_some((path, name))
            })
            .collect();
        let ids: Vec<&str> = paths
            .iter()
            .map(|(_, name)| name.as_str())
            .filter(|name| is_bundle_id(name))
            .collect();
        self.resolve(&ids);
        let mut out = Vec::new();
        for (path, name) in paths {
            if self.gate_apps(&name).iter().any(|a| ctx.known_running(a)) {
                continue;
            }
            let evidence = match self.owner(&name) {
                Some(name) => Evidence::OwningApp { name },
                None => Evidence::OwningAppUnknown,
            };
            let size = dir_size_checked(&path, ctx);
            out.push(CleanItem {
                category_id: ID.to_string(),
                category_label: LABEL.to_string(),
                path,
                size,
                risk: RiskLevel::Review,
                evidence: vec![evidence],
            });
        }
        Ok(out)
    }
}

impl CleanProvider for UserCaches {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn inclusion_reason(&self) -> String {
        "Per-app caches in ~/Library/Caches that apps rebuild".into()
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn desktop_trash_paths(&self) -> bool {
        true
    }
    fn item_apps(&self, path: &Path) -> Vec<String> {
        match path.file_name().and_then(|s| s.to_str()) {
            Some(name) => self.gate_apps(name),
            None => Vec::new(),
        }
    }
    fn discover(&self, ctx: &ScanContext<'_>) -> Result<Vec<CleanItem>> {
        let h = match home() {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        self.list_caches(ctx, &h.join("Library/Caches"))
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, ID)
    }
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// System daemon caches (`com.apple.bird`, ...) are not the user's to clear.
fn is_apple_cache(name: &str) -> bool {
    name.get(..APPLE_PREFIX.len())
        .is_some_and(|p| p.eq_ignore_ascii_case(APPLE_PREFIX))
}

/// Looks like a reverse-DNS bundle id. Checked before the name goes into a
/// Spotlight query, so only `[A-Za-z0-9.-]` ever reaches it.
fn is_bundle_id(name: &str) -> bool {
    name.contains('.')
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
}

/// Fallback when no app resolves: the *last* dot segment of the folder name
/// (`pgrep -x` target) — best-effort, not airtight.
fn guess_app(name: &str) -> Option<&str> {
    let last_segment = name.rsplit('.').next().unwrap_or(name);
    (!last_segment.is_empty()).then_some(last_segment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clean::process::test_support::MockChecker;
    use crate::runner::test_support::MockRunner;

    fn fixture(label: &str, names: &[&str]) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("tiny-user-caches-{label}-{}", std::process::id()));
        for name in names {
            std::fs::create_dir_all(root.join(name)).unwrap();
        }
        root
    }

    fn cleanup(root: &Path) {
        let _ = crate::clean::fs_safe::remove_recursive_safe(root);
    }

    /// Mocks the batched query and one `plutil` read for VS Code.
    fn vscode_runner() -> MockRunner {
        let query = "kMDItemCFBundleIdentifier == 'com.microsoft.VSCode'";
        MockRunner::new()
            .with_exit(
                MDFIND,
                &["-attr", "kMDItemCFBundleIdentifier", query],
                0,
                "/Applications/Visual Studio Code.app   kMDItemCFBundleIdentifier = com.microsoft.VSCode\n",
            )
            .with_exit(
                PLUTIL,
                &[
                    "-extract",
                    "CFBundleExecutable",
                    "raw",
                    "-o",
                    "-",
                    "/Applications/Visual Studio Code.app/Contents/Info.plist",
                ],
                0,
                "Code\n",
            )
    }

    fn provider(runner: MockRunner) -> UserCaches {
        UserCaches::with_runner(Arc::new(runner))
    }

    fn names(items: &[CleanItem]) -> Vec<String> {
        items
            .iter()
            .map(|i| i.path.file_name().unwrap().to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn apple_daemon_caches_are_never_offered() {
        let root = fixture("apple", &["com.apple.bird", "COM.APPLE.passd", "Homebrew"]);
        let m = MockChecker::none();
        let ctx = ScanContext::new(None, &m);
        let items = provider(MockRunner::new())
            .list_caches(&ctx, &root)
            .unwrap();
        assert_eq!(names(&items), vec!["Homebrew"]);
        cleanup(&root);
    }

    #[test]
    fn a_resolved_executable_gates_the_cache_while_the_app_runs() {
        let root = fixture("running", &["com.microsoft.VSCode"]);
        let m = MockChecker::with_running(["Code"]);
        let ctx = ScanContext::new(None, &m);
        let items = provider(vscode_runner()).list_caches(&ctx, &root).unwrap();
        assert!(items.is_empty(), "Code is running");
        assert!(ctx.take_findings().roots.contains(&root));
        cleanup(&root);
    }

    #[test]
    fn a_resolved_app_that_is_idle_is_offered_with_its_executable() {
        let root = fixture("idle", &["com.microsoft.VSCode"]);
        let m = MockChecker::with_running(["VSCode"]);
        let ctx = ScanContext::new(None, &m);
        let items = provider(vscode_runner()).list_caches(&ctx, &root).unwrap();
        assert_eq!(names(&items), vec!["com.microsoft.VSCode"]);
        assert_eq!(
            items[0].evidence,
            vec![Evidence::OwningApp {
                name: "Code".into()
            }]
        );
        cleanup(&root);
    }

    #[test]
    fn an_unresolved_name_is_offered_unknown_and_gated_by_the_guess() {
        let root = fixture("unknown", &["com.example.Idle", "com.example.Busy"]);
        let m = MockChecker::with_running(["Busy"]);
        let ctx = ScanContext::new(None, &m);
        let items = provider(MockRunner::new())
            .list_caches(&ctx, &root)
            .unwrap();
        assert_eq!(names(&items), vec!["com.example.Idle"]);
        assert_eq!(items[0].evidence, vec![Evidence::OwningAppUnknown]);
        cleanup(&root);
    }

    #[test]
    fn failed_tools_fall_back_to_the_guess_without_failing_the_category() {
        let root = fixture("fail", &["com.microsoft.VSCode"]);
        let query = "kMDItemCFBundleIdentifier == 'com.microsoft.VSCode'";
        let timeout = crate::runner::CommandError::Timeout {
            bin: MDFIND.into(),
            timeout_ms: 1,
        };
        let runner = MockRunner::new().with_output(
            MDFIND,
            &["-attr", "kMDItemCFBundleIdentifier", query],
            Err(timeout),
        );
        let m = MockChecker::none();
        let ctx = ScanContext::new(None, &m);
        let caches = provider(runner);
        let items = caches.list_caches(&ctx, &root).unwrap();
        assert_eq!(items[0].evidence, vec![Evidence::OwningAppUnknown]);
        let path = root.join("com.microsoft.VSCode");
        assert_eq!(caches.item_apps(&path), vec!["VSCode"]);
        cleanup(&root);
    }

    #[test]
    fn a_failed_plutil_read_is_unresolved() {
        let query = "kMDItemCFBundleIdentifier == 'com.microsoft.VSCode'";
        let runner = MockRunner::new().with_exit(
            MDFIND,
            &["-attr", "kMDItemCFBundleIdentifier", query],
            0,
            "/Applications/Visual Studio Code.app   kMDItemCFBundleIdentifier = com.microsoft.VSCode\n",
        );
        let caches = provider(runner);
        let path = Path::new("/c/com.microsoft.VSCode");
        assert_eq!(caches.item_apps(path), vec!["VSCode"]);
    }

    #[test]
    fn item_apps_names_the_executable_when_resolved_else_the_guess() {
        let caches = provider(vscode_runner());
        assert_eq!(
            caches.item_apps(Path::new("/c/com.microsoft.VSCode")),
            vec!["Code"]
        );
        assert_eq!(caches.item_apps(Path::new("/c/Homebrew")), vec!["Homebrew"]);
        assert_eq!(
            caches.item_apps(Path::new("/c/com.apple.Safari")),
            vec!["Safari"]
        );
    }

    #[test]
    fn discovery_resolves_all_ids_with_one_query_and_spawns_nothing_more_for_item_apps() {
        let root = fixture("spawns", &["com.example.A", "com.example.B", "Homebrew"]);
        let runner = Arc::new(MockRunner::new());
        let caches = UserCaches::with_runner(runner.clone());
        let m = MockChecker::none();
        let ctx = ScanContext::new(None, &m);
        let items = caches.list_caches(&ctx, &root).unwrap();
        for item in &items {
            caches.item_apps(&item.path);
        }
        let calls = runner.output_calls.lock().unwrap().clone();
        assert_eq!(calls.len(), 1, "{calls:?}");
        assert!(calls[0].contains("== 'com.example.A'"));
        assert!(calls[0].contains("== 'com.example.B'"));
        assert!(!calls[0].contains("Homebrew"));
        cleanup(&root);
    }

    #[test]
    fn only_bundle_id_shaped_names_reach_the_query() {
        assert!(is_bundle_id("com.microsoft.VSCode"));
        assert!(is_bundle_id("org.gonhanh.GoNhanh"));
        for bad in ["Homebrew", "a'b.c", "x.y z", "a.b') || ('1", "ünï.code", ""] {
            assert!(!is_bundle_id(bad), "{bad}");
        }
    }

    #[test]
    fn a_failed_probe_keeps_the_path_for_execution_to_recheck() {
        use crate::clean::process::test_support::FailingProbe;
        let root = fixture("probe", &["com.example.Safari"]);
        let ctx = ScanContext::new(None, &FailingProbe);
        let items = provider(MockRunner::new())
            .list_caches(&ctx, &root)
            .unwrap();
        assert_eq!(items.len(), 1);
        cleanup(&root);
    }
}
