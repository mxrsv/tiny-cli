use crate::error::{ErrorPayload, Result};
use serde::Serialize;
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionStatus {
    pub status: String,
    pub home_path: String,
    pub explanation: String,
}
/// macOS has no public FDA query. Probe existing TCC-protected directories and report uncertainty honestly.
pub fn status() -> PermissionStatus {
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_default();
    let mut verified = false;
    for path in [
        home.join("Library/Mail"),
        home.join("Library/Safari"),
        home.join("Library/Application Support/com.apple.TCC"),
    ] {
        match std::fs::read_dir(&path) {
            Ok(_) => verified = true,
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => return PermissionStatus { status: "required".into(), home_path: home.display().to_string(), explanation: "macOS denied access to a protected folder. Add Tiny to Full Disk Access and restart the app.".into() },
            Err(_) => {},
        }
    }
    PermissionStatus { status: if verified { "granted" } else { "unknown" }.into(), home_path: home.display().to_string(), explanation: if verified { "Protected folders are readable. Some individual paths can still be inaccessible." } else { "No existing protected folder could verify Full Disk Access. Scans may be incomplete; you can grant access in System Settings." }.into() }
}
pub fn ensure_access() -> Result<()> {
    if status().status == "required" {
        return Err(ErrorPayload::new(
            "permission_required",
            "Grant Full Disk Access and restart Tiny before scanning.",
        ));
    }
    Ok(())
}
pub fn open_settings() -> Result<()> {
    let status = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles")
        .status()?;
    if !status.success() {
        return Err(ErrorPayload::new(
            "open_settings_failed",
            "Could not open System Settings.",
        ));
    }
    Ok(())
}
