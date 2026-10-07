use crate::{
    error::{ErrorPayload, Result},
    models::{CleanupPreview, CleanupResult, QuarantineEntry},
    service, settings,
    state::{lock, AppState},
};
use tauri::{Emitter, Manager};
#[tauri::command]
pub async fn preview_cleanup(
    app: tauri::AppHandle,
    scan_id: String,
    selected_paths: Vec<String>,
) -> Result<CleanupPreview> {
    let idle_days = settings::load(&app)?.idle_days;
    tauri::async_runtime::spawn_blocking(move || {
        service::preview_cleanup(
            &app.state::<AppState>(),
            &scan_id,
            &selected_paths,
            idle_days,
        )
    })
    .await
    .map_err(|e| ErrorPayload::new("worker_error", e))?
}
#[tauri::command]
pub async fn execute_cleanup(
    app: tauri::AppHandle,
    preview_id: String,
    confirmed: bool,
) -> Result<CleanupResult> {
    crate::permissions::ensure_access()?;
    crate::events::blocking(app, "clean", move |app| {
        let progress = |payload| {
            let _ = app.emit("clean:progress", payload);
        };
        service::execute_cleanup(
            &app.state::<AppState>(),
            &preview_id,
            confirmed,
            Some(&progress),
        )
    })
    .await
}

#[tauri::command]
pub async fn get_undo_entries(app: tauri::AppHandle) -> Result<Vec<QuarantineEntry>> {
    tauri::async_runtime::spawn_blocking(move || lock(&app.state::<AppState>().db)?.entries())
        .await
        .map_err(|e| ErrorPayload::new("worker_error", e))?
}
#[tauri::command]
pub async fn restore_entry(app: tauri::AppHandle, entry_id: String) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || {
        service::restore_entry(&app.state::<AppState>(), &entry_id)
    })
    .await
    .map_err(|e| ErrorPayload::new("worker_error", e))?
}
#[tauri::command]
pub fn open_trash() -> Result<()> {
    let output = std::process::Command::new("osascript")
        .args(["-e", "tell application \"Finder\" to open trash"])
        .output()?;
    if !output.status.success() {
        return Err(ErrorPayload::new(
            "finder_error",
            String::from_utf8_lossy(&output.stderr),
        ));
    }
    Ok(())
}
