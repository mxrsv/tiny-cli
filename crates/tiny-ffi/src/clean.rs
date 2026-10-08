//! Cleanup discovery, trusted preview and Trash execution for the native app.
//!
//! The session keeps what discovery found (paths, fingerprints, roots and
//! the options used). A preview is built only from those candidate IDs, and
//! execution only from a preview ID, so Swift never hands Rust a path to act
//! on. Execution moves each path through the core `Trash` route and never
//! calls a provider's own cleanup.

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::{Arc, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use tiny_core::clean::checked_execute::{
    execute_checked, CheckedExecReport, ExecContext, ItemOutcome,
};
use tiny_core::clean::discover::{
    discover_checked, select_providers_with, validate_options, CategoryOutcome, CheckedCategory,
};
use tiny_core::clean::finder_trash::{FinderTrash, Trash};
use tiny_core::clean::fs_safe::{fingerprint, PathFingerprint};
use tiny_core::clean::process::{AppProbe, RunnerProbe};
use tiny_core::clean::providers::{
    all_providers_with, category_family, desktop_report_only, known_category_ids, CleanProvider,
    ReportOnly,
};
use tiny_core::clean::scan_context::{ScanContext, Unreadable};
use tiny_core::clean::trash_plan::{home_dir, partition_overlaps, Overlap, PlannedItem};
use tiny_core::clean::types::CleanItem;
use tiny_core::options::CleanOptions;
use tiny_core::runner::{CommandRunner, ToolLookup, ToolRunner};

use crate::session::CancellationToken;
use crate::{FfiError, ProgressListener, TinySession};

mod records;
#[cfg(test)]
mod tests;

pub use records::*;

/// How long a confirmed preview stays executable.
pub const PREVIEW_TTL: Duration = Duration::from_secs(15 * 60);

/// Session-held cleanup state. Never locked across provider work, Trash
/// calls or progress callbacks.
#[derive(Default)]
pub(crate) struct CleanState {
    next_id: u64,
    discovery: Option<StoredDiscovery>,
    preview: Option<StoredPreview>,
    /// Test override for `PREVIEW_TTL`.
    ttl: Option<Duration>,
    /// Test override for the home folder read from `HOME`.
    home: Option<Option<PathBuf>>,
}

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
    desktop_action: FfiDesktopAction,
}

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
        let (mut kept, covered) = partition_overlaps(planned, |(_, plan)| plan.item.path.as_path());
        let excluded = covered
            .into_iter()
            .map(|((id, plan), overlap)| {
                let excluded = exclusion(&kept, id, &plan, overlap);
                // The kept path moves this one too, so it inherits its gates.
                let (Overlap::Duplicate { kept: index } | Overlap::Inside { kept: index }) =
                    overlap;
                kept[index].1.covers.push(plan.item);
                excluded
            })
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

    /// Moves the preview's paths to Trash through Finder. The preview is
    /// consumed before the first move, so it can never run twice, even
    /// after a panic. Always returns the per-item report once execution
    /// starts; a Finder Automation denial stops it with
    /// `stopped = AutomationDenied`.
    pub fn clean_execute(
        &self,
        preview_id: String,
        token: Arc<CancellationToken>,
        progress: Arc<dyn ProgressListener>,
    ) -> Result<FfiExecReport, FfiError> {
        let runner = app_runner();
        let probe = RunnerProbe(runner.as_ref());
        let providers = |options: &CleanOptions| all_providers_with(options, runner.clone());
        self.execute_clean(
            &preview_id,
            &providers,
            &probe,
            &FinderTrash,
            &token,
            progress.as_ref(),
        )
    }
}

/// The app's runner: `PATH`, then the Homebrew prefixes.
fn app_runner() -> Arc<dyn CommandRunner> {
    Arc::new(ToolRunner::new(ToolLookup::app()))
}

type ProviderFactory<'a> = &'a dyn Fn(&CleanOptions) -> Vec<Box<dyn CleanProvider>>;

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
        let home = self.clean_home();
        let ctx = ScanContext::new(Some(token.flag()), probe).with_home(home);
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
            .map(|category| {
                let action = desktop_action(providers, &category.id);
                store_category(id, category, action, &mut candidates)
            })
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

    /// `clean_execute` with injected providers, probe and Trash (the test
    /// seam).
    pub(crate) fn execute_clean(
        &self,
        preview_id: &str,
        providers: ProviderFactory<'_>,
        probe: &dyn AppProbe,
        trash: &dyn Trash,
        token: &CancellationToken,
        progress: &dyn ProgressListener,
    ) -> Result<FfiExecReport, FfiError> {
        let _gate = self.begin()?;
        let (items, options) = self.consume_preview(preview_id)?;
        let providers = providers(&options);
        let plan: Vec<PlannedItem> = items.iter().map(|(_, plan)| plan.clone()).collect();
        let report = |p: tiny_core::progress::Progress| progress.on_progress(p.into());
        let home = self.clean_home();
        let ctx = ExecContext {
            providers: &providers,
            home: home.as_deref(),
            probe,
            trash,
            cancel: Some(token.flag()),
            progress: Some(&report),
        };
        let executed = execute_checked(&plan, &ctx);
        Ok(exec_report(&items, executed))
    }

    fn clean_home(&self) -> Option<PathBuf> {
        self.clean_state().home.clone().unwrap_or_else(home_dir)
    }

    /// Marks the preview consumed, under the lock, before any mutation.
    fn consume_preview(
        &self,
        preview_id: &str,
    ) -> Result<(Vec<(String, PlannedItem)>, CleanOptions), FfiError> {
        let invalid = |detail: &str| FfiError::PreviewInvalid {
            detail: detail.into(),
        };
        let mut state = self.clean_state();
        let state = &mut *state;
        let discovery = state.discovery.as_ref();
        let preview = state
            .preview
            .as_mut()
            .filter(|p| p.id == preview_id)
            .ok_or_else(|| invalid("unknown or replaced preview"))?;
        if preview.consumed {
            return Err(invalid("preview already used"));
        }
        if Instant::now() >= preview.expires_at {
            return Err(invalid("preview expired"));
        }
        let discovery = discovery
            .filter(|d| d.id == preview.discovery_id)
            .ok_or_else(|| invalid("discovery was replaced"))?;
        preview.consumed = true;
        Ok((preview.items.clone(), discovery.options.clone()))
    }
}

fn desktop_action(providers: &[Box<dyn CleanProvider>], id: &str) -> FfiDesktopAction {
    let reason = match providers.iter().find(|p| p.id() == id) {
        Some(provider) => desktop_report_only(provider.as_ref()),
        None => Some(ReportOnly::NotPerPathTrash),
    };
    match reason {
        None => FfiDesktopAction::MoveToTrash,
        Some(reason) => FfiDesktopAction::ReportOnly {
            reason: reason.into(),
        },
    }
}

fn store_category(
    discovery_id: u64,
    category: CheckedCategory,
    desktop_action: FfiDesktopAction,
    candidates: &mut HashMap<String, StoredCandidate>,
) -> FfiCleanCategory {
    let family = known_category_ids()
        .contains(&category.id.as_str())
        .then(|| category_family(&category.id).id().to_string());
    let mut ffi = FfiCleanCategory {
        id: category.id,
        label: category.label,
        inclusion_reason: category.inclusion_reason,
        family,
        risk: category.risk.into(),
        status: FfiCategoryStatus::Found,
        desktop_action,
        candidates: Vec::new(),
        total_bytes: 0,
        unreadable: Vec::new(),
        refused: Vec::new(),
    };
    match category.outcome {
        CategoryOutcome::Found {
            items,
            unreadable,
            roots,
            refused,
        } => {
            ffi.refused = refused
                .into_iter()
                .map(|(path, reason)| FfiRefusedPath {
                    path: path.to_string_lossy().into_owned(),
                    reason,
                })
                .collect();
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
                        desktop_action,
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
            let path = candidate.item.path.display();
            if candidate.desktop_action != FfiDesktopAction::MoveToTrash {
                return Err(FfiError::InvalidInput {
                    detail: format!("{path} is report-only on the desktop"),
                });
            }
            let fingerprint =
                candidate
                    .fingerprint
                    .clone()
                    .ok_or_else(|| FfiError::InvalidInput {
                        detail: format!("{path} could not be inspected at discovery"),
                    })?;
            let plan = PlannedItem {
                item: candidate.item.clone(),
                fingerprint,
                roots: candidate.roots.as_ref().clone(),
                covers: Vec::new(),
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

fn exec_report(items: &[(String, PlannedItem)], executed: CheckedExecReport) -> FfiExecReport {
    let count = |keep: fn(&ItemOutcome) -> bool| executed.count(keep) as u64;
    let summary = FfiExecReport {
        results: Vec::new(),
        bytes_selected: executed.bytes_selected(),
        bytes_moved_to_trash: executed.bytes_moved_to_trash(),
        moved_count: executed.moved_count() as u64,
        failed_count: count(|o| matches!(o, ItemOutcome::Failed { .. })),
        skipped_count: count(|o| matches!(o, ItemOutcome::Skipped(_))),
        not_attempted_count: count(|o| matches!(o, ItemOutcome::NotAttempted)),
        stopped: executed.stopped.clone().map(Into::into),
    };
    let results = items
        .iter()
        .zip(executed.results)
        .map(|((id, _), result)| FfiItemResult {
            candidate_id: id.clone(),
            category_id: result.item.category_id,
            path: result.item.path.to_string_lossy().into_owned(),
            size_bytes: result.item.size,
            outcome: result.outcome.into(),
        })
        .collect();
    FfiExecReport { results, ..summary }
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
