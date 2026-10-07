use crate::{
    error::{ErrorPayload, Result},
    models::Settings,
};
use tauri_plugin_store::StoreExt;

pub fn load(app: &tauri::AppHandle) -> Result<Settings> {
    let store = app
        .store("settings.json")
        .map_err(|e| ErrorPayload::new("settings_error", e))?;
    store
        .get("preferences")
        .map(|value| serde_json::from_value(value).map_err(Into::into))
        .unwrap_or_else(|| Ok(Settings::default()))
}
pub fn save(app: &tauri::AppHandle, settings: Settings) -> Result<Settings> {
    use tauri_plugin_autostart::ManagerExt;
    if settings.idle_days == 0 || settings.idle_days > 36500 || settings.scan_interval_hours > 8760
    {
        return Err(ErrorPayload::new(
            "invalid_settings",
            "Idle days must be 1–36500; scan interval must be 0–8760 hours.",
        ));
    }
    if settings.launch_at_login {
        app.autolaunch().enable()
    } else {
        app.autolaunch().disable()
    }
    .map_err(|e| ErrorPayload::new("autostart_error", e))?;
    let store = app
        .store("settings.json")
        .map_err(|e| ErrorPayload::new("settings_error", e))?;
    store.set("preferences", serde_json::to_value(&settings)?);
    store
        .save()
        .map_err(|e| ErrorPayload::new("settings_error", e))?;
    Ok(settings)
}
