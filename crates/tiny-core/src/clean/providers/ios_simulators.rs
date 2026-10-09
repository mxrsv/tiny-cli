use std::path::{Path, PathBuf};

use crate::error::Result;

use super::{execute_per_item, top_level_entries, CleanProvider};
use crate::clean::fs_safe::is_dir_safe;
use crate::clean::scan_context::ScanContext;
use crate::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "ios-simulators";
const LABEL: &str = "iOS Simulator caches/devices";
const APP: &str = "Xcode";

const SIM_DIRS: &[&str] = &[
    "Library/Developer/CoreSimulator/Caches",
    "Library/Developer/CoreSimulator/Devices",
];

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub struct IosSimulators;

impl CleanProvider for IosSimulators {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn inclusion_reason(&self) -> String {
        "Simulator caches and devices Xcode can recreate".into()
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn desktop_trash_paths(&self) -> bool {
        true
    }
    fn requires_app_quit(&self) -> Option<&'static str> {
        Some(APP)
    }
    fn available(&self) -> bool {
        let h = match home() {
            Some(h) => h,
            None => return false,
        };
        SIM_DIRS.iter().any(|s| h.join(s).is_dir())
    }
    fn discover(&self, ctx: &ScanContext<'_>) -> Result<Vec<CleanItem>> {
        let h = match home() {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        Ok(SIM_DIRS
            .iter()
            .flat_map(|sub| simulator_entries(ctx, &h.join(sub)))
            .collect())
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, ID)
    }
}

/// Folders directly inside `root`. Files are skipped: `Devices/` also holds
/// `device_set.plist`, the registry CoreSimulator reads to list devices.
fn simulator_entries(ctx: &ScanContext<'_>, root: &Path) -> Vec<CleanItem> {
    top_level_entries(ctx, root, ID, LABEL, RiskLevel::Review)
        .into_iter()
        .filter(|item| is_dir_safe(&item.path))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clean::providers::known_category_ids;

    #[test]
    fn device_registry_plist_is_never_a_candidate() {
        let root = std::env::temp_dir().join(format!("tiny-sim-{}", std::process::id()));
        let device = root.join("01DADBA3-1A21-4914-A093-29B4D0D7B9C8");
        std::fs::create_dir_all(&device).unwrap();
        std::fs::write(root.join("device_set.plist"), b"<plist/>").unwrap();
        let items = simulator_entries(&ScanContext::unchecked(), &root);
        let paths: Vec<_> = items.into_iter().map(|i| i.path).collect();
        assert_eq!(paths, vec![device]);
        let _ = crate::clean::fs_safe::remove_recursive_safe(&root);
    }

    #[test]
    fn ios_simulators_id_in_known_categories() {
        assert!(known_category_ids().contains(&ID));
    }

    #[test]
    fn ios_simulators_gates_xcode() {
        let p = IosSimulators;
        assert_eq!(p.requires_app_quit(), Some("Xcode"));
    }
}
