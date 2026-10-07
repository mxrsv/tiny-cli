use crate::{error::Result, state::AppState};
use tauri::{Emitter, Manager};
use tiny_core::space_lens::SpaceReport;
#[tauri::command]
pub async fn run_space_lens(app: tauri::AppHandle, path: String) -> Result<SpaceReport> {
    crate::permissions::ensure_access()?;
    crate::events::blocking(app, "space-lens", move |app| {
        let state = app.state::<AppState>();
        let _guard = state.begin()?;
        let progress = |payload| {
            let _ = app.emit("space-lens:progress", payload);
        };
        tiny_core::space_lens::scan(std::path::Path::new(&path), Some(&progress))
            .map_err(Into::into)
    })
    .await
}
