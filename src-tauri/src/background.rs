use crate::{
    service, settings,
    state::{lock, AppState},
};
use notify::{RecursiveMode, Watcher};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

pub fn start(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let changed = Arc::new(AtomicBool::new(false));
        let signal = changed.clone();
        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                if event
                    .map(|e| !matches!(e.kind, notify::EventKind::Access(_)))
                    .unwrap_or(false)
                {
                    signal.store(true, Ordering::Release);
                }
            })
            .ok();
        let mut watching = false;
        let mut last_scan = Instant::now();
        let mut last_live = Instant::now();
        loop {
            std::thread::sleep(Duration::from_secs(30));
            let preferences = match settings::load(&app) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(%e, "Could not read background settings");
                    continue;
                }
            };
            if preferences.watch_changes != watching {
                if let Some(watcher) = watcher.as_mut() {
                    if let Some(home) = std::env::var_os("HOME") {
                        let home = std::path::PathBuf::from(home);
                        for folder in [
                            "Downloads",
                            "Desktop",
                            "Documents",
                            "Library/Caches",
                            "Library/Logs",
                        ] {
                            let path = home.join(folder);
                            if preferences.watch_changes {
                                let _ = watcher.watch(&path, RecursiveMode::Recursive);
                            } else {
                                let _ = watcher.unwatch(&path);
                            }
                        }
                    }
                }
                watching = preferences.watch_changes;
            }
            let last_saved = lock(&app.state::<AppState>().db)
                .and_then(|db| db.history())
                .ok()
                .and_then(|history| history.first().cloned())
                .and_then(|scan| chrono::DateTime::parse_from_rfc3339(&scan.created_at).ok());
            let seconds_since_scan = last_saved
                .map(|time| {
                    chrono::Utc::now()
                        .signed_duration_since(time)
                        .num_seconds()
                        .max(0) as u64
                })
                .unwrap_or_else(|| last_scan.elapsed().as_secs());
            let scheduled = preferences.scan_interval_hours > 0
                && seconds_since_scan >= u64::from(preferences.scan_interval_hours) * 3600;
            let live = preferences.watch_changes
                && last_live.elapsed() >= Duration::from_secs(300)
                && changed.swap(false, Ordering::AcqRel);
            if (scheduled || live) && crate::permissions::ensure_access().is_ok() {
                let progress = |payload| {
                    let _ = app.emit("scan:progress", payload);
                };
                match service::smart_scan(
                    &app.state::<AppState>(),
                    preferences.idle_days,
                    Some(&progress),
                ) {
                    Ok(scan) => {
                        last_scan = Instant::now();
                        last_live = Instant::now();
                        let _ = app.emit("scan:done", &scan);
                        let rules = lock(&app.state::<AppState>().db)
                            .and_then(|db| db.rules())
                            .unwrap_or_default();
                        let triggered = rules.iter().filter(|r| r.enabled).any(|r| {
                            scan.discovery
                                .groups
                                .iter()
                                .any(|g| g.id == r.category_id && g.total_size >= r.min_bytes)
                        });
                        if preferences.notifications && (triggered || scan.health.score < 60) {
                            let _ = app
                                .notification()
                                .builder()
                                .title("Tiny · a little room to breathe")
                                .body(format!(
                                    "Health score {}. Review cleanup opportunities in Tiny.",
                                    scan.health.score
                                ))
                                .show();
                        }
                    }
                    Err(e) => {
                        let _ = app.emit("scan:error", &e);
                        tracing::warn!(%e, "Background scan skipped");
                    }
                }
            }
            // The tray remains useful while the main window is hidden.
            let info = tiny_core::sys::information();
            if let Some(tray) = app.tray_by_id("tiny-tray") {
                let memory = if info.memory_total == 0 {
                    0.0
                } else {
                    info.memory_used as f64 / info.memory_total as f64 * 100.0
                };
                let _ = tray.set_tooltip(Some(format!(
                    "Tiny · CPU {:.0}% · memory {:.0}%",
                    info.cpu_usage, memory
                )));
            }
        }
    });
}
