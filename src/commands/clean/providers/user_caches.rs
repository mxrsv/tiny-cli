use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::{execute_per_item, read_root, CleanProvider};
use crate::commands::clean::fs_safe::dir_size_safe;
use crate::commands::clean::process::{PgrepChecker, ProcessChecker};
use crate::commands::clean::runner::{CommandRunner, RealRunner};
use crate::commands::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "user-caches";
const LABEL: &str = "User caches";

/// Absolute paths: a Finder-launched app has a minimal `PATH`.
const MDFIND: &str = "/usr/bin/mdfind";
const PLUTIL: &str = "/usr/bin/plutil";
const BUNDLE_ID_ATTR: &str = "kMDItemCFBundleIdentifier";
const APPLE_PREFIX: &str = "com.apple.";
/// Apple caches whose folder name lacks the `com.apple.` prefix.
const APPLE_FOLDERS: &[&str] = &[
    "Animoji",
    "CloudKit",
    "FamilyCircle",
    "familycircled",
    "GeoServices",
    "PassKit",
];
/// Folders named after a vendor, not a bundle id, and the app that owns them.
const VENDOR_APPS: &[(&str, &str)] = &[
    ("Google", "Google Chrome"),
    ("BraveSoftware", "Brave Browser"),
];
/// Ids per Spotlight query, so hundreds of cache folders stay safe.
const QUERY_CHUNK: usize = 50;

pub struct UserCaches {
    runner: Arc<dyn CommandRunner>,
    checker: Arc<dyn ProcessChecker>,
}

impl UserCaches {
    pub fn new() -> Self {
        Self {
            runner: Arc::new(RealRunner),
            checker: Arc::new(PgrepChecker),
        }
    }

    #[cfg(test)]
    fn with(runner: Arc<dyn CommandRunner>, checker: Arc<dyn ProcessChecker>) -> Self {
        Self { runner, checker }
    }

    /// Bundle id -> `CFBundleExecutable` of its app, for every id (and the
    /// parent id of `X.ShipIt`-style helper folders) Spotlight resolves.
    /// A failed tool leaves the affected ids out; it never fails the category.
    fn resolve(&self, ids: &[&str]) -> HashMap<String, String> {
        let mut queried: Vec<&str> = ids
            .iter()
            .copied()
            .chain(ids.iter().filter_map(|id| parent_id(id)))
            .collect();
        queried.sort_unstable();
        queried.dedup();
        self.find_apps(&queried)
            .into_iter()
            .filter_map(|(id, apps)| {
                let exe = apps.iter().find_map(|app| self.executable_of(app))?;
                Some((id, exe))
            })
            .collect()
    }

    /// Bundle id -> paths of the `.app` bundles Spotlight lists for it.
    /// `ids` passed `is_bundle_id`, so they are safe to put in the query.
    fn find_apps(&self, ids: &[&str]) -> HashMap<String, Vec<String>> {
        let mut found: HashMap<String, Vec<String>> = HashMap::new();
        let marker = format!(" {BUNDLE_ID_ATTR} = ");
        for chunk in ids.chunks(QUERY_CHUNK) {
            let query = chunk
                .iter()
                .map(|id| format!("{BUNDLE_ID_ATTR} == '{id}'"))
                .collect::<Vec<_>>()
                .join(" || ");
            let out = self.runner.run(MDFIND, &["-attr", BUNDLE_ID_ATTR, &query]);
            if !out.success {
                continue;
            }
            for line in out.stdout.lines() {
                let Some((app, id)) = line.rsplit_once(&marker) else {
                    continue;
                };
                let id = id.trim();
                if chunk.contains(&id) {
                    found
                        .entry(id.to_string())
                        .or_default()
                        .push(app.trim().to_string());
                }
            }
        }
        found
    }

    /// `CFBundleExecutable`: the process name `pgrep -x` matches (VS Code is
    /// `Code`, not `VSCode`).
    fn executable_of(&self, app: &str) -> Option<String> {
        let plist = format!("{app}/Contents/Info.plist");
        let out = self.runner.run(
            PLUTIL,
            &["-extract", "CFBundleExecutable", "raw", "-o", "-", &plist],
        );
        let exe = out.stdout.trim();
        (out.success && !exe.is_empty()).then(|| exe.to_string())
    }

    fn list_caches(&self, root: &Path) -> Result<Vec<CleanItem>> {
        let entries = match read_root(root)? {
            Some(it) => it,
            None => return Ok(Vec::new()),
        };
        let paths: Vec<(PathBuf, String)> = entries
            .flatten()
            .filter_map(|entry| {
                let path = entry.path();
                let name = path.file_name()?.to_str()?.to_string();
                (!is_apple_cache(&name)).then_some((path, name))
            })
            .collect();
        let ids: Vec<&str> = paths
            .iter()
            .map(|(_, name)| name.as_str())
            .filter(|name| is_bundle_id(name))
            .collect();
        let resolved = self.resolve(&ids);
        let mut out = Vec::new();
        for (path, name) in paths {
            let gate = gate_apps(&name, &resolved);
            if gate.iter().any(|app| self.checker.is_running(app)) {
                continue;
            }
            let size = dir_size_safe(&path);
            out.push(CleanItem {
                category_id: ID.to_string(),
                category_label: LABEL.to_string(),
                path,
                size,
                risk: RiskLevel::Review,
            });
        }
        Ok(out)
    }
}

impl Default for UserCaches {
    fn default() -> Self {
        Self::new()
    }
}

impl CleanProvider for UserCaches {
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
        self.list_caches(&h.join("Library/Caches"))
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, ID)
    }
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Process names to check for a cache folder: the vendor app for a vendor
/// folder; the resolved executable for a bundle id; for a helper folder
/// resolved through its parent, that executable plus the last-segment guess;
/// and the guess alone when nothing resolved, so the gate is never empty.
fn gate_apps(name: &str, resolved: &HashMap<String, String>) -> Vec<String> {
    if let Some((_, app)) = VENDOR_APPS.iter().find(|(folder, _)| *folder == name) {
        return vec![(*app).to_string()];
    }
    if let Some(exe) = resolved.get(name) {
        return vec![exe.clone()];
    }
    let guess = guess_app(name).map(String::from);
    match parent_id(name).and_then(|parent| resolved.get(parent)) {
        Some(exe) => {
            let guess = guess.filter(|g| g != exe);
            std::iter::once(exe.clone()).chain(guess).collect()
        }
        None => guess.into_iter().collect(),
    }
}

/// System daemon caches (`com.apple.bird`, `CloudKit`, ...) are not the
/// user's to clear.
fn is_apple_cache(name: &str) -> bool {
    name.get(..APPLE_PREFIX.len())
        .is_some_and(|p| p.eq_ignore_ascii_case(APPLE_PREFIX))
        || APPLE_FOLDERS.iter().any(|f| f.eq_ignore_ascii_case(name))
}

/// Looks like a reverse-DNS bundle id. Checked before the name goes into a
/// Spotlight query, so only `[A-Za-z0-9.-]` ever reaches it.
fn is_bundle_id(name: &str) -> bool {
    name.contains('.')
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
}

/// `com.microsoft.VSCode.ShipIt` -> `com.microsoft.VSCode`: one level up,
/// where a helper folder's app usually lives.
fn parent_id(id: &str) -> Option<&str> {
    id.rsplit_once('.')
        .map(|(parent, _)| parent)
        .filter(|parent| is_bundle_id(parent))
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
    use crate::commands::clean::fs_safe::remove_recursive_safe;
    use crate::commands::clean::process::test_support::MockChecker;
    use crate::commands::clean::runner::test_support::MockRunner;

    fn fixture(label: &str, names: &[&str]) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "tiny-user-caches-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        for name in names {
            std::fs::create_dir_all(root.join(name)).unwrap();
        }
        root
    }

    fn cleanup(root: &Path) {
        let _ = remove_recursive_safe(root);
    }

    /// The batched query for `ids`, as the provider builds it.
    fn query(ids: &[&str]) -> String {
        let clauses: Vec<_> = ids
            .iter()
            .map(|id| format!("kMDItemCFBundleIdentifier == '{id}'"))
            .collect();
        clauses.join(" || ")
    }

    /// Mocks the batched query for `ids` and one `plutil` read for VS Code.
    fn vscode_runner_for(ids: &[&str]) -> MockRunner {
        let query = query(ids);
        MockRunner::new()
            .with_response(
                MDFIND,
                &["-attr", "kMDItemCFBundleIdentifier", &query],
                true,
                "/Applications/Visual Studio Code.app   kMDItemCFBundleIdentifier = com.microsoft.VSCode\n",
            )
            .with_response(
                PLUTIL,
                &[
                    "-extract",
                    "CFBundleExecutable",
                    "raw",
                    "-o",
                    "-",
                    "/Applications/Visual Studio Code.app/Contents/Info.plist",
                ],
                true,
                "Code\n",
            )
    }

    fn provider(runner: MockRunner, running: &[&str]) -> UserCaches {
        UserCaches::with(
            Arc::new(runner),
            Arc::new(MockChecker::with_running(running.iter().copied())),
        )
    }

    fn names(items: &[CleanItem]) -> Vec<String> {
        let mut names: Vec<String> = items
            .iter()
            .map(|i| i.path.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn missing_root_is_empty_not_an_error() {
        let items = provider(MockRunner::new(), &[])
            .list_caches(Path::new("/tmp/__tiny_user_caches_missing__"))
            .unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn apple_daemon_caches_are_never_offered() {
        let root = fixture(
            "apple",
            &[
                "com.apple.bird",
                "COM.APPLE.passd",
                "Animoji",
                "CloudKit",
                "FamilyCircle",
                "familycircled",
                "GeoServices",
                "PassKit",
                "Homebrew",
            ],
        );
        let items = provider(MockRunner::new(), &[]).list_caches(&root).unwrap();
        assert_eq!(names(&items), vec!["Homebrew"]);
        cleanup(&root);
    }

    #[test]
    fn a_vendor_folder_is_gated_by_the_vendor_app() {
        let root = fixture("vendor", &["Google", "BraveSoftware"]);
        let items = provider(MockRunner::new(), &["Google Chrome"])
            .list_caches(&root)
            .unwrap();
        assert_eq!(names(&items), vec!["BraveSoftware"], "Chrome is running");
        let items = provider(MockRunner::new(), &[]).list_caches(&root).unwrap();
        assert_eq!(names(&items), vec!["BraveSoftware", "Google"]);
        cleanup(&root);
    }

    #[test]
    fn a_resolved_executable_gates_the_cache_while_the_app_runs() {
        let root = fixture("running", &["com.microsoft.VSCode"]);
        let runner = vscode_runner_for(&["com.microsoft", "com.microsoft.VSCode"]);
        let items = provider(runner, &["Code"]).list_caches(&root).unwrap();
        assert!(items.is_empty(), "Code is running");
        cleanup(&root);
    }

    #[test]
    fn a_resolved_app_ignores_the_last_segment_guess() {
        let root = fixture("idle", &["com.microsoft.VSCode"]);
        let runner = vscode_runner_for(&["com.microsoft", "com.microsoft.VSCode"]);
        let items = provider(runner, &["VSCode"]).list_caches(&root).unwrap();
        assert_eq!(names(&items), vec!["com.microsoft.VSCode"]);
        cleanup(&root);
    }

    #[test]
    fn an_unresolved_name_is_gated_by_the_guess() {
        let root = fixture("unknown", &["com.example.Idle", "com.example.Busy"]);
        let items = provider(MockRunner::new(), &["Busy"])
            .list_caches(&root)
            .unwrap();
        assert_eq!(names(&items), vec!["com.example.Idle"]);
        cleanup(&root);
    }

    #[test]
    fn a_helper_folder_is_gated_by_its_parent_app_and_the_guess() {
        let root = fixture("shipit", &["com.microsoft.VSCode.ShipIt"]);
        let ids = ["com.microsoft.VSCode", "com.microsoft.VSCode.ShipIt"];
        let items = provider(vscode_runner_for(&ids), &["Code"])
            .list_caches(&root)
            .unwrap();
        assert!(items.is_empty(), "Code is running");
        let items = provider(vscode_runner_for(&ids), &["ShipIt"])
            .list_caches(&root)
            .unwrap();
        assert!(items.is_empty(), "the guess is checked too");
        let items = provider(vscode_runner_for(&ids), &[])
            .list_caches(&root)
            .unwrap();
        assert_eq!(names(&items), vec!["com.microsoft.VSCode.ShipIt"]);
        cleanup(&root);
    }

    #[test]
    fn discovery_resolves_all_ids_with_one_query() {
        let root = fixture("spawns", &["com.example.A", "com.example.B", "Homebrew"]);
        let runner = Arc::new(MockRunner::new());
        let caches = UserCaches::with(runner.clone(), Arc::new(MockChecker::none()));
        caches.list_caches(&root).unwrap();
        let calls = runner.calls.lock().unwrap().clone();
        assert_eq!(calls.len(), 1, "{calls:?}");
        assert!(calls[0].contains("== 'com.example.A'"));
        assert!(calls[0].contains("== 'com.example.B'"));
        assert!(!calls[0].contains("Homebrew"));
        cleanup(&root);
    }

    #[test]
    fn long_id_lists_are_split_into_bounded_queries() {
        let runner = Arc::new(MockRunner::new());
        let caches = UserCaches::with(runner.clone(), Arc::new(MockChecker::none()));
        let owned: Vec<String> = (0..120).map(|i| format!("com.example.App{i}")).collect();
        let ids: Vec<&str> = owned.iter().map(String::as_str).collect();
        caches.find_apps(&ids);
        let calls = runner.calls.lock().unwrap().clone();
        assert_eq!(calls.len(), 3, "{calls:?}");
        for call in &calls {
            assert!(call.matches(" == '").count() <= QUERY_CHUNK);
        }
    }

    #[test]
    fn only_bundle_id_shaped_names_reach_the_query() {
        assert!(is_bundle_id("com.microsoft.VSCode"));
        assert!(is_bundle_id("org.gonhanh.GoNhanh"));
        for bad in ["Homebrew", "a'b.c", "x.y z", "a.b') || ('1", "ünï.code", ""] {
            assert!(!is_bundle_id(bad), "{bad}");
        }
    }
}
