use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
pub fn setup(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Open Tiny", true, None::<&str>)?;
    let scan = MenuItem::with_id(app, "scan", "Smart Scan", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Tiny", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &scan, &quit])?;
    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))?;
    TrayIconBuilder::with_id("tiny-tray")
        .icon(icon)
        .tooltip("Tiny · ready")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_window(app),
            "scan" => {
                show_window(app);
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = crate::commands::health::run_smart_scan(app).await {
                        tracing::warn!(%error, "Tray scan failed");
                    }
                });
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
fn show_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
