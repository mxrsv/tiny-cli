use std::path::PathBuf;

use anyhow::Result;

use super::{execute_per_item, root_as_item, CleanProvider};
use crate::commands::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "vscode";
const LABEL: &str = "VS Code caches";

const VSCODE_SUBDIRS: &[&str] = &[
    "Library/Application Support/Code/Cache",
    "Library/Application Support/Code/CachedData",
    "Library/Application Support/Code/logs",
];

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub struct VsCode;

impl CleanProvider for VsCode {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn available(&self) -> bool {
        let h = match home() {
            Some(h) => h,
            None => return false,
        };
        VSCODE_SUBDIRS.iter().any(|s| h.join(s).exists())
    }
    fn discover(&self) -> Result<Vec<CleanItem>> {
        let h = match home() {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        let mut items = Vec::new();
        for sub in VSCODE_SUBDIRS {
            items.extend(root_as_item(&h.join(sub), ID, LABEL, RiskLevel::Review));
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
    fn vscode_id_in_known_categories() {
        assert!(known_category_ids().contains(&ID));
    }
}
