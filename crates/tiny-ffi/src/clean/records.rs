//! Records and enums the cleanup API exposes to Swift, mirroring core data.

use tiny_core::clean::checked_execute::{ItemOutcome, SkipReason, StopReason};
use tiny_core::clean::finder_trash::TrashError;
use tiny_core::clean::providers::ReportOnly;
use tiny_core::clean::types::RiskLevel;
use tiny_core::options::CleanOptions;

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

/// What the desktop may do with a category's candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiDesktopAction {
    MoveToTrash,
    /// Shown for information; cleanup stays in the CLI.
    ReportOnly {
        reason: FfiReportOnlyReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiReportOnlyReason {
    /// Permanent deletion or Empty Trash.
    Destructive,
    /// Cleanup is a tool command (e.g. `docker system prune`), not a
    /// per-path move to Trash.
    NotPerPathTrash,
    /// The selection rule can flag data still in use (app-orphans).
    UnreliableMatch,
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
    /// Why this category's items are candidates; English diagnostic text
    /// Swift may show or replace with its own copy.
    pub inclusion_reason: String,
    /// `dev`, `user-storage` or `system`; `None` for an unregistered ID.
    pub family: Option<String>,
    pub risk: FfiRisk,
    pub status: FfiCategoryStatus,
    pub desktop_action: FfiDesktopAction,
    pub candidates: Vec<FfiCleanCandidate>,
    pub total_bytes: u64,
    /// Listing or search paths with unreadable entries.
    pub unreadable: Vec<FfiUnreadablePath>,
    /// Paths the provider found but that are never offered: the root, the
    /// home folder or its ancestors, or tool output outside the home folder.
    pub refused: Vec<FfiRefusedPath>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiRefusedPath {
    pub path: String,
    /// English diagnostic text.
    pub reason: String,
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
    /// Moving this path would also move Review-risk candidates the user did
    /// not select (PC-C3); it is kept only when all of them are selected.
    CoversUnselectedReview { candidate_ids: Vec<String> },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiTrashFailure {
    AutomationDenied,
    TrashFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiSkipReason {
    ReportOnly,
    UnknownCategory,
    Missing,
    Symlink,
    /// Replaced or edited since discovery.
    Changed,
    OutsideScanRoots,
    /// The root, the home folder or an ancestor, or a tool-reported path
    /// outside the home folder.
    ProtectedPath,
    /// On another volume than the home folder, where Finder may delete
    /// permanently instead of moving to Trash.
    NotOnHomeVolume,
    ProviderGuard,
    AppRunning,
    /// A required safety check could not run.
    SafetyCheckUnavailable,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum FfiItemOutcome {
    MovedToTrash,
    /// The source is re-checked after the failure; nothing else is tried.
    Failed {
        reason: FfiTrashFailure,
        detail: String,
        source_still_present: bool,
    },
    Skipped {
        reason: FfiSkipReason,
        /// The guard message, app name or failed check, when there is one.
        detail: Option<String>,
    },
    NotAttempted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiStopReason {
    Cancelled,
    AutomationDenied,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiItemResult {
    pub candidate_id: String,
    pub category_id: String,
    pub path: String,
    pub size_bytes: u64,
    pub outcome: FfiItemOutcome,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiExecReport {
    /// One entry per preview item, in preview order.
    pub results: Vec<FfiItemResult>,
    /// Discovery-time sizes of every preview item.
    pub bytes_selected: u64,
    /// Discovery-time sizes of the moved items. Trash keeps them on disk,
    /// so this is not freed space.
    pub bytes_moved_to_trash: u64,
    pub moved_count: u64,
    pub failed_count: u64,
    pub skipped_count: u64,
    pub not_attempted_count: u64,
    pub stopped: Option<FfiStopReason>,
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

impl From<ReportOnly> for FfiReportOnlyReason {
    fn from(reason: ReportOnly) -> Self {
        match reason {
            ReportOnly::Destructive => Self::Destructive,
            ReportOnly::NotPerPathTrash => Self::NotPerPathTrash,
            ReportOnly::UnreliableMatch => Self::UnreliableMatch,
        }
    }
}

impl From<StopReason> for FfiStopReason {
    fn from(reason: StopReason) -> Self {
        match reason {
            StopReason::Cancelled => Self::Cancelled,
            StopReason::AutomationDenied => Self::AutomationDenied,
        }
    }
}

impl From<ItemOutcome> for FfiItemOutcome {
    fn from(outcome: ItemOutcome) -> Self {
        match outcome {
            ItemOutcome::MovedToTrash => Self::MovedToTrash,
            ItemOutcome::Failed {
                error,
                source_present,
            } => {
                let (reason, detail) = match error {
                    TrashError::AutomationDenied(d) => (FfiTrashFailure::AutomationDenied, d),
                    TrashError::Failed(d) => (FfiTrashFailure::TrashFailed, d),
                };
                Self::Failed {
                    reason,
                    detail,
                    source_still_present: source_present,
                }
            }
            ItemOutcome::Skipped(reason) => {
                let (reason, detail) = skip_reason(reason);
                Self::Skipped { reason, detail }
            }
            ItemOutcome::NotAttempted => Self::NotAttempted,
        }
    }
}

fn skip_reason(reason: SkipReason) -> (FfiSkipReason, Option<String>) {
    match reason {
        SkipReason::ReportOnly => (FfiSkipReason::ReportOnly, None),
        SkipReason::UnknownCategory => (FfiSkipReason::UnknownCategory, None),
        SkipReason::Missing => (FfiSkipReason::Missing, None),
        SkipReason::Symlink => (FfiSkipReason::Symlink, None),
        SkipReason::Changed => (FfiSkipReason::Changed, None),
        SkipReason::OutsideRoots => (FfiSkipReason::OutsideScanRoots, None),
        SkipReason::ProtectedPath(d) => (FfiSkipReason::ProtectedPath, Some(d)),
        SkipReason::NotOnHomeVolume => (FfiSkipReason::NotOnHomeVolume, None),
        SkipReason::ProviderGuard(d) => (FfiSkipReason::ProviderGuard, Some(d)),
        SkipReason::AppRunning(app) => (FfiSkipReason::AppRunning, Some(app)),
        SkipReason::SafetyCheckFailed(d) => (FfiSkipReason::SafetyCheckUnavailable, Some(d)),
    }
}
