use crate::{
    error::Result,
    models::SmartScan,
    permissions, service, settings,
    state::{lock, AppState},
};
use tauri::{Emitter, Manager};
#[tauri::command]
pub async fn run_smart_scan(app: tauri::AppHandle) -> Result<SmartScan> {
    permissions::ensure_access()?;
    let idle_days = settings::load(&app)?.idle_days;
    crate::events::blocking(app, "scan", move |app| {
        let state = app.state::<AppState>();
        let progress = |payload| {
            let _ = app.emit("scan:progress", payload);
        };
        service::smart_scan(&state, idle_days, Some(&progress))
    })
    .await
}
#[tauri::command]
pub fn get_latest_scan(state: tauri::State<'_, AppState>) -> Result<Option<SmartScan>> {
    Ok(lock(&state.latest)?.clone())
}
