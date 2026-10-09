use std::path::Path;

use crate::error::Result;

use super::{execute_per_item, CleanProvider};
use crate::clean::fs_safe::list_children;
use crate::clean::scan_context::ScanContext;
use crate::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "device-updates";
const LABEL: &str = "iPhone and iPad updates";

const UPDATE_DIRS: &[&str] = &[
    "Library/iTunes/iPhone Software Updates",
    "Library/iTunes/iPad Software Updates",
];

fn is_ipsw(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("ipsw"))
}

pub struct DeviceUpdates;

impl DeviceUpdates {
    /// One item per `.ipsw` regular file; folders, symlinks and partial
    /// downloads are left alone.
    pub(crate) fn discover_in(&self, ctx: &ScanContext<'_>, home: &Path) -> Vec<CleanItem> {
        let mut out = Vec::new();
        for dir in UPDATE_DIRS {
            for path in list_children(&home.join(dir), ctx) {
                let Ok(meta) = std::fs::symlink_metadata(&path) else {
                    continue;
                };
                if !meta.file_type().is_file() || !is_ipsw(&path) {
                    continue;
                }
                out.push(CleanItem {
                    category_id: ID.to_string(),
                    category_label: LABEL.to_string(),
                    path,
                    size: meta.len(),
                    risk: RiskLevel::Safe,
                    evidence: Vec::new(),
                });
            }
        }
        out
    }
}

impl CleanProvider for DeviceUpdates {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn inclusion_reason(&self) -> String {
        "iOS and iPadOS firmware a Mac downloaded to update a device; it is downloaded again when needed"
            .into()
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Safe
    }
    fn desktop_trash_paths(&self) -> bool {
        true
    }
    fn check_item(&self, path: &Path) -> std::result::Result<(), String> {
        if is_ipsw(path) {
            Ok(())
        } else {
            Err(format!("{} is not an .ipsw file", path.display()))
        }
    }
    fn discover(&self, ctx: &ScanContext<'_>) -> Result<Vec<CleanItem>> {
        Ok(super::home()
            .map(|h| self.discover_in(ctx, &h))
            .unwrap_or_default())
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, ID)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clean::providers::test_home::TestHome;

    #[test]
    fn offers_each_ipsw_file_and_nothing_else() {
        let home = TestHome::new("device-updates");
        let iphone = "Library/iTunes/iPhone Software Updates";
        home.file(&format!("{iphone}/iPhone15,2_17.0_Restore.ipsw"), 5);
        home.file(&format!("{iphone}/iPhone15,2_16.0_Restore.IPSW"), 3);
        home.file(&format!("{iphone}/notes.txt"), 1);
        home.dir(&format!("{iphone}/folder.ipsw"));
        home.file(
            "Library/iTunes/iPad Software Updates/iPad13,1_17.0_Restore.ipsw",
            2,
        );
        let items = DeviceUpdates.discover_in(&ScanContext::unchecked(), home.path());
        let mut found = home.relative(&items);
        found.sort();
        assert_eq!(
            found,
            [
                "Library/iTunes/iPad Software Updates/iPad13,1_17.0_Restore.ipsw",
                "Library/iTunes/iPhone Software Updates/iPhone15,2_16.0_Restore.IPSW",
                "Library/iTunes/iPhone Software Updates/iPhone15,2_17.0_Restore.ipsw",
            ]
        );
        assert_eq!(items.iter().map(|i| i.size).sum::<u64>(), 10);
        assert!(DeviceUpdates.check_item(Path::new("/a/notes.txt")).is_err());
    }
}
