use crate::{
    commands::{clean::*, health::*, space_lens::*, system::*},
    state::AppState,
};
use tauri::Manager;

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter("tiny_desktop=info")
        .init();
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .macos_launcher(tauri_plugin_autostart::MacosLauncher::LaunchAgent)
                .build(),
        )
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            app.manage(AppState::new(&directory)?);
            crate::tray::setup(app)?;
            crate::background::start(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            run_smart_scan,
            get_latest_scan,
            preview_cleanup,
            execute_cleanup,
            get_undo_entries,
            restore_entry,
            open_trash,
            run_space_lens,
            get_monitor,
            get_permissions,
            open_permission_settings,
            get_settings,
            save_settings,
            get_rules,
            save_rule,
            delete_rule
        ])
        .run(tauri::generate_context!());
    if let Err(error) = result {
        tracing::error!(%error, "Tiny could not start");
    }
}
