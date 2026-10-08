//! The desktop's Move to Trash: each planned path is revalidated right
//! before it is moved, and every path gets its own outcome. It never calls
//! `CleanProvider::execute`, so no provider command runs from here.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use super::finder_trash::{Trash, TrashError};
use super::fs_safe::PathFingerprint;
use super::process::AppProbe;
use super::providers::{desktop_report_only, CleanProvider};
use super::trash_plan::{protected_reason, PlannedItem};
use super::types::CleanItem;

/// Why a path was not moved even though it was planned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// The category is report-only on the desktop.
    ReportOnly,
    /// No registered provider has the item's category ID.
    UnknownCategory,
    Missing,
    /// The path itself is a symlink now.
    Symlink,
    /// Replaced or edited since discovery.
    Changed,
    /// Not inside a discovery root, or a symlink appeared on the way to it.
    OutsideRoots,
    /// The file-system root, the home folder or one of its ancestors, or a
    /// tool-reported path outside the home folder.
    ProtectedPath(String),
    /// On another volume than the home folder. Finder may delete such a
    /// path permanently when that volume has no Trash.
    NotOnHomeVolume,
    /// The provider's own guard refused the path.
    ProviderGuard(String),
    AppRunning(String),
    /// A required safety check could not run; acting would be a guess.
    SafetyCheckFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemOutcome {
    MovedToTrash,
    /// Trash refused the path. `source_present` is re-checked afterwards.
    Failed {
        error: TrashError,
        source_present: bool,
    },
    Skipped(SkipReason),
    /// Execution stopped before this path.
    NotAttempted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopReason {
    Cancelled,
    AutomationDenied,
}

#[derive(Debug, Clone)]
pub struct ItemResult {
    pub item: CleanItem,
    pub outcome: ItemOutcome,
}

#[derive(Debug, Clone, Default)]
pub struct CheckedExecReport {
    /// One entry per planned item, in plan order.
    pub results: Vec<ItemResult>,
    pub stopped: Option<StopReason>,
}

impl CheckedExecReport {
    /// Sizes measured at discovery of every planned item.
    pub fn bytes_selected(&self) -> u64 {
        self.sum(|_| true)
    }

    /// Sizes measured at discovery of the items moved to Trash. Moving to
    /// Trash frees no space until Trash is emptied.
    pub fn bytes_moved_to_trash(&self) -> u64 {
        self.sum(|outcome| *outcome == ItemOutcome::MovedToTrash)
    }

    pub fn moved_count(&self) -> usize {
        self.count(|outcome| *outcome == ItemOutcome::MovedToTrash)
    }

    pub fn count(&self, keep: impl Fn(&ItemOutcome) -> bool) -> usize {
        self.results.iter().filter(|r| keep(&r.outcome)).count()
    }

    fn sum(&self, keep: impl Fn(&ItemOutcome) -> bool) -> u64 {
        self.results
            .iter()
            .filter(|r| keep(&r.outcome))
            .fold(0, |sum, r| sum.saturating_add(r.item.size))
    }
}

/// Everything execution needs besides the plan.
pub struct ExecContext<'a> {
    pub providers: &'a [Box<dyn CleanProvider>],
    /// The home folder (`trash_plan::home_dir()` in the app).
    pub home: Option<&'a Path>,
    pub probe: &'a dyn AppProbe,
    pub trash: &'a dyn Trash,
    pub cancel: Option<&'a AtomicBool>,
    pub progress: crate::progress::ProgressCallback<'a>,
}

impl ExecContext<'_> {
    fn is_cancelled(&self) -> bool {
        self.cancel.is_some_and(|flag| flag.load(Ordering::Acquire))
    }
}

/// Moves each planned path to Trash after `validate` passes for it. Stops
/// on cancellation or an Automation denial; the remaining paths are
/// reported as not attempted.
pub fn execute_checked(plan: &[PlannedItem], ctx: &ExecContext<'_>) -> CheckedExecReport {
    let total = plan.len() as u64;
    let mut report = CheckedExecReport::default();
    for (index, planned) in plan.iter().enumerate() {
        let outcome = match report.stopped {
            Some(_) => ItemOutcome::NotAttempted,
            None => {
                crate::progress::report(
                    ctx.progress,
                    "trash",
                    index as u64,
                    Some(total),
                    &planned.item.path.to_string_lossy(),
                );
                act_on(planned, ctx, &mut report.stopped)
            }
        };
        report.results.push(ItemResult {
            item: planned.item.clone(),
            outcome,
        });
    }
    if report.stopped.is_none() {
        crate::progress::report(ctx.progress, "trash", total, Some(total), "Done");
    }
    report
}

fn act_on(
    planned: &PlannedItem,
    ctx: &ExecContext<'_>,
    stopped: &mut Option<StopReason>,
) -> ItemOutcome {
    let cancelled = |stopped: &mut Option<StopReason>| {
        *stopped = Some(StopReason::Cancelled);
        ItemOutcome::NotAttempted
    };
    if ctx.is_cancelled() {
        return cancelled(stopped);
    }
    if let Err(reason) = validate(planned, ctx) {
        return ItemOutcome::Skipped(reason);
    }
    // Last check before the mutation.
    if ctx.is_cancelled() {
        return cancelled(stopped);
    }
    let path = &planned.item.path;
    match ctx.trash.move_to_trash(path) {
        Ok(()) => ItemOutcome::MovedToTrash,
        Err(error) => {
            if matches!(error, TrashError::AutomationDenied(_)) {
                *stopped = Some(StopReason::AutomationDenied);
            }
            ItemOutcome::Failed {
                error,
                source_present: fs::symlink_metadata(path).is_ok(),
            }
        }
    }
}

/// The per-item validator, run immediately before each move: provider
/// eligibility, `symlink_metadata`, fingerprint, root containment, the
/// providers' guards, then the running-app check. Guards and app gates of
/// every item merged into this one apply too.
pub fn validate(planned: &PlannedItem, ctx: &ExecContext<'_>) -> Result<(), SkipReason> {
    let item = &planned.item;
    let gated = gated_items(planned, ctx)?;
    let meta = fs::symlink_metadata(&item.path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => SkipReason::Missing,
        _ => SkipReason::SafetyCheckFailed(e.to_string()),
    })?;
    if meta.file_type().is_symlink() {
        return Err(SkipReason::Symlink);
    }
    if PathFingerprint::of(&meta) != planned.fingerprint {
        return Err(SkipReason::Changed);
    }
    if meta.dev() != home_device(ctx.home)? {
        return Err(SkipReason::NotOnHomeVolume);
    }
    if !planned
        .roots
        .iter()
        .any(|root| inside_root(&item.path, root))
    {
        return Err(SkipReason::OutsideRoots);
    }
    let tool_printed = gated.iter().any(|(p, _)| p.roots_from_tool_output());
    if let Some(reason) = protected_reason(&item.path, ctx.home, tool_printed) {
        return Err(SkipReason::ProtectedPath(reason));
    }
    for (provider, path) in &gated {
        provider
            .check_item(path)
            .map_err(SkipReason::ProviderGuard)?;
    }
    let apps: BTreeSet<String> = gated
        .iter()
        .flat_map(|(provider, path)| provider.item_apps(path))
        .collect();
    for app in apps {
        match ctx.probe.probe(&app) {
            Ok(false) => {}
            Ok(true) => return Err(SkipReason::AppRunning(app)),
            Err(e) => {
                return Err(SkipReason::SafetyCheckFailed(format!(
                    "running-app check for {app} failed: {e}"
                )))
            }
        }
    }
    Ok(())
}

/// Device of the home folder, whose volume is known to have a Trash.
fn home_device(home: Option<&Path>) -> Result<u64, SkipReason> {
    let home = home.ok_or_else(|| SkipReason::SafetyCheckFailed("home folder unknown".into()))?;
    fs::metadata(home)
        .map(|m| m.dev())
        .map_err(|e| SkipReason::SafetyCheckFailed(format!("cannot stat home folder: {e}")))
}

/// The item and every item merged into it, each with its provider. Any
/// unknown or report-only provider refuses the whole move.
fn gated_items<'a>(
    planned: &'a PlannedItem,
    ctx: &'a ExecContext<'_>,
) -> Result<Vec<(&'a dyn CleanProvider, &'a Path)>, SkipReason> {
    std::iter::once(&planned.item)
        .chain(&planned.covers)
        .map(|item| {
            let provider = ctx
                .providers
                .iter()
                .find(|p| p.id() == item.category_id)
                .ok_or(SkipReason::UnknownCategory)?;
            if desktop_report_only(provider.as_ref()).is_some() {
                return Err(SkipReason::ReportOnly);
            }
            Ok((provider.as_ref(), item.path.as_path()))
        })
        .collect()
}

/// `path` is `root` or below it, and `root` plus every directory between
/// them is still a real directory, so no symlink redirects the move.
fn inside_root(path: &Path, root: &Path) -> bool {
    if !path.starts_with(root) {
        return false;
    }
    // Excludes `path` itself, which `validate` already checked.
    path.ancestors()
        .skip(1)
        .take_while(|ancestor| ancestor.starts_with(root))
        .all(is_real_dir)
}

fn is_real_dir(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_dir())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clean::fs_safe::{fingerprint, remove_recursive_safe};
    use crate::clean::process::test_support::{FailingProbe, MockChecker};
    use crate::clean::scan_context::ScanContext;
    use crate::clean::types::{ExecAction, ExecReport, RiskLevel};
    use crate::error::Result;
    use std::path::PathBuf;
    use std::sync::Mutex;

    struct Fixture {
        id: &'static str,
        desktop: bool,
        risk: RiskLevel,
        app: Option<&'static str>,
        guard: Option<&'static str>,
        tool: bool,
    }

    fn fixture(id: &'static str) -> Fixture {
        Fixture {
            id,
            desktop: true,
            risk: RiskLevel::Safe,
            app: None,
            guard: None,
            tool: false,
        }
    }

    impl CleanProvider for Fixture {
        fn id(&self) -> &'static str {
            self.id
        }
        fn label(&self) -> &'static str {
            self.id
        }
        fn risk(&self) -> RiskLevel {
            self.risk
        }
        fn inclusion_reason(&self) -> String {
            "fixture".into()
        }
        fn requires_app_quit(&self) -> Option<&'static str> {
            self.app
        }
        fn discover(&self, _: &ScanContext<'_>) -> Result<Vec<CleanItem>> {
            Ok(Vec::new())
        }
        fn execute(&self, _: &[CleanItem], _: ExecAction) -> Result<ExecReport> {
            panic!("checked execution must never call provider.execute");
        }
        fn desktop_trash_paths(&self) -> bool {
            self.desktop
        }
        fn roots_from_tool_output(&self) -> bool {
            self.tool
        }
        fn check_item(&self, path: &Path) -> std::result::Result<(), String> {
            match self.guard {
                Some(name) if path.ends_with(name) => Err(format!("{name} is protected")),
                _ => Ok(()),
            }
        }
    }

    /// Records each move; fails with `fail` for paths ending in `fail_on`.
    #[derive(Default)]
    struct FakeTrash {
        moved: Mutex<Vec<PathBuf>>,
        fail_on: Option<(&'static str, TrashError)>,
    }

    impl Trash for FakeTrash {
        fn move_to_trash(&self, path: &Path) -> std::result::Result<(), TrashError> {
            if let Some((name, error)) = &self.fail_on {
                if path.ends_with(name) {
                    return Err(error.clone());
                }
            }
            self.moved.lock().unwrap().push(path.to_path_buf());
            Ok(())
        }
    }

    fn fixture_root(label: &str, names: &[&str]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "tiny-checked-exec-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        for name in names {
            fs::write(dir.join(name), b"1234").unwrap();
        }
        dir
    }

    fn plan(category: &str, root: &Path, name: &str) -> PlannedItem {
        let path = root.join(name);
        PlannedItem {
            fingerprint: fingerprint(&path).unwrap(),
            item: CleanItem {
                category_id: category.into(),
                category_label: category.into(),
                size: 4,
                path,
                risk: RiskLevel::Safe,
            },
            roots: vec![root.to_path_buf()],
            covers: Vec::new(),
        }
    }

    fn run(
        plan: &[PlannedItem],
        providers: Vec<Box<dyn CleanProvider>>,
        probe: &dyn AppProbe,
        trash: &dyn Trash,
        cancel: Option<&AtomicBool>,
    ) -> CheckedExecReport {
        let home = std::env::temp_dir();
        let ctx = ExecContext {
            providers: &providers,
            home: Some(&home),
            probe,
            trash,
            cancel,
            progress: None,
        };
        execute_checked(plan, &ctx)
    }

    fn outcomes(report: &CheckedExecReport) -> Vec<ItemOutcome> {
        report.results.iter().map(|r| r.outcome.clone()).collect()
    }

    #[test]
    fn empty_plan_touches_nothing() {
        let trash = FakeTrash::default();
        let report = run(&[], vec![], &MockChecker::none(), &trash, None);
        assert!(report.results.is_empty());
        assert!(trash.moved.lock().unwrap().is_empty());
    }

    #[test]
    fn moves_valid_items_and_reports_bytes_without_claiming_freed_space() {
        let root = fixture_root("ok", &["a", "b"]);
        let items = [plan("logs", &root, "a"), plan("logs", &root, "b")];
        let trash = FakeTrash::default();
        let report = run(
            &items,
            vec![Box::new(fixture("logs"))],
            &MockChecker::none(),
            &trash,
            None,
        );
        assert_eq!(outcomes(&report), vec![ItemOutcome::MovedToTrash; 2]);
        assert_eq!(report.bytes_selected(), 8);
        assert_eq!(report.bytes_moved_to_trash(), 8);
        assert_eq!(trash.moved.lock().unwrap().len(), 2);
        let _ = remove_recursive_safe(&root);
    }

    #[test]
    fn report_only_and_destructive_categories_are_refused() {
        let root = fixture_root("report-only", &["images", "snap"]);
        let mut docker = fixture("docker");
        docker.desktop = false;
        let mut destructive = fixture("tm");
        destructive.risk = RiskLevel::Destructive;
        let items = [plan("docker", &root, "images"), plan("tm", &root, "snap")];
        let trash = FakeTrash::default();
        let report = run(
            &items,
            vec![Box::new(docker), Box::new(destructive)],
            &MockChecker::none(),
            &trash,
            None,
        );
        assert_eq!(
            outcomes(&report),
            vec![ItemOutcome::Skipped(SkipReason::ReportOnly); 2]
        );
        assert!(trash.moved.lock().unwrap().is_empty());
        let _ = remove_recursive_safe(&root);
    }

    #[test]
    fn symlinks_replacements_and_missing_paths_are_skipped() {
        let root = fixture_root("swap", &["edited", "replaced", "gone", "target"]);
        let items = [
            plan("logs", &root, "edited"),
            plan("logs", &root, "replaced"),
            plan("logs", &root, "gone"),
            plan("logs", &root, "target"),
        ];
        fs::write(root.join("edited"), b"longer than before").unwrap();
        fs::write(root.join("replacement"), b"1234").unwrap();
        fs::rename(root.join("replacement"), root.join("replaced")).unwrap();
        remove_recursive_safe(&root.join("gone")).unwrap();
        fs::rename(root.join("target"), root.join("moved")).unwrap();
        std::os::unix::fs::symlink(root.join("moved"), root.join("target")).unwrap();

        let trash = FakeTrash::default();
        let report = run(
            &items,
            vec![Box::new(fixture("logs"))],
            &MockChecker::none(),
            &trash,
            None,
        );
        assert_eq!(
            outcomes(&report),
            vec![
                ItemOutcome::Skipped(SkipReason::Changed),
                ItemOutcome::Skipped(SkipReason::Changed),
                ItemOutcome::Skipped(SkipReason::Missing),
                ItemOutcome::Skipped(SkipReason::Symlink),
            ]
        );
        assert!(trash.moved.lock().unwrap().is_empty());
        let _ = remove_recursive_safe(&root);
    }

    #[test]
    fn a_symlinked_directory_on_the_way_from_the_root_is_refused() {
        let root = fixture_root("ancestor", &[]);
        fs::create_dir(root.join("sub")).unwrap();
        fs::write(root.join("sub/f"), b"1234").unwrap();
        let item = plan("logs", &root, "sub/f");
        // Swap `sub` for a symlink to a directory holding the same file.
        let elsewhere = fixture_root("elsewhere", &[]);
        fs::rename(root.join("sub"), elsewhere.join("sub")).unwrap();
        std::os::unix::fs::symlink(elsewhere.join("sub"), root.join("sub")).unwrap();

        let trash = FakeTrash::default();
        let report = run(
            &[item],
            vec![Box::new(fixture("logs"))],
            &MockChecker::none(),
            &trash,
            None,
        );
        assert_eq!(
            outcomes(&report),
            vec![ItemOutcome::Skipped(SkipReason::OutsideRoots)]
        );
        let _ = remove_recursive_safe(&root);
        let _ = remove_recursive_safe(&elsewhere);
    }

    #[test]
    fn paths_outside_the_discovery_roots_are_refused() {
        let root = fixture_root("roots", &["a"]);
        let mut item = plan("logs", &root, "a");
        item.roots = vec![root.join("elsewhere")];
        let trash = FakeTrash::default();
        let report = run(
            &[item],
            vec![Box::new(fixture("logs"))],
            &MockChecker::none(),
            &trash,
            None,
        );
        assert_eq!(
            outcomes(&report),
            vec![ItemOutcome::Skipped(SkipReason::OutsideRoots)]
        );
        let _ = remove_recursive_safe(&root);
    }

    #[test]
    fn provider_guard_running_apps_and_failed_probes_refuse_items() {
        let root = fixture_root("guards", &["registry", "bin"]);
        let items = [
            plan("cargo", &root, "registry"),
            plan("cargo", &root, "bin"),
            plan("xcode", &root, "registry"),
        ];
        let trash = FakeTrash::default();
        let providers = || -> Vec<Box<dyn CleanProvider>> {
            let mut guarded = fixture("cargo");
            guarded.guard = Some("bin");
            let mut gated = fixture("xcode");
            gated.app = Some("Xcode");
            vec![Box::new(guarded), Box::new(gated)]
        };
        let running = MockChecker::with_running(["Xcode"]);
        let report = run(&items, providers(), &running, &trash, None);
        assert_eq!(
            outcomes(&report),
            vec![
                ItemOutcome::MovedToTrash,
                ItemOutcome::Skipped(SkipReason::ProviderGuard("bin is protected".into())),
                ItemOutcome::Skipped(SkipReason::AppRunning("Xcode".into())),
            ]
        );

        let report = run(&items[2..], providers(), &FailingProbe, &trash, None);
        assert!(matches!(
            &outcomes(&report)[0],
            ItemOutcome::Skipped(SkipReason::SafetyCheckFailed(detail)) if detail.contains("Xcode")
        ));
        let _ = remove_recursive_safe(&root);
    }

    #[test]
    fn home_its_ancestors_and_tool_paths_outside_home_are_refused() {
        let home = fixture_root("home", &[]);
        let outside = fixture_root("outside", &["cache"]);
        let mut at_home = plan("logs", &home, "");
        at_home.item.path = home.clone();
        at_home.fingerprint = fingerprint(&home).unwrap();
        let tool_outside = plan("npm", &outside, "cache");
        let plain_outside = plan("logs", &outside, "cache");
        let providers: Vec<Box<dyn CleanProvider>> = vec![Box::new(fixture("logs")), {
            let mut npm = fixture("npm");
            npm.tool = true;
            Box::new(npm)
        }];
        let trash = FakeTrash::default();
        let ctx = ExecContext {
            providers: &providers,
            home: Some(&home),
            probe: &MockChecker::none(),
            trash: &trash,
            cancel: None,
            progress: None,
        };
        let report = execute_checked(&[at_home, tool_outside, plain_outside], &ctx);
        assert_eq!(
            outcomes(&report),
            vec![
                ItemOutcome::Skipped(SkipReason::ProtectedPath("the home folder".into())),
                ItemOutcome::Skipped(SkipReason::ProtectedPath(
                    "a tool reported a path outside the home folder".into()
                )),
                ItemOutcome::MovedToTrash,
            ]
        );
        let _ = remove_recursive_safe(&home);
        let _ = remove_recursive_safe(&outside);
    }

    #[test]
    fn items_on_another_volume_than_home_are_refused() {
        let root = fixture_root("volume", &["a"]);
        let providers: Vec<Box<dyn CleanProvider>> = vec![Box::new(fixture("logs"))];
        let trash = FakeTrash::default();
        // devfs is a different device than the temp dir's volume.
        let ctx = ExecContext {
            providers: &providers,
            home: Some(Path::new("/dev")),
            probe: &MockChecker::none(),
            trash: &trash,
            cancel: None,
            progress: None,
        };
        let report = execute_checked(&[plan("logs", &root, "a")], &ctx);
        assert_eq!(
            outcomes(&report),
            vec![ItemOutcome::Skipped(SkipReason::NotOnHomeVolume)]
        );
        let unknown = ExecContext { home: None, ..ctx };
        let report = execute_checked(&[plan("logs", &root, "a")], &unknown);
        assert!(matches!(
            &outcomes(&report)[0],
            ItemOutcome::Skipped(SkipReason::SafetyCheckFailed(_))
        ));
        assert!(trash.moved.lock().unwrap().is_empty());
        let _ = remove_recursive_safe(&root);
    }

    #[test]
    fn merged_items_keep_their_guards_and_app_gates() {
        let root = fixture_root("merged", &["cache", "bin"]);
        let providers = || -> Vec<Box<dyn CleanProvider>> {
            let mut guarded = fixture("cargo");
            guarded.guard = Some("bin");
            let mut gated = fixture("streaming");
            gated.app = Some("Spotify");
            vec![
                Box::new(fixture("caches")),
                Box::new(guarded),
                Box::new(gated),
            ]
        };
        // A duplicate found by an app-gated category.
        let mut duplicate = plan("caches", &root, "cache");
        duplicate.covers = vec![plan("streaming", &root, "cache").item];
        // A path covering a guarded one.
        let mut covering = plan("caches", &root, "cache");
        covering.covers = vec![plan("cargo", &root, "bin").item];
        let trash = FakeTrash::default();
        let running = MockChecker::with_running(["Spotify"]);
        let report = run(&[duplicate, covering], providers(), &running, &trash, None);
        assert_eq!(
            outcomes(&report),
            vec![
                ItemOutcome::Skipped(SkipReason::AppRunning("Spotify".into())),
                ItemOutcome::Skipped(SkipReason::ProviderGuard("bin is protected".into())),
            ]
        );
        assert!(trash.moved.lock().unwrap().is_empty());
        let _ = remove_recursive_safe(&root);
    }

    #[test]
    fn cancellation_reports_completed_work_and_leaves_the_rest() {
        let root = fixture_root("cancel", &["a", "b", "c"]);
        let items = [
            plan("logs", &root, "a"),
            plan("logs", &root, "b"),
            plan("logs", &root, "c"),
        ];
        let flag = AtomicBool::new(false);
        /// Cancels after the first move.
        struct CancellingTrash<'a>(&'a AtomicBool, FakeTrash);
        impl Trash for CancellingTrash<'_> {
            fn move_to_trash(&self, path: &Path) -> std::result::Result<(), TrashError> {
                self.0.store(true, Ordering::Release);
                self.1.move_to_trash(path)
            }
        }
        let trash = CancellingTrash(&flag, FakeTrash::default());
        let report = run(
            &items,
            vec![Box::new(fixture("logs"))],
            &MockChecker::none(),
            &trash,
            Some(&flag),
        );
        assert_eq!(report.stopped, Some(StopReason::Cancelled));
        assert_eq!(
            outcomes(&report),
            vec![
                ItemOutcome::MovedToTrash,
                ItemOutcome::NotAttempted,
                ItemOutcome::NotAttempted
            ]
        );
        assert_eq!(report.bytes_moved_to_trash(), 4);
        assert!(root.join("b").exists() && root.join("c").exists());
        let _ = remove_recursive_safe(&root);
    }

    #[test]
    fn cancellation_is_checked_before_validation_too() {
        let root = fixture_root("cancel-first", &["a", "b"]);
        let items = [plan("logs", &root, "a"), plan("logs", &root, "b")];
        let flag = AtomicBool::new(true);
        let mut gated = fixture("logs");
        gated.app = Some("Xcode");
        let trash = FakeTrash::default();
        // A failing probe would turn a validated item into a skip.
        let report = run(
            &items,
            vec![Box::new(gated)],
            &FailingProbe,
            &trash,
            Some(&flag),
        );
        assert_eq!(report.stopped, Some(StopReason::Cancelled));
        assert_eq!(outcomes(&report), vec![ItemOutcome::NotAttempted; 2]);
        let _ = remove_recursive_safe(&root);
    }

    #[test]
    fn automation_denial_stops_and_trash_failures_keep_the_source() {
        let root = fixture_root("denied", &["a", "b", "c"]);
        let items = [
            plan("logs", &root, "a"),
            plan("logs", &root, "b"),
            plan("logs", &root, "c"),
        ];
        let failing = FakeTrash {
            fail_on: Some(("a", TrashError::Failed("Finder error".into()))),
            ..Default::default()
        };
        let report = run(
            &items[..2],
            vec![Box::new(fixture("logs"))],
            &MockChecker::none(),
            &failing,
            None,
        );
        assert_eq!(
            outcomes(&report),
            vec![
                ItemOutcome::Failed {
                    error: TrashError::Failed("Finder error".into()),
                    source_present: true,
                },
                ItemOutcome::MovedToTrash,
            ],
            "a Trash failure is per item and never deletes"
        );
        assert!(root.join("a").exists());

        let denied = FakeTrash {
            fail_on: Some(("c", TrashError::AutomationDenied("(-1743)".into()))),
            ..Default::default()
        };
        let report = run(
            &[items[2].clone(), plan("logs", &root, "a")],
            vec![Box::new(fixture("logs"))],
            &MockChecker::none(),
            &denied,
            None,
        );
        assert_eq!(report.stopped, Some(StopReason::AutomationDenied));
        assert_eq!(outcomes(&report)[1], ItemOutcome::NotAttempted);
        assert!(denied.moved.lock().unwrap().is_empty());
        let _ = remove_recursive_safe(&root);
    }
}
