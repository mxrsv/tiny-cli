use crate::error::{ErrorPayload, Result};
use tauri::Emitter;

/// Always finish the event stream, including worker panics and engine errors.
pub async fn blocking<T, F>(app: tauri::AppHandle, domain: &'static str, job: F) -> Result<T>
where
    T: serde::Serialize + Clone + Send + 'static,
    F: FnOnce(tauri::AppHandle) -> Result<T> + Send + 'static,
{
    let worker_app = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || job(worker_app))
        .await
        .map_err(|error| ErrorPayload::new("worker_error", error))
        .and_then(|result| result);
    match &result {
        Ok(payload) => {
            let _ = app.emit(&format!("{domain}:done"), payload);
        }
        Err(error) => {
            let _ = app.emit(&format!("{domain}:error"), error);
        }
    }
    result
}
