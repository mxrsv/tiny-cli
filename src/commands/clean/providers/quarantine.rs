use std::path::PathBuf;

use anyhow::Result;

use super::{execute_per_item, root_as_item, CleanProvider};
use crate::commands::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "quarantine";
const LABEL: &str = "Gatekeeper quarantine events";

/// Single-file path: stores Gatekeeper "open from unknown developer"
/// prompt history. Risk is Review because deleting clears the recorded
/// approvals (user may re-prompt next time).
const QUARANTINE_FILE: &str = "Library/Preferences/com.apple.LaunchServices.QuarantineEventsV2";

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub struct Quarantine;

impl CleanProvider for Quarantine {
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
        Ok(root_as_item(
            &h.join(QUARANTINE_FILE),
            ID,
            LABEL,
            RiskLevel::Review,
        ))
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
    fn quarantine_id_in_known_categories() {
        assert!(known_category_ids().contains(&ID));
    }

    #[test]
    fn quarantine_review_risk() {
        assert_eq!(Quarantine.risk(), RiskLevel::Review);
    }
}
