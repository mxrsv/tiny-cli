use crate::{
    error::{ErrorPayload, Result},
    models::{MonitorReport, Rule, Settings},
    permissions::PermissionStatus,
    state::{lock, AppState},
};
use tauri::Manager;
#[tauri::command]
pub async fn get_monitor(app: tauri::AppHandle) -> Result<MonitorReport> {
    tauri::async_runtime::spawn_blocking(move || crate::service::monitor(&app.state::<AppState>()))
        .await
        .map_err(|e| ErrorPayload::new("worker_error", e))?
}
#[tauri::command]
pub fn get_permissions() -> PermissionStatus {
    crate::permissions::status()
}
#[tauri::command]
pub fn open_permission_settings() -> Result<()> {
    crate::permissions::open_settings()
}
#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> Result<Settings> {
    crate::settings::load(&app)
}
#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<Settings> {
    crate::settings::save(&app, settings)
}
#[tauri::command]
pub async fn get_rules(app: tauri::AppHandle) -> Result<Vec<Rule>> {
    tauri::async_runtime::spawn_blocking(move || lock(&app.state::<AppState>().db)?.rules())
        .await
        .map_err(|e| ErrorPayload::new("worker_error", e))?
}
#[tauri::command]
pub async fn save_rule(app: tauri::AppHandle, rule: Rule) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || {
        lock(&app.state::<AppState>().db)?.save_rule(&rule)
    })
    .await
    .map_err(|e| ErrorPayload::new("worker_error", e))?
}
#[tauri::command]
pub async fn delete_rule(app: tauri::AppHandle, rule_id: String) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || {
        lock(&app.state::<AppState>().db)?.delete_rule(&rule_id)
    })
    .await
    .map_err(|e| ErrorPayload::new("worker_error", e))?
}
