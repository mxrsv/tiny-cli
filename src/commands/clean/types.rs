use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Safe,
    Review,
    Destructive,
}

impl RiskLevel {
    pub fn badge(&self) -> &'static str {
        match self {
            RiskLevel::Safe => "safe",
            RiskLevel::Review => "review",
            RiskLevel::Destructive => "destructive",
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CleanItem {
    pub category_id: String,
    pub category_label: String,
    pub path: PathBuf,
    pub size: u64,
    pub risk: RiskLevel,
}

/// What reaches a provider's execute(). Three semantics, distinct on
/// purpose so providers can't accidentally conflate them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecAction {
    Trash,
    HardDelete,
    EmptyTrash,
}

/// What the user sees in the action menu. Mapped to per-provider
/// ExecAction by execute.rs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanAction {
    Trash,
    HardDelete,
    DryRun,
    Cancel,
}

#[derive(Debug, Default)]
pub struct ExecReport {
    pub removed_paths: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,
    pub skipped_running_app: Option<String>,
}

impl ExecReport {
    pub fn merge(&mut self, other: ExecReport) {
        self.removed_paths.extend(other.removed_paths);
        self.failed.extend(other.failed);
        if self.skipped_running_app.is_none() {
            self.skipped_running_app = other.skipped_running_app;
        }
    }
}
