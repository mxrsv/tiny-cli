use crate::engine_error;
use crate::error::{Context, Result};
use crate::runner::{CommandOutcome, CommandRunner, RealRunner, TOOL_TIMEOUT};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use super::fs_safe::{dir_size_checked, list_children, remove_recursive_safe};
use super::scan_context::ScanContext;
use super::types::{CleanItem, ComesBack, ExecAction, ExecReport, RiskLevel};

pub mod ai_models;
pub mod android_sdk;
pub mod app_orphans;
pub mod browser_automation;
pub mod browser_caches;
pub mod browser_profiles;
pub mod chat_caches;
pub mod crash_reports;
pub mod dev_caches;
pub mod device_updates;
pub mod docker;
pub mod downloads_old;
pub mod electron_app_caches;
pub mod font_quicklook_caches;
pub mod go_cache;
pub mod gradle_maven;
pub mod homebrew_cache;
pub mod ios_simulators;
pub mod jetbrains;
pub mod js_tool_caches;
pub mod macos_installers;
pub mod mail_attachments;
pub mod node_modules;
pub mod project_activity;
pub mod python_caches;
pub mod quarantine;
pub mod rust_targets;
pub mod screenshots_old;
pub mod simulator_devices;
pub mod streaming_caches;
pub mod swift_packages;
pub mod time_machine_local;
pub mod trash;
pub mod user_caches;
pub mod user_logs;
pub mod vscode;
pub mod xcode;

/// Where a category's data comes from; the Clean screen and the CLI picker
/// group categories by it. Assigned in `CATEGORIES`, never by a provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Family {
    AppsBrowsers,
    DeveloperTools,
    SystemLogs,
    YourFiles,
    Leftovers,
    AiModels,
}

impl Family {
    /// Display order.
    pub const ALL: [Family; 6] = [
        Family::AppsBrowsers,
        Family::DeveloperTools,
        Family::SystemLogs,
        Family::YourFiles,
        Family::Leftovers,
        Family::AiModels,
    ];

    pub fn id(&self) -> &'static str {
        match self {
            Family::AppsBrowsers => "apps-browsers",
            Family::DeveloperTools => "developer-tools",
            Family::SystemLogs => "system-logs",
            Family::YourFiles => "your-files",
            Family::Leftovers => "leftovers",
            Family::AiModels => "ai-models",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Family::AppsBrowsers => "Apps & browsers",
            Family::DeveloperTools => "Developer tools",
            Family::SystemLogs => "System & logs",
            Family::YourFiles => "Your files",
            Family::Leftovers => "Leftovers",
            Family::AiModels => "AI models",
        }
    }
}

/// The desktop trust group a category's tile sits in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustSection {
    /// Movable, and a tool or app recreates the items.
    Rebuilt,
    /// Movable, and only the Trash brings the items back.
    YourFiles,
    /// The desktop shows the items but never moves them.
    ReportOnly,
}

/// Per-category facts the UI needs without running discovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CategoryInfo {
    pub id: &'static str,
    pub family: Family,
    pub comes_back: ComesBack,
    /// `None` when the desktop moves the items to Trash. Enforcement stays
    /// with the provider (`desktop_report_only`); a test keeps the two equal.
    pub report_only: Option<ReportOnly>,
}

impl CategoryInfo {
    /// Report-only wins; otherwise a category is "rebuilt" only when
    /// something recreates its items, so permanent removal never reads as
    /// rebuilt.
    pub fn section(&self) -> TrustSection {
        if self.report_only.is_some() {
            return TrustSection::ReportOnly;
        }
        match self.comes_back {
            ComesBack::Rebuild { .. } | ComesBack::Redownload | ComesBack::AppRecreates => {
                TrustSection::Rebuilt
            }
            ComesBack::TrashOnly | ComesBack::NotRecoverable => TrustSection::YourFiles,
        }
    }
}

const fn category(
    id: &'static str,
    family: Family,
    comes_back: ComesBack,
    report_only: Option<ReportOnly>,
) -> CategoryInfo {
    CategoryInfo {
        id,
        family,
        comes_back,
        report_only,
    }
}

const fn rebuild(command: &'static str) -> ComesBack {
    ComesBack::Rebuild { command }
}

/// Every registered category in canonical id order, the order of
/// `all_providers`. A new category is added here and in `all_providers`.
pub const CATEGORIES: &[CategoryInfo] = {
    use ComesBack::{AppRecreates, NotRecoverable, Redownload, TrashOnly};
    use Family::{AiModels, AppsBrowsers, DeveloperTools, Leftovers, SystemLogs, YourFiles};
    &[
        category("user-logs", SystemLogs, AppRecreates, None),
        category(
            "xcode-derived",
            DeveloperTools,
            rebuild("Build in Xcode"),
            None,
        ),
        category("user-caches", AppsBrowsers, AppRecreates, None),
        category("xcode-archives", DeveloperTools, TrashOnly, None),
        category("xcode-devicesupport", DeveloperTools, AppRecreates, None),
        category("cargo", DeveloperTools, Redownload, None),
        category("npm", DeveloperTools, Redownload, None),
        category("pnpm", DeveloperTools, Redownload, None),
        category("yarn", DeveloperTools, Redownload, None),
        category(
            "node-modules",
            DeveloperTools,
            rebuild("npm, pnpm or yarn install"),
            None,
        ),
        category(
            "python-caches",
            DeveloperTools,
            rebuild("python -m venv, then pip install"),
            None,
        ),
        category("rust-targets", DeveloperTools, rebuild("cargo build"), None),
        category("gradle-maven", DeveloperTools, Redownload, None),
        category("jetbrains", DeveloperTools, AppRecreates, None),
        category("vscode", DeveloperTools, AppRecreates, None),
        category("ios-simulators", DeveloperTools, AppRecreates, None),
        // `xcrun simctl delete` removes the device for good.
        category(
            "simulator-devices",
            DeveloperTools,
            NotRecoverable,
            Some(ReportOnly::Destructive),
        ),
        category("android-sdk", DeveloperTools, Redownload, None),
        category("go-cache", DeveloperTools, rebuild("go build"), None),
        category(
            "browser-automation",
            DeveloperTools,
            rebuild("npx playwright install, or the tool's own install command"),
            None,
        ),
        category("swift-packages", DeveloperTools, Redownload, None),
        category("js-tool-caches", DeveloperTools, Redownload, None),
        category("homebrew-cache", DeveloperTools, Redownload, None),
        // Runs `docker image prune` / `docker builder prune`, not a per-path move.
        category(
            "docker",
            DeveloperTools,
            Redownload,
            Some(ReportOnly::NotPerPathTrash),
        ),
        // `docker volume prune` deletes container data.
        category(
            "docker-volumes",
            DeveloperTools,
            NotRecoverable,
            Some(ReportOnly::Destructive),
        ),
        category("downloads-old", YourFiles, TrashOnly, None),
        category("screenshots-old", YourFiles, TrashOnly, None),
        category("mail-attachments", YourFiles, TrashOnly, None),
        category("streaming-caches", AppsBrowsers, Redownload, None),
        category("chat-caches", AppsBrowsers, Redownload, None),
        category("browser-caches", AppsBrowsers, AppRecreates, None),
        category("browser-profiles", AppsBrowsers, AppRecreates, None),
        category("electron-app-caches", AppsBrowsers, AppRecreates, None),
        category("quarantine", SystemLogs, AppRecreates, None),
        category("crash-reports", SystemLogs, AppRecreates, None),
        // Folder names are matched against bundle IDs, so live data such as
        // `Code` (VS Code) or `AddressBook` is flagged as orphaned.
        category(
            "app-orphans",
            Leftovers,
            TrashOnly,
            Some(ReportOnly::UnreliableMatch),
        ),
        // `tmutil deletelocalsnapshots`.
        category(
            "time-machine-local",
            SystemLogs,
            NotRecoverable,
            Some(ReportOnly::Destructive),
        ),
        category("font-quicklook-caches", SystemLogs, AppRecreates, None),
        category("device-updates", Leftovers, Redownload, None),
        category("macos-installers", Leftovers, Redownload, None),
        // Measured only until per-tool removal (`ollama rm`, ...) exists.
        category(
            "ai-models",
            AiModels,
            Redownload,
            Some(ReportOnly::SizeOnly),
        ),
        // Empty Trash.
        category(
            "trash",
            Leftovers,
            NotRecoverable,
            Some(ReportOnly::Destructive),
        ),
    ]
};

/// The `CATEGORIES` entry for `category_id`; `None` for an unknown id.
pub fn category_info(category_id: &str) -> Option<&'static CategoryInfo> {
    CATEGORIES.iter().find(|info| info.id == category_id)
}

/// Maps a category id to its family. Panics on an unknown id.
pub fn category_family(category_id: &str) -> Family {
    match category_info(category_id) {
        Some(info) => info.family,
        None => panic!("unknown category id: {}", category_id),
    }
}

/// How a category's items come back after a move to Trash. `None` for an
/// id outside `known_category_ids()`.
pub fn comes_back(category_id: &str) -> Option<ComesBack> {
    category_info(category_id).map(|info| info.comes_back)
}

pub trait CleanProvider {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    /// One English sentence on why this category's items are candidates,
    /// shown beside them on the desktop (PC-C1). Not printed by the CLI.
    fn inclusion_reason(&self) -> String;
    fn risk(&self) -> RiskLevel;

    /// Process name that should NOT be running before discover/execute.
    /// `None` means no app gating.
    fn requires_app_quit(&self) -> Option<&'static str> {
        None
    }

    /// Every process name that must NOT be running before discover/execute:
    /// `requires_app_quit`, unless the provider gates on more than one app.
    fn quit_apps(&self) -> Vec<&'static str> {
        self.requires_app_quit().into_iter().collect()
    }

    /// Returns false when the provider is fundamentally unavailable on this
    /// system (e.g. CLI not installed). A provider that returns true here may
    /// still legitimately discover zero items.
    fn available(&self) -> bool {
        true
    }

    /// Tool whose absence makes `available()` false. Checked discovery
    /// reports such a provider as unavailable instead of omitting it.
    fn required_tool(&self) -> Option<&'static str> {
        None
    }

    /// Walks and running-app checks go through `ctx`, so they observe
    /// cancellation and record what could not be read.
    fn discover(&self, ctx: &ScanContext<'_>) -> Result<Vec<CleanItem>>;

    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport>;

    /// Deny by default. True only when cleanup is exactly "move each listed
    /// path to Trash": the desktop then moves those paths itself and never
    /// calls `execute`. Everything else is report-only on the desktop.
    fn desktop_trash_paths(&self) -> bool {
        false
    }

    /// True when item paths come from a tool's output; such items must lie
    /// inside the home folder (see `trash_plan::protected_reason`).
    fn roots_from_tool_output(&self) -> bool {
        false
    }

    /// Why a provider without `desktop_trash_paths` is report-only.
    fn desktop_report_only_reason(&self) -> ReportOnly {
        ReportOnly::NotPerPathTrash
    }

    /// Apps that must not be running when `path` is acted on.
    fn item_apps(&self, _path: &Path) -> Vec<String> {
        self.quit_apps().into_iter().map(String::from).collect()
    }

    /// Provider-specific guard re-checked immediately before acting on
    /// `path`; `Err` carries the reason it is refused.
    fn check_item(&self, _path: &Path) -> std::result::Result<(), String> {
        Ok(())
    }
}

/// Why the desktop only reports a category instead of moving its items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportOnly {
    /// Permanent deletion or Empty Trash.
    Destructive,
    /// Cleanup is a tool command, not a per-path move to Trash.
    NotPerPathTrash,
    /// The rule that selects items can flag data still in use.
    UnreliableMatch,
    /// Shown for its size; no removal is offered yet.
    SizeOnly,
}

/// `None` when the desktop may move this provider's items to Trash.
pub fn desktop_report_only(provider: &dyn CleanProvider) -> Option<ReportOnly> {
    if provider.risk() == RiskLevel::Destructive {
        Some(ReportOnly::Destructive)
    } else if provider.desktop_trash_paths() {
        None
    } else {
        Some(provider.desktop_report_only_reason())
    }
}

/// Returns every provider in canonical id order. Filtering by risk level,
/// `--category`, and runtime availability happens in `discover.rs`.
///
/// Takes `&CleanOpts` so providers needing tunables (idle_days, search
/// roots, ...) can read them at construction. M0 providers don't yet, but
/// the param exists so M1+ can land without re-threading callers.
pub fn all_providers(opts: &crate::options::CleanOptions) -> Vec<Box<dyn CleanProvider>> {
    all_providers_with(opts, Arc::new(RealRunner))
}

/// `all_providers` with the runner tool-backed providers use, so the app can
/// pass its own tool lookup and tests a mock.
pub fn all_providers_with(
    opts: &crate::options::CleanOptions,
    runner: Arc<dyn CommandRunner>,
) -> Vec<Box<dyn CleanProvider>> {
    vec![
        Box::new(user_logs::UserLogs),
        Box::new(xcode::XcodeDerivedData),
        Box::new(user_caches::UserCaches::with_runner(runner.clone())),
        Box::new(xcode::XcodeArchives),
        Box::new(xcode::XcodeDeviceSupport),
        Box::new(dev_caches::CargoCache),
        Box::new(dev_caches::NpmCache::with_runner(runner.clone())),
        Box::new(dev_caches::PnpmStore::with_runner(runner.clone())),
        Box::new(dev_caches::YarnCache::with_runner(runner.clone())),
        Box::new(node_modules::NodeModules::with_runner(
            opts.idle_days,
            runner.clone(),
        )),
        Box::new(python_caches::PythonCaches::with_runner(
            opts.idle_days,
            runner.clone(),
        )),
        Box::new(rust_targets::RustTargets::with_runner(
            opts.idle_days,
            runner.clone(),
        )),
        Box::new(gradle_maven::GradleMaven),
        Box::new(jetbrains::JetBrains),
        Box::new(vscode::VsCode),
        Box::new(ios_simulators::IosSimulators),
        Box::new(simulator_devices::SimulatorDevices::with_runner(
            runner.clone(),
        )),
        Box::new(android_sdk::AndroidSdk::new(opts.idle_days)),
        Box::new(go_cache::GoCache::with_runner(runner.clone())),
        Box::new(browser_automation::BrowserAutomation),
        Box::new(swift_packages::SwiftPackages),
        Box::new(js_tool_caches::JsToolCaches),
        Box::new(homebrew_cache::HomebrewCache),
        Box::new(docker::Docker::with_runner(runner.clone())),
        Box::new(docker::DockerVolumes::with_runner(runner.clone())),
        Box::new(downloads_old::DownloadsOld::with_runner(
            opts.idle_days,
            runner.clone(),
        )),
        Box::new(screenshots_old::ScreenshotsOld::with_runner(
            opts.idle_days,
            runner.clone(),
        )),
        Box::new(mail_attachments::MailAttachments),
        Box::new(streaming_caches::StreamingCaches::new()),
        Box::new(chat_caches::ChatCaches::new()),
        Box::new(browser_caches::BrowserCaches::new()),
        Box::new(browser_profiles::BrowserProfiles),
        Box::new(electron_app_caches::ElectronAppCaches),
        Box::new(quarantine::Quarantine),
        Box::new(crash_reports::CrashReports),
        Box::new(app_orphans::AppOrphans::with_runner(runner.clone())),
        Box::new(time_machine_local::TimeMachineLocal::with_runner(
            runner.clone(),
        )),
        Box::new(font_quicklook_caches::FontQuicklookCaches::with_runner(
            runner,
        )),
        Box::new(device_updates::DeviceUpdates),
        Box::new(macos_installers::MacosInstallers),
        Box::new(ai_models::AiModels),
        Box::new(trash::TrashProvider),
    ]
}

/// Children of `~/Library/Caches` split out of `user-caches` into their own
/// categories, so no path is offered twice. Each provider declares its own.
pub(crate) fn split_from_user_caches(name: &str) -> bool {
    [
        browser_automation::LIBRARY_CACHES,
        swift_packages::LIBRARY_CACHES,
        js_tool_caches::LIBRARY_CACHES,
        homebrew_cache::LIBRARY_CACHES,
    ]
    .iter()
    .any(|names| names.contains(&name))
}

pub(crate) fn home() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(std::path::PathBuf::from)
}

/// Each path in `roots` that is a real directory (never a symlink) as one
/// item, in order. Unlike `root_as_item`, a symlinked root is skipped.
pub(crate) fn dir_roots_as_items(
    ctx: &ScanContext<'_>,
    roots: &[std::path::PathBuf],
    category_id: &str,
    category_label: &str,
    risk: RiskLevel,
) -> Vec<CleanItem> {
    roots
        .iter()
        .filter(|root| super::fs_safe::is_dir_safe(root))
        .flat_map(|root| root_as_item(ctx, root, category_id, category_label, risk))
        .collect()
}

/// Canonical list of category ids accepted by `--category`.
pub fn known_category_ids() -> &'static [&'static str] {
    static IDS: std::sync::OnceLock<Vec<&'static str>> = std::sync::OnceLock::new();
    IDS.get_or_init(|| CATEGORIES.iter().map(|info| info.id).collect())
}

/// Canonical set of dev project roots scanned by walking providers
/// (`node_modules`, `rust_targets`, `python_caches`). Only roots that exist
/// are returned. Order is stable so test discovery is deterministic.
pub(crate) fn dev_search_roots() -> Vec<std::path::PathBuf> {
    let h = match std::env::var_os("HOME") {
        Some(h) => std::path::PathBuf::from(h),
        None => return Vec::new(),
    };
    const ROOTS: &[&str] = &["Documents", "Projects", "Code", "Developer", "Workspace"];
    ROOTS
        .iter()
        .map(|r| h.join(r))
        .filter(|p| p.is_dir())
        .collect()
}

/// True iff `manifest_path` exists and was modified more than `idle_days`
/// days ago. Treats unreadable mtime as "not idle" (conservative — won't
/// flag a project we can't measure).
pub(crate) fn is_idle(manifest_path: &std::path::Path, idle_days: u64) -> bool {
    let meta = match std::fs::symlink_metadata(manifest_path) {
        Ok(m) => m,
        Err(_) => return false,
    };
    let mtime = match meta.modified() {
        Ok(t) => t,
        Err(_) => return false,
    };
    let elapsed = match std::time::SystemTime::now().duration_since(mtime) {
        Ok(d) => d,
        Err(_) => return false,
    };
    elapsed.as_secs() > idle_days * 86_400
}

/// Lists immediate children of `root` as `CleanItem`s, sized via the
/// symlink-safe walk. Returns empty when `root` does not exist.
pub(crate) fn top_level_entries(
    ctx: &ScanContext<'_>,
    root: &Path,
    category_id: &str,
    category_label: &str,
    risk: RiskLevel,
) -> Vec<CleanItem> {
    let mut out = Vec::new();
    for path in list_children(root, ctx) {
        let size = dir_size_checked(&path, ctx);
        out.push(CleanItem {
            category_id: category_id.to_string(),
            category_label: category_label.to_string(),
            path,
            size,
            risk,
            evidence: Vec::new(),
        });
    }
    out
}

/// Treats `root` itself as a single CleanItem (used for category-rooted
/// providers like xcode-derived). Returns empty when missing.
pub(crate) fn root_as_item(
    ctx: &ScanContext<'_>,
    root: &Path,
    category_id: &str,
    category_label: &str,
    risk: RiskLevel,
) -> Vec<CleanItem> {
    if !root.exists() {
        return Vec::new();
    }
    ctx.add_root(root);
    let size = dir_size_checked(root, ctx);
    vec![CleanItem {
        category_id: category_id.to_string(),
        category_label: category_label.to_string(),
        path: root.to_path_buf(),
        size,
        risk,
        evidence: Vec::new(),
    }]
}

/// Runs a discovery tool through the bounded runner. A missing tool, spawn
/// failure or timeout is an error (checked discovery reports the category
/// as failed); the exit status is left to the provider.
pub(crate) fn run_tool(
    runner: &dyn CommandRunner,
    bin: &str,
    args: &[&str],
) -> Result<CommandOutcome> {
    runner
        .output(bin, args, TOOL_TIMEOUT)
        .map_err(|e| engine_error!("{e}"))
}

/// Default per-item executor used by every provider except Trash. Rejects
/// `ExecAction::EmptyTrash`.
pub(crate) fn execute_per_item(
    items: &[CleanItem],
    action: ExecAction,
    provider_id: &'static str,
) -> Result<ExecReport> {
    if matches!(action, ExecAction::EmptyTrash) {
        return Err(engine_error!(
            "{} provider does not accept EmptyTrash",
            provider_id
        ));
    }
    let mut report = ExecReport::default();
    for item in items {
        let result = match action {
            ExecAction::Trash => move_to_trash(&item.path),
            ExecAction::HardDelete => remove_recursive_safe(&item.path)
                .map_err(|e| engine_error!("remove {}: {}", item.path.display(), e)),
            ExecAction::EmptyTrash => unreachable!(),
        };
        match result {
            Ok(()) => report.removed_paths.push(item.path.clone()),
            Err(e) => report.failed.push((item.path.clone(), e.to_string())),
        }
    }
    Ok(report)
}

/// Move a path to the user's Trash via Finder. Mirrors the helper in
/// `uninstall.rs` but lives here so the clean module is self-contained.
pub(crate) fn move_to_trash(path: &Path) -> Result<()> {
    let posix = path
        .to_str()
        .ok_or_else(|| engine_error!("non-utf8 path: {}", path.display()))?;
    let output = Command::new("osascript")
        .arg("-e")
        .arg(super::finder_trash::FINDER_DELETE_SCRIPT)
        .arg(posix)
        .output()
        .with_context(|| format!("failed to spawn osascript for {}", path.display()))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(engine_error!(
            "osascript failed for {}: {}",
            path.display(),
            err
        ));
    }
    Ok(())
}

/// A throwaway folder standing in for `$HOME` (or `/Applications`) in
/// provider fixtures; removed on drop. Never the real home (R6).
#[cfg(test)]
pub(crate) mod test_home {
    use super::CleanItem;
    use std::path::{Path, PathBuf};

    pub struct TestHome(tempfile::TempDir);

    impl TestHome {
        pub fn new(label: &str) -> Self {
            Self(
                tempfile::Builder::new()
                    .prefix(&format!("tiny-{label}-"))
                    .tempdir()
                    .unwrap(),
            )
        }

        pub fn path(&self) -> &Path {
            self.0.path()
        }

        pub fn dir(&self, rel: &str) -> PathBuf {
            let path = self.path().join(rel);
            std::fs::create_dir_all(&path).unwrap();
            path
        }

        /// A file of `len` bytes, creating its parents.
        pub fn file(&self, rel: &str, len: usize) -> PathBuf {
            let path = self.path().join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, vec![0u8; len]).unwrap();
            path
        }

        /// `link` pointing at `target`, both relative to the fixture.
        pub fn symlink(&self, link: &str, target: &str) {
            let link = self.path().join(link);
            std::fs::create_dir_all(link.parent().unwrap()).unwrap();
            std::os::unix::fs::symlink(self.path().join(target), link).unwrap();
        }

        /// Item paths relative to the fixture, in discovery order.
        pub fn relative(&self, items: &[CleanItem]) -> Vec<String> {
            items
                .iter()
                .map(|item| {
                    let rel = item.path.strip_prefix(self.path()).unwrap();
                    rel.to_string_lossy().into_owned()
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn family_id_and_label_stable() {
        let ids: Vec<&str> = Family::ALL.iter().map(Family::id).collect();
        assert_eq!(
            ids,
            [
                "apps-browsers",
                "developer-tools",
                "system-logs",
                "your-files",
                "leftovers",
                "ai-models"
            ]
        );
        let labels: Vec<&str> = Family::ALL.iter().map(Family::label).collect();
        assert_eq!(
            labels,
            [
                "Apps & browsers",
                "Developer tools",
                "System & logs",
                "Your files",
                "Leftovers",
                "AI models"
            ]
        );
    }

    /// The table is registration: a provider without an entry, an entry
    /// without a provider, or a desktop decision that differs from what the
    /// provider enforces fails here.
    #[test]
    fn every_provider_has_a_category_entry_that_matches_it() {
        let runner: Arc<dyn CommandRunner> =
            Arc::new(crate::runner::test_support::MockRunner::new());
        let providers = all_providers_with(&crate::options::CleanOptions::default(), runner);
        let ids: Vec<&str> = providers.iter().map(|p| p.id()).collect();
        let table: Vec<&str> = CATEGORIES.iter().map(|info| info.id).collect();
        assert_eq!(ids, table);
        assert_eq!(known_category_ids(), table.as_slice());
        for provider in &providers {
            let info = category_info(provider.id()).unwrap();
            assert_eq!(
                desktop_report_only(provider.as_ref()),
                info.report_only,
                "{}",
                provider.id()
            );
        }
    }

    #[test]
    fn sections_follow_the_desktop_decision_and_comes_back() {
        let section = |id| category_info(id).unwrap().section();
        assert_eq!(section("cargo"), TrustSection::Rebuilt);
        assert_eq!(section("rust-targets"), TrustSection::Rebuilt);
        assert_eq!(section("user-caches"), TrustSection::Rebuilt);
        assert_eq!(section("downloads-old"), TrustSection::YourFiles);
        assert_eq!(section("xcode-archives"), TrustSection::YourFiles);
        assert_eq!(section("docker"), TrustSection::ReportOnly);
        assert_eq!(section("app-orphans"), TrustSection::ReportOnly);
        assert_eq!(section("trash"), TrustSection::ReportOnly);
        for info in CATEGORIES {
            if info.comes_back == ComesBack::NotRecoverable {
                assert_ne!(info.section(), TrustSection::Rebuilt, "{}", info.id);
            }
        }
    }

    #[test]
    fn every_family_has_a_category() {
        for family in Family::ALL {
            assert!(
                CATEGORIES.iter().any(|info| info.family == family),
                "{}",
                family.id()
            );
        }
    }

    #[test]
    fn every_category_states_why_its_items_are_candidates() {
        let runner: Arc<dyn CommandRunner> =
            Arc::new(crate::runner::test_support::MockRunner::new());
        let opts = crate::options::CleanOptions {
            idle_days: 45,
            ..Default::default()
        };
        let providers = all_providers_with(&opts, runner);
        let ids: Vec<&str> = providers.iter().map(|p| p.id()).collect();
        assert_eq!(ids, known_category_ids());
        for provider in &providers {
            let reason = provider.inclusion_reason();
            assert!(!reason.trim().is_empty(), "{} has no reason", provider.id());
        }
        let reason_of = |id: &str| {
            providers
                .iter()
                .find(|p| p.id() == id)
                .unwrap()
                .inclusion_reason()
        };
        for idle in [
            "node-modules",
            "rust-targets",
            "python-caches",
            "downloads-old",
        ] {
            assert!(
                reason_of(idle).contains("45 days"),
                "{idle} names its threshold"
            );
        }
    }

    #[test]
    fn per_path_app_gates_name_the_owning_app() {
        let h = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
        let browser = browser_caches::BrowserCaches::new();
        let chrome = h.join("Library/Application Support/Google/Chrome/Default/Cache");
        assert_eq!(browser.item_apps(&chrome), vec!["Google Chrome"]);
        assert!(browser.item_apps(Path::new("/elsewhere")).len() > 1);
        let caches = user_caches::UserCaches::with_runner(Arc::new(
            crate::runner::test_support::MockRunner::new(),
        ));
        assert_eq!(
            caches.item_apps(&h.join("Library/Caches/com.apple.Safari")),
            vec!["Safari"]
        );
        assert_eq!(
            xcode::XcodeDerivedData.item_apps(Path::new("/x")),
            vec!["Xcode"]
        );
    }

    #[test]
    fn cargo_guard_refuses_everything_but_the_pinned_cache_dirs() {
        // Exercises the guard only; no execute or Trash path runs.
        let h = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
        let cargo = dev_caches::CargoCache;
        for pinned in ["registry/cache", "registry/src", "git/db", "git/checkouts"] {
            assert!(cargo.check_item(&h.join(".cargo").join(pinned)).is_ok());
        }
        for other in [".cargo/bin", ".cargo/credentials.toml", ".cargo", ".rustup"] {
            assert!(cargo.check_item(&h.join(other)).is_err(), "{other}");
        }
    }

    #[test]
    fn every_category_says_how_its_items_come_back() {
        for id in known_category_ids() {
            assert!(comes_back(id).is_some(), "{id}");
        }
        assert_eq!(comes_back("not-a-category"), None);
    }

    #[test]
    #[should_panic(expected = "unknown category id")]
    fn category_family_panics_on_unknown() {
        let _ = category_family("definitely-not-a-category");
    }

    /// AC5: one fixture home holding every path the new categories and the
    /// providers they border read; no path is offered twice and none sits
    /// inside another provider's item.
    #[test]
    fn no_path_is_offered_by_two_providers() {
        use crate::clean::process::test_support::MockChecker;
        let home = test_home::TestHome::new("overlap");
        let caches = "Library/Caches";
        for name in [
            "ms-playwright",
            "ms-playwright-mcp",
            "Cypress",
            "org.swift.swiftpm",
            "CocoaPods",
            "org.carthage.CarthageKit",
            "electron",
            "node-gyp",
            "deno",
            "Homebrew",
            "Google",
            "com.example.Other",
        ] {
            home.dir(&format!("{caches}/{name}/x"));
        }
        for dir in [".cache/puppeteer", ".cache/selenium", ".bun/install/cache"] {
            home.dir(dir);
        }
        let support = "Library/Application Support";
        for dir in [
            "Google/Chrome/Default/Cache",
            "Google/Chrome/Default/Code Cache",
            "Arc/User Data/Default/Cache",
            "Arc/User Data/Default/Code Cache",
            "Notion/Cache",
            "Code/Cache",
            "Slack/Cache",
            "discord/Cache",
        ] {
            home.dir(&format!("{support}/{dir}"));
        }
        home.file("Library/iTunes/iPhone Software Updates/a.ipsw", 1);
        home.dir(".ollama/models");
        home.dir(".cache/huggingface/hub/models--a--b");

        let none = MockChecker::none();
        let ctx = ScanContext::new(None, &none);
        let h = home.path();
        let user_caches = user_caches::UserCaches::with_runner(Arc::new(
            crate::runner::test_support::MockRunner::new(),
        ));
        let mut items = user_caches.list_caches(&ctx, &h.join(caches)).unwrap();
        items.extend(browser_caches::BrowserCaches::new().discover_in(&ctx, h));
        items.extend(browser_automation::BrowserAutomation.discover_in(&ctx, h));
        items.extend(swift_packages::SwiftPackages.discover_in(&ctx, h));
        items.extend(js_tool_caches::JsToolCaches.discover_in(&ctx, h));
        items.extend(homebrew_cache::HomebrewCache.discover_in(&ctx, h));
        items.extend(browser_profiles::BrowserProfiles.discover_in(&ctx, h));
        items.extend(electron_app_caches::ElectronAppCaches.discover_in(&ctx, h));
        items.extend(device_updates::DeviceUpdates.discover_in(&ctx, h));
        items.extend(ai_models::AiModels.discover_in(&ctx, h));

        for (i, a) in items.iter().enumerate() {
            for b in &items[i + 1..] {
                assert!(
                    !a.path.starts_with(&b.path) && !b.path.starts_with(&a.path),
                    "{} ({}) overlaps {} ({})",
                    a.path.display(),
                    a.category_id,
                    b.path.display(),
                    b.category_id
                );
            }
        }
        // Every new category found its fixture, so the check above saw it.
        for id in [
            "browser-automation",
            "swift-packages",
            "js-tool-caches",
            "homebrew-cache",
            "browser-profiles",
            "electron-app-caches",
            "device-updates",
            "ai-models",
        ] {
            assert!(items.iter().any(|i| i.category_id == id), "{id}");
        }
        let mut user: Vec<_> = items
            .iter()
            .filter(|i| i.category_id == "user-caches")
            .map(|i| i.path.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        user.sort();
        assert_eq!(user, ["Google", "com.example.Other"]);
    }
}
