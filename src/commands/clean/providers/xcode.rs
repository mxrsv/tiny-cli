use anyhow::Result;
use std::path::PathBuf;

use super::{execute_per_item, root_as_item, CleanProvider};
use crate::commands::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const APP: &str = "Xcode";

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub struct XcodeDerivedData;

const DERIVED_ID: &str = "xcode-derived";
const DERIVED_LABEL: &str = "Xcode DerivedData";

impl CleanProvider for XcodeDerivedData {
    fn id(&self) -> &'static str {
        DERIVED_ID
    }
    fn label(&self) -> &'static str {
        DERIVED_LABEL
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Safe
    }
    fn requires_app_quit(&self) -> Option<&'static str> {
        Some(APP)
    }
    fn discover(&self) -> Result<Vec<CleanItem>> {
        let h = match home() {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        let root = h.join("Library/Developer/Xcode/DerivedData");
        Ok(root_as_item(&root, DERIVED_ID, DERIVED_LABEL, RiskLevel::Safe))
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, DERIVED_ID)
    }
}

pub struct XcodeArchives;

const ARCHIVES_ID: &str = "xcode-archives";
const ARCHIVES_LABEL: &str = "Xcode Archives";

impl CleanProvider for XcodeArchives {
    fn id(&self) -> &'static str {
        ARCHIVES_ID
    }
    fn label(&self) -> &'static str {
        ARCHIVES_LABEL
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn requires_app_quit(&self) -> Option<&'static str> {
        Some(APP)
    }
    fn discover(&self) -> Result<Vec<CleanItem>> {
        let h = match home() {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        let root = h.join("Library/Developer/Xcode/Archives");
        Ok(root_as_item(
            &root,
            ARCHIVES_ID,
            ARCHIVES_LABEL,
            RiskLevel::Review,
        ))
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, ARCHIVES_ID)
    }
}

pub struct XcodeDeviceSupport;

const DS_ID: &str = "xcode-devicesupport";
const DS_LABEL: &str = "Xcode iOS DeviceSupport";

impl CleanProvider for XcodeDeviceSupport {
    fn id(&self) -> &'static str {
        DS_ID
    }
    fn label(&self) -> &'static str {
        DS_LABEL
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn requires_app_quit(&self) -> Option<&'static str> {
        Some(APP)
    }
    fn discover(&self) -> Result<Vec<CleanItem>> {
        let h = match home() {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        let root = h.join("Library/Developer/Xcode/iOS DeviceSupport");
        Ok(root_as_item(&root, DS_ID, DS_LABEL, RiskLevel::Review))
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, DS_ID)
    }
}
