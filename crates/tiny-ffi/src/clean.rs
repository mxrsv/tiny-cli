//! Cleanup discovery, trusted preview and Trash execution for the native app.
//!
//! The session keeps what discovery found (paths, fingerprints, roots and
//! the options used). A preview is built only from those candidate IDs, and
//! execution only from a preview ID, so Swift never hands Rust a path to act
//! on.

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::{Arc, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use tiny_core::clean::discover::{
    discover_checked, select_providers_with, validate_options, CategoryOutcome, CheckedCategory,
};
use tiny_core::clean::fs_safe::{fingerprint, PathFingerprint};
use tiny_core::clean::process::{AppProbe, RunnerProbe};
use tiny_core::clean::providers::{category_family, known_category_ids, CleanProvider};
use tiny_core::clean::scan_context::{ScanContext, Unreadable};
use tiny_core::clean::trash_plan::{partition_overlaps, Overlap, PlannedItem};
use tiny_core::clean::types::{CleanItem, RiskLevel};
use tiny_core::options::CleanOptions;
use tiny_core::runner::{CommandRunner, ToolLookup, ToolRunner};

use crate::session::CancellationToken;
use crate::{FfiError, ProgressListener, TinySession};

/// How long a confirmed preview stays executable.
pub const PREVIEW_TTL: Duration = Duration::from_secs(15 * 60);

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiCleanOptions {
    /// Category IDs to scan; empty means "filter by risk flags".
    pub categories: Vec<String>,
    pub include_review: bool,
    pub include_destructive: bool,
    pub idle_days: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiRisk {
    Safe,
    Review,
    Destructive,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum FfiCategoryStatus {
    Found,
    /// The owning app is running, so the category was not scanned.
    AppRunning {
        app: String,
    },
    /// A required tool or safety probe is unavailable.
    Unavailable {
        reason: String,
    },
    Failed {
        detail: String,
    },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiUnreadablePath {
    pub path: String,
    pub entries: u64,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiCleanCandidate {
    /// Opaque; valid only for the discovery that returned it.
    pub id: String,
    pub path: String,
    pub size_bytes: u64,
    /// Entries the size walk could not read; non-zero makes `size_bytes` a
    /// lower bound.
    pub unreadable_entries: u64,
    pub risk: FfiRisk,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiCleanCategory {
    pub id: String,
    pub label: String,
    /// `dev`, `user-storage` or `system`; `None` for an unregistered ID.
    pub family: Option<String>,
    pub risk: FfiRisk,
    pub status: FfiCategoryStatus,
    pub candidates: Vec<FfiCleanCandidate>,
    pub total_bytes: u64,
    /// Listing or search paths with unreadable entries.
    pub unreadable: Vec<FfiUnreadablePath>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiDiscovery {
    pub discovery_id: String,
    pub categories: Vec<FfiCleanCategory>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiPreviewItem {
    pub candidate_id: String,
    pub category_id: String,
    pub path: String,
    pub size_bytes: u64,
    pub risk: FfiRisk,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum FfiExclusionReason {
    /// Another selected category found the same path.
    Duplicate { kept_candidate_id: String },
    /// A selected ancestor moves this path with it.
    InsideSelected { parent_candidate_id: String },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiPreviewExclusion {
    pub candidate_id: String,
    pub path: String,
    pub reason: FfiExclusionReason,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiPreview {
    pub preview_id: String,
    pub items: Vec<FfiPreviewItem>,
    pub excluded: Vec<FfiPreviewExclusion>,
    /// Sum of `items` sizes as measured at discovery. Not freed space.
    pub bytes_selected: u64,
    pub expires_in_seconds: u64,
}

/// Session-held cleanup state. Never locked across provider work, Trash
/// calls or progress callbacks.
#[derive(Default)]
pub(crate) struct CleanState {
    next_id: u64,
    discovery: Option<StoredDiscovery>,
    preview: Option<StoredPreview>,
    /// Test override for `PREVIEW_TTL`.
    ttl: Option<Duration>,
}

#[allow(dead_code)] // `options` is read by execution (T4c)
struct StoredDiscovery {
    id: u64,
    options: CleanOptions,
    candidates: HashMap<String, StoredCandidate>,
}

struct StoredCandidate {
    item: CleanItem,
    /// `None` when the path could not be stat'ed (e.g. a tool placeholder).
    fingerprint: Option<PathFingerprint>,
    roots: Arc<Vec<PathBuf>>,
}

#[allow(dead_code)] // read by execution (T4c)
struct StoredPreview {
    id: String,
    discovery_id: u64,
    /// `(candidate ID, plan)` in execution order.
    items: Vec<(String, PlannedItem)>,
    expires_at: Instant,
    /// Kept after use so a replay reports "consumed", not "unknown".
    consumed: bool,
}

impl CleanState {
    fn next_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }
}

#[uniffi::export]
impl TinySession {
    /// Scans cleanup categories. Holds the operation gate, so it returns
    /// `Busy` while another scan or mutation runs. Replaces the previous
    /// discovery and any preview built from it.
    pub fn clean_discover(
        &self,
        options: FfiCleanOptions,
        token: Arc<CancellationToken>,
        progress: Arc<dyn ProgressListener>,
    ) -> Result<FfiDiscovery, FfiError> {
        let options = CleanOptions::from(options);
        let runner = app_runner();
        let providers = select_providers_with(&options, runner.clone());
        let probe = RunnerProbe(runner.as_ref());
        self.discover_clean(options, &providers, &probe, &token, progress.as_ref())
    }

    /// Builds a preview from candidate IDs of the current discovery. Unknown
    /// or stale IDs are rejected; overlapping paths are listed as excluded.
    /// Replaces any earlier preview.
    pub fn clean_preview(&self, candidate_ids: Vec<String>) -> Result<FfiPreview, FfiError> {
        let mut state = self.clean_state();
        let ttl = state.ttl.unwrap_or(PREVIEW_TTL);
        let discovery = state
            .discovery
            .as_ref()
            .ok_or_else(|| FfiError::PreviewInvalid {
                detail: "no current discovery; scan again".into(),
            })?;
        let discovery_id = discovery.id;
        let planned = plan_selection(discovery, candidate_ids)?;
        let (kept, covered) = partition_overlaps(planned, |(_, plan)| plan.item.path.as_path());
        let excluded = covered
            .into_iter()
            .map(|((id, plan), overlap)| exclusion(&kept, id, &plan, overlap))
            .collect();
        let preview_id = format!("p{}", state.next_id());
        let preview = FfiPreview {
            preview_id: preview_id.clone(),
            items: kept
                .iter()
                .map(|(id, plan)| preview_item(id, plan))
                .collect(),
            excluded,
            bytes_selected: total_size(kept.iter().map(|(_, plan)| &plan.item)),
            expires_in_seconds: ttl.as_secs(),
        };
        state.preview = Some(StoredPreview {
            id: preview_id,
            discovery_id,
            items: kept,
            expires_at: Instant::now() + ttl,
            consumed: false,
        });
        Ok(preview)
    }
}

/// The app's runner: `PATH`, then the Homebrew prefixes.
fn app_runner() -> Arc<dyn CommandRunner> {
    Arc::new(ToolRunner::new(ToolLookup::app()))
}

impl TinySession {
    /// Lock on cleanup state. Locked sections never run foreign or provider
    /// code, so a poisoned lock is recovered rather than failing every call.
    pub(crate) fn clean_state(&self) -> MutexGuard<'_, CleanState> {
        self.clean.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// `clean_discover` with injected providers and probe (the test seam).
    pub(crate) fn discover_clean(
        &self,
        options: CleanOptions,
        providers: &[Box<dyn CleanProvider>],
        probe: &dyn AppProbe,
        token: &CancellationToken,
        progress: &dyn ProgressListener,
    ) -> Result<FfiDiscovery, FfiError> {
        validate_options(&options)?;
        let _gate = self.begin()?;
        {
            let mut state = self.clean_state();
            state.discovery = None;
            state.preview = None;
        }
        let report = |p: tiny_core::progress::Progress| progress.on_progress(p.into());
        let ctx = ScanContext::new(Some(token.flag()), probe);
        let checked = discover_checked(providers, &ctx, Some(&report));
        if checked.cancelled {
            return Err(FfiError::Cancelled);
        }
        let mut state = self.clean_state();
        let id = state.next_id();
        let mut candidates = HashMap::new();
        let categories = checked
            .categories
            .into_iter()
            .map(|category| store_category(id, category, &mut candidates))
            .collect();
        state.discovery = Some(StoredDiscovery {
            id,
            options,
            candidates,
        });
        Ok(FfiDiscovery {
            discovery_id: format!("d{id}"),
            categories,
        })
    }
}

fn store_category(
    discovery_id: u64,
    category: CheckedCategory,
    candidates: &mut HashMap<String, StoredCandidate>,
) -> FfiCleanCategory {
    let family = known_category_ids()
        .contains(&category.id.as_str())
        .then(|| category_family(&category.id).id().to_string());
    let mut ffi = FfiCleanCategory {
        id: category.id,
        label: category.label,
        family,
        risk: category.risk.into(),
        status: FfiCategoryStatus::Found,
        candidates: Vec::new(),
        total_bytes: 0,
        unreadable: Vec::new(),
    };
    match category.outcome {
        CategoryOutcome::Found {
            items,
            unreadable,
            roots,
        } => {
            let roots = Arc::new(roots);
            for checked in items {
                let id = format!("d{discovery_id}-{}", candidates.len());
                let item = checked.item;
                ffi.total_bytes = ffi.total_bytes.saturating_add(item.size);
                ffi.candidates.push(FfiCleanCandidate {
                    id: id.clone(),
                    path: item.path.to_string_lossy().into_owned(),
                    size_bytes: item.size,
                    unreadable_entries: checked.unreadable.map_or(0, |u| u.entries),
                    risk: item.risk.into(),
                });
                candidates.insert(
                    id,
                    StoredCandidate {
                        fingerprint: fingerprint(&item.path).ok(),
                        item,
                        roots: roots.clone(),
                    },
                );
            }
            ffi.unreadable = unreadable.into_iter().map(unreadable_path).collect();
        }
        CategoryOutcome::AppRunning { app } => ffi.status = FfiCategoryStatus::AppRunning { app },
        CategoryOutcome::Unavailable { reason } => {
            ffi.status = FfiCategoryStatus::Unavailable { reason }
        }
        CategoryOutcome::Failed { error } => {
            ffi.status = FfiCategoryStatus::Failed { detail: error }
        }
    }
    ffi
}

/// Resolves selected IDs against the stored discovery only.
fn plan_selection(
    discovery: &StoredDiscovery,
    candidate_ids: Vec<String>,
) -> Result<Vec<(String, PlannedItem)>, FfiError> {
    let unique: BTreeSet<String> = candidate_ids.into_iter().collect();
    if unique.is_empty() {
        return Err(FfiError::InvalidInput {
            detail: "empty selection".into(),
        });
    }
    unique
        .into_iter()
        .map(|id| {
            let candidate =
                discovery
                    .candidates
                    .get(&id)
                    .ok_or_else(|| FfiError::PreviewInvalid {
                        detail: format!("unknown candidate {id}; scan again"),
                    })?;
            let fingerprint =
                candidate
                    .fingerprint
                    .clone()
                    .ok_or_else(|| FfiError::InvalidInput {
                        detail: format!(
                            "{} is not a file-system path that can be moved to Trash",
                            candidate.item.path.display()
                        ),
                    })?;
            let plan = PlannedItem {
                item: candidate.item.clone(),
                fingerprint,
                roots: candidate.roots.as_ref().clone(),
            };
            Ok((id, plan))
        })
        .collect()
}

fn exclusion(
    kept: &[(String, PlannedItem)],
    id: String,
    plan: &PlannedItem,
    overlap: Overlap,
) -> FfiPreviewExclusion {
    let reason = match overlap {
        Overlap::Duplicate { kept: index } => FfiExclusionReason::Duplicate {
            kept_candidate_id: kept[index].0.clone(),
        },
        Overlap::Inside { kept: index } => FfiExclusionReason::InsideSelected {
            parent_candidate_id: kept[index].0.clone(),
        },
    };
    FfiPreviewExclusion {
        candidate_id: id,
        path: plan.item.path.to_string_lossy().into_owned(),
        reason,
    }
}

fn preview_item(id: &str, plan: &PlannedItem) -> FfiPreviewItem {
    FfiPreviewItem {
        candidate_id: id.to_string(),
        category_id: plan.item.category_id.clone(),
        path: plan.item.path.to_string_lossy().into_owned(),
        size_bytes: plan.item.size,
        risk: plan.item.risk.into(),
    }
}

fn total_size<'a>(items: impl Iterator<Item = &'a CleanItem>) -> u64 {
    items.fold(0, |sum, item| sum.saturating_add(item.size))
}

fn unreadable_path((path, unreadable): (PathBuf, Unreadable)) -> FfiUnreadablePath {
    FfiUnreadablePath {
        path: path.to_string_lossy().into_owned(),
        entries: unreadable.entries,
        detail: unreadable.first_error,
    }
}

impl From<FfiCleanOptions> for CleanOptions {
    fn from(options: FfiCleanOptions) -> Self {
        Self {
            category: options.categories,
            include_review: options.include_review,
            include_destructive: options.include_destructive,
            idle_days: options.idle_days,
        }
    }
}

impl From<RiskLevel> for FfiRisk {
    fn from(risk: RiskLevel) -> Self {
        match risk {
            RiskLevel::Safe => Self::Safe,
            RiskLevel::Review => Self::Review,
            RiskLevel::Destructive => Self::Destructive,
        }
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    use tiny_core::clean::fs_safe::{dir_size_checked, list_children};
    use tiny_core::clean::process::AppProbe;
    use tiny_core::clean::providers::CleanProvider;
    use tiny_core::clean::scan_context::ScanContext;
    use tiny_core::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};
    use tiny_core::error::Result;

    use crate::{FfiProgress, ProgressListener};

    /// Lists `root`'s children as candidates. Execution must never reach
    /// `execute`: the desktop path acts through the `Trash` trait only.
    pub struct Fixture {
        pub id: &'static str,
        pub root: PathBuf,
        pub risk: RiskLevel,
        pub app: Option<&'static str>,
        pub extra: Vec<PathBuf>,
    }

    impl Fixture {
        pub fn new(id: &'static str, root: &Path) -> Self {
            Self {
                id,
                root: root.to_path_buf(),
                risk: RiskLevel::Safe,
                app: None,
                extra: Vec::new(),
            }
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
        fn requires_app_quit(&self) -> Option<&'static str> {
            self.app
        }
        fn discover(&self, ctx: &ScanContext<'_>) -> Result<Vec<CleanItem>> {
            let paths = list_children(&self.root, ctx)
                .into_iter()
                .chain(self.extra.iter().cloned());
            Ok(paths
                .map(|path| CleanItem {
                    category_id: self.id.into(),
                    category_label: self.id.into(),
                    size: dir_size_checked(&path, ctx),
                    path,
                    risk: self.risk,
                })
                .collect())
        }
        fn execute(&self, _: &[CleanItem], _: ExecAction) -> Result<ExecReport> {
            panic!("the desktop path must never call provider.execute");
        }
    }

    /// No app is running.
    pub struct NotRunning;

    impl AppProbe for NotRunning {
        fn probe(&self, _: &str) -> std::result::Result<bool, String> {
            Ok(false)
        }
    }

    #[derive(Default)]
    pub struct Progress(pub Mutex<Vec<FfiProgress>>);

    impl ProgressListener for Progress {
        fn on_progress(&self, progress: FfiProgress) {
            self.0.lock().unwrap().push(progress);
        }
    }

    /// A unique temp dir with `names` as children holding `len` bytes each.
    pub fn fixture_dir(label: &str, names: &[&str], len: usize) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "tiny-ffi-clean-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        for name in names {
            std::fs::write(dir.join(name), vec![b'x'; len]).unwrap();
        }
        dir
    }

    pub fn remove(dir: &Path) {
        let _ = tiny_core::clean::fs_safe::remove_recursive_safe(dir);
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;

    fn options() -> CleanOptions {
        CleanOptions::default()
    }

    fn discover(
        session: &TinySession,
        providers: &[Box<dyn CleanProvider>],
    ) -> Result<FfiDiscovery, FfiError> {
        session.discover_clean(
            options(),
            providers,
            &NotRunning,
            &CancellationToken::default(),
            &Progress::default(),
        )
    }

    fn ids(discovery: &FfiDiscovery) -> Vec<String> {
        discovery
            .categories
            .iter()
            .flat_map(|c| c.candidates.iter().map(|c| c.id.clone()))
            .collect()
    }

    #[test]
    fn preview_is_built_from_discovered_ids_only() {
        let dir = fixture_dir("preview", &["a", "b"], 4);
        let providers: Vec<Box<dyn CleanProvider>> = vec![Box::new(Fixture::new("logs", &dir))];
        let session = TinySession::new();
        let progress = Progress::default();
        let discovery = session
            .discover_clean(
                options(),
                &providers,
                &NotRunning,
                &CancellationToken::default(),
                &progress,
            )
            .unwrap();
        assert!(
            !progress.0.lock().unwrap().is_empty(),
            "progress reached the caller"
        );
        let category = &discovery.categories[0];
        assert_eq!(category.status, FfiCategoryStatus::Found);
        assert_eq!(category.total_bytes, 8);
        assert_eq!(category.family, None, "fixture IDs are not registered");

        let preview = session.clean_preview(ids(&discovery)).unwrap();
        assert_eq!(preview.items.len(), 2);
        assert_eq!(preview.bytes_selected, 8);
        assert!(preview.excluded.is_empty());
        assert_eq!(preview.expires_in_seconds, PREVIEW_TTL.as_secs());

        let forged = session.clean_preview(vec!["d1-99".into()]);
        assert!(matches!(forged, Err(FfiError::PreviewInvalid { .. })));
        let empty = session.clean_preview(Vec::new());
        assert!(matches!(empty, Err(FfiError::InvalidInput { .. })));
        remove(&dir);
    }

    #[test]
    fn a_new_discovery_invalidates_old_candidate_ids() {
        let dir = fixture_dir("replaced", &["a"], 1);
        let providers: Vec<Box<dyn CleanProvider>> = vec![Box::new(Fixture::new("logs", &dir))];
        let session = TinySession::new();
        let old = ids(&discover(&session, &providers).unwrap());
        let new = ids(&discover(&session, &providers).unwrap());
        assert_ne!(old, new);
        assert!(matches!(
            session.clean_preview(old),
            Err(FfiError::PreviewInvalid { .. })
        ));
        assert!(session.clean_preview(new).is_ok());
        remove(&dir);
    }

    #[test]
    fn overlapping_selections_are_excluded_with_a_reason() {
        let dir = fixture_dir("overlap", &[], 0);
        std::fs::create_dir(dir.join("Caches")).unwrap();
        std::fs::write(dir.join("Caches/x"), b"12").unwrap();
        // `caches` lists `Caches`; `browser` finds `Caches/x` and `Caches`.
        let mut browser = Fixture::new("browser", &dir.join("Caches"));
        browser.extra = vec![dir.join("Caches")];
        let providers: Vec<Box<dyn CleanProvider>> =
            vec![Box::new(Fixture::new("caches", &dir)), Box::new(browser)];
        let session = TinySession::new();
        let discovery = discover(&session, &providers).unwrap();
        let preview = session.clean_preview(ids(&discovery)).unwrap();
        assert_eq!(preview.items.len(), 1);
        assert_eq!(
            preview.bytes_selected, 2,
            "covered bytes are not double-counted"
        );
        let kept = preview.items[0].candidate_id.clone();
        let reasons: Vec<_> = preview.excluded.iter().map(|e| e.reason.clone()).collect();
        assert!(reasons.contains(&FfiExclusionReason::Duplicate {
            kept_candidate_id: kept.clone()
        }));
        assert!(reasons.contains(&FfiExclusionReason::InsideSelected {
            parent_candidate_id: kept
        }));
        remove(&dir);
    }

    #[test]
    fn a_path_without_a_fingerprint_cannot_be_previewed() {
        let dir = fixture_dir("placeholder", &[], 0);
        let mut docker_like = Fixture::new("tool", &dir);
        docker_like.extra = vec![PathBuf::from("<docker:images>")];
        let providers: Vec<Box<dyn CleanProvider>> = vec![Box::new(docker_like)];
        let session = TinySession::new();
        let discovery = discover(&session, &providers).unwrap();
        assert!(matches!(
            session.clean_preview(ids(&discovery)),
            Err(FfiError::InvalidInput { .. })
        ));
        remove(&dir);
    }

    #[test]
    fn cancelled_discovery_stores_nothing() {
        let dir = fixture_dir("cancel", &["a"], 1);
        let providers: Vec<Box<dyn CleanProvider>> = vec![Box::new(Fixture::new("logs", &dir))];
        let session = TinySession::new();
        let token = CancellationToken::default();
        token.cancel();
        let result = session.discover_clean(
            options(),
            &providers,
            &NotRunning,
            &token,
            &Progress::default(),
        );
        assert!(matches!(result, Err(FfiError::Cancelled)));
        assert!(!session.is_busy());
        assert!(matches!(
            session.clean_preview(vec!["d1-0".into()]),
            Err(FfiError::PreviewInvalid { .. })
        ));
        remove(&dir);
    }

    #[test]
    fn discovery_is_rejected_while_another_operation_runs() {
        let session = TinySession::new();
        let _gate = session.begin().unwrap();
        assert!(matches!(discover(&session, &[]), Err(FfiError::Busy)));
    }

    #[test]
    fn invalid_options_are_rejected_before_scanning() {
        let session = TinySession::new();
        let mut bad = options();
        bad.idle_days = 0;
        let result = session.discover_clean(
            bad,
            &[],
            &NotRunning,
            &CancellationToken::default(),
            &Progress::default(),
        );
        assert!(matches!(result, Err(FfiError::InvalidInput { .. })));
    }
}
