use serde::{Deserialize, Serialize};
use tiny_core::{clean::discover::DiscoveryReport, health::HealthScore, sys::SystemInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartScan {
    pub id: String,
    pub created_at: String,
    pub system: SystemInfo,
    pub discovery: DiscoveryReport,
    pub health: HealthScore,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub id: String,
    pub created_at: String,
    pub used_bytes: u64,
    pub reclaimable_bytes: u64,
    pub health_score: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorReport {
    pub system: SystemInfo,
    pub history: Vec<ScanSummary>,
    pub weekly_delta_bytes: Option<i64>,
    pub days_until_full: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPreview {
    pub id: String,
    pub items: Vec<PreviewItem>,
    pub total_bytes: u64,
    pub equivalent_command: String,
    pub expires_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewItem {
    pub category_id: String,
    pub category_label: String,
    pub path: String,
    pub bytes: u64,
    pub risk: tiny_core::clean::types::RiskLevel,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    pub operation_id: String,
    pub moved_count: usize,
    pub moved_bytes: u64,
    pub failed: Vec<(String, String)>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuarantineEntry {
    pub id: String,
    pub operation_id: String,
    pub original_path: String,
    pub stored_path: Option<String>,
    pub category_id: String,
    pub bytes: u64,
    pub created_at: String,
    pub expires_at: String,
    pub method: String,
    pub status: String,
    pub error: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub category_id: String,
    pub min_bytes: u64,
    pub enabled: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub scan_interval_hours: u32,
    pub notifications: bool,
    pub launch_at_login: bool,
    pub watch_changes: bool,
    pub idle_days: u64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            scan_interval_hours: 168,
            notifications: true,
            launch_at_login: false,
            watch_changes: false,
            idle_days: 30,
        }
    }
}
pub fn iso_time(epoch: i64) -> String {
    chrono::DateTime::from_timestamp(epoch, 0)
        .map(|d| d.to_rfc3339())
        .unwrap_or_default()
}
pub fn primary_disk(system: &SystemInfo) -> Option<&tiny_core::sys::DiskInfo> {
    system
        .disks
        .iter()
        .find(|d| d.mount_point == "/")
        .or_else(|| system.disks.first())
}
