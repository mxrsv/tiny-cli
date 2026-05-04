use std::path::PathBuf;

use anyhow::Result;

use super::{execute_per_item, top_level_entries, CleanProvider};
use crate::commands::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

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
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
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
    fn discover(&self) -> Result<Vec<CleanItem>> {
        let h = match home() {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        let mut items = Vec::new();
        for sub in SIM_DIRS {
            items.extend(top_level_entries(
                &h.join(sub),
                ID,
                LABEL,
                RiskLevel::Review,
            ));
        }
        Ok(items)
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, ID)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::clean::providers::known_category_ids;

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
