use crate::{
    delete::{self, NativeTrash, Trash},
    error::{ErrorPayload, Result},
    models::*,
    state::{lock, AppState, PendingPlan},
};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    time::{Duration, Instant},
};
use tiny_core::{
    clean::{
        discover::{discover, discover_with_progress},
        fs_safe::dir_size_safe,
    },
    options::CleanOptions,
    progress::{report, ProgressCallback},
};

pub fn smart_scan(
    state: &AppState,
    idle_days: u64,
    progress: ProgressCallback<'_>,
) -> Result<SmartScan> {
    let _guard = state.begin()?;
    let system = tiny_core::sys::information();
    let opts = CleanOptions {
        include_review: true,
        include_destructive: true,
        idle_days,
        ..Default::default()
    };
    let mut discovery = discover_with_progress(&opts, progress)?;
    discovery.groups.sort_by(|a, b| {
        risk_order(a.risk)
            .cmp(&risk_order(b.risk))
            .then(b.total_size.cmp(&a.total_size))
    });
    let health = tiny_core::health::score(&system, &discovery);
    let result = SmartScan {
        id: uuid::Uuid::new_v4().to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        system,
        discovery,
        health,
    };
    lock(&state.db)?.save_scan(&result)?;
    *lock(&state.latest)? = Some(result.clone());
    lock(&state.plans)?.clear();
    Ok(result)
}
fn risk_order(risk: tiny_core::clean::types::RiskLevel) -> u8 {
    use tiny_core::clean::types::RiskLevel::*;
    match risk {
        Safe => 0,
        Review => 1,
        Destructive => 2,
    }
}

pub fn preview_cleanup(
    state: &AppState,
    scan_id: &str,
    selected_paths: &[String],
    idle_days: u64,
) -> Result<CleanupPreview> {
    let _guard = state.begin()?;
    if selected_paths.is_empty() {
        return Err(ErrorPayload::new(
            "empty_selection",
            "Select at least one path.",
        ));
    }
    let latest = lock(&state.latest)?;
    let scan = latest.as_ref().filter(|s| s.id == scan_id).ok_or_else(|| {
        ErrorPayload::new(
            "stale_scan",
            "Run Smart Scan before creating a cleanup plan.",
        )
    })?;
    let selected: HashSet<&str> = selected_paths.iter().map(String::as_str).collect();
    let mut fingerprints = HashMap::new();
    let mut items = Vec::new();
    for group in &scan.discovery.groups {
        for item in &group.items {
            let path = item.path.to_str().ok_or_else(|| {
                ErrorPayload::new("invalid_path", "Path cannot be represented as UTF-8.")
            })?;
            if !selected.contains(path) {
                continue;
            }
            if let Some(app_data) = state.quarantine_root.parent() {
                if item.path.starts_with(app_data) || app_data.starts_with(&item.path) {
                    return Err(ErrorPayload::new(
                        "protected_path",
                        "Tiny cannot clean its own data folder.",
                    ));
                }
            }
            if !tiny_core::health::is_recoverable(&group.id) {
                return Err(ErrorPayload::new(
                    "irreversible_category",
                    "This category is report-only because it cannot be undone.",
                ));
            }
            // Fingerprints come from the filesystem and stored scan, never from the frontend.
            fingerprints.insert(path.to_owned(), delete::fingerprint(&item.path)?);
            items.push(PreviewItem {
                category_id: group.id.clone(),
                category_label: group.label.clone(),
                path: path.into(),
                bytes: dir_size_safe(&item.path),
                risk: item.risk,
            });
        }
    }
    if fingerprints.len() != selected.len() {
        return Err(ErrorPayload::new(
            "unknown_path",
            "Selection contains paths that were not in the latest scan.",
        ));
    }
    // A parent and a child must never be moved twice or double-counted.
    items.sort_by_key(|i| Path::new(&i.path).components().count());
    let mut accepted: Vec<PreviewItem> = Vec::new();
    for item in items {
        if !accepted
            .iter()
            .any(|parent| Path::new(&item.path).starts_with(&parent.path))
        {
            accepted.push(item);
        }
    }
    let categories: std::collections::BTreeSet<_> =
        accepted.iter().map(|i| i.category_id.as_str()).collect();
    let equivalent_command = format!(
        "tiny clean {} --review-paths --idle-days {}",
        categories
            .iter()
            .map(|id| format!("--category {id}"))
            .collect::<Vec<_>>()
            .join(" "),
        idle_days
    );
    let preview = CleanupPreview {
        id: uuid::Uuid::new_v4().to_string(),
        total_bytes: accepted
            .iter()
            .map(|i| i.bytes)
            .fold(0u64, u64::saturating_add),
        items: accepted,
        equivalent_command,
        expires_at: (chrono::Utc::now() + chrono::Duration::minutes(10)).to_rfc3339(),
    };
    let mut plans = lock(&state.plans)?;
    plans.clear();
    plans.insert(
        preview.id.clone(),
        PendingPlan {
            preview: preview.clone(),
            fingerprints,
            created_at: Instant::now(),
            idle_days,
        },
    );
    Ok(preview)
}

pub fn execute_cleanup(
    state: &AppState,
    preview_id: &str,
    confirmed: bool,
    progress: ProgressCallback<'_>,
) -> Result<CleanupResult> {
    execute_cleanup_with_trash(state, preview_id, confirmed, progress, &NativeTrash)
}
fn execute_cleanup_with_trash(
    state: &AppState,
    preview_id: &str,
    confirmed: bool,
    progress: ProgressCallback<'_>,
    trash: &dyn Trash,
) -> Result<CleanupResult> {
    if !confirmed {
        return Err(ErrorPayload::new(
            "confirmation_required",
            "Confirm the preview before cleaning.",
        ));
    }
    let _guard = state.begin()?;
    let plan = lock(&state.plans)?.remove(preview_id).ok_or_else(|| {
        ErrorPayload::new(
            "stale_preview",
            "This preview is missing or has already been used.",
        )
    })?;
    if plan.created_at.elapsed() > Duration::from_secs(600) {
        return Err(ErrorPayload::new(
            "stale_preview",
            "This preview expired. Review a new plan.",
        ));
    }
    let categories: HashSet<_> = plan
        .preview
        .items
        .iter()
        .map(|i| i.category_id.clone())
        .collect();
    let current = discover(&CleanOptions {
        category: categories.into_iter().collect(),
        idle_days: plan.idle_days,
        ..Default::default()
    })?;
    let available: HashSet<_> = current
        .groups
        .iter()
        .flat_map(|g| {
            g.items
                .iter()
                .map(|i| (g.id.clone(), i.path.display().to_string()))
        })
        .collect();
    let operation_id = uuid::Uuid::new_v4().to_string();
    let mut result = CleanupResult {
        operation_id: operation_id.clone(),
        moved_count: 0,
        moved_bytes: 0,
        failed: vec![],
    };
    for (index, item) in plan.preview.items.iter().enumerate() {
        report(
            progress,
            "clean",
            index as u64,
            Some(plan.preview.items.len() as u64),
            &item.path,
        );
        if !available.contains(&(item.category_id.clone(), item.path.clone())) {
            result.failed.push((
                item.path.clone(),
                "No longer eligible, or the owning app is running. Run Smart Scan again.".into(),
            ));
            continue;
        }
        let expected = plan
            .fingerprints
            .get(&item.path)
            .ok_or_else(|| ErrorPayload::new("state_error", "Missing preview fingerprint."))?;
        let id = uuid::Uuid::new_v4().to_string();
        let stored = delete::quarantine_destination(&state.quarantine_root, &id);
        let entry = QuarantineEntry {
            id: id.clone(),
            operation_id: operation_id.clone(),
            original_path: item.path.clone(),
            stored_path: Some(stored.display().to_string()),
            category_id: item.category_id.clone(),
            bytes: item.bytes,
            created_at: chrono::Utc::now().to_rfc3339(),
            expires_at: (chrono::Utc::now() + chrono::Duration::days(30)).to_rfc3339(),
            method: "pending".into(),
            status: "pending".into(),
            error: None,
        };
        // Journal before moving. Even a crash between move and update leaves a recoverable record.
        lock(&state.db)?.record_entry(&entry)?;
        match delete::move_recoverable(Path::new(&item.path), &stored, expected, trash) {
            Ok(method) => {
                lock(&state.db)?.update_entry(&id, method, "moved", None)?;
                result.moved_count += 1;
                result.moved_bytes = result.moved_bytes.saturating_add(item.bytes);
            }
            Err(error) => {
                lock(&state.db)?.update_entry(&id, "failed", "failed", Some(&error.message))?;
                result.failed.push((item.path.clone(), error.message));
            }
        }
    }
    report(
        progress,
        "clean",
        plan.preview.items.len() as u64,
        Some(plan.preview.items.len() as u64),
        "Cleanup complete",
    );
    *lock(&state.latest)? = None;
    Ok(result)
}

pub fn restore_entry(state: &AppState, id: &str) -> Result<()> {
    let _guard = state.begin()?;
    let entry = lock(&state.db)?
        .entries()?
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| ErrorPayload::new("unknown_entry", "No undo record found."))?;
    if entry.status != "moved" && entry.status != "pending" {
        return Err(ErrorPayload::new(
            "invalid_restore",
            "This entry has already been restored or could not be moved.",
        ));
    }
    if entry.method == "trash" {
        return Err(ErrorPayload::new(
            "finder_put_back",
            "Open Trash in Finder, select the item, and choose Put Back.",
        ));
    }
    let stored = entry
        .stored_path
        .as_deref()
        .ok_or_else(|| ErrorPayload::new("invalid_restore", "No quarantine path recorded."))?;
    delete::restore(
        Path::new(stored),
        Path::new(&entry.original_path),
        &state.quarantine_root,
    )?;
    lock(&state.db)?.update_entry(id, "quarantine", "restored", None)?;
    Ok(())
}
pub fn monitor(state: &AppState) -> Result<MonitorReport> {
    let system = tiny_core::sys::information();
    let history = lock(&state.db)?.history()?;
    let now = chrono::Utc::now();
    let oldest = history.iter().rev().find(|s| {
        chrono::DateTime::parse_from_rfc3339(&s.created_at)
            .map(|t| now.signed_duration_since(t).num_days() <= 7)
            .unwrap_or(false)
    });
    let current_disk = primary_disk(&system);
    let current_used = current_disk.map(|d| d.total_bytes.saturating_sub(d.available_bytes));
    let delta = oldest.and_then(|s| {
        current_used.map(|used| {
            (used as i128 - s.used_bytes as i128).clamp(i64::MIN as i128, i64::MAX as i128) as i64
        })
    });
    let days_until_full = oldest.and_then(|s| {
        let elapsed = now
            .signed_duration_since(chrono::DateTime::parse_from_rfc3339(&s.created_at).ok()?)
            .num_seconds() as f64
            / 86400.0;
        let growth = delta? as f64;
        if elapsed < 1.0 || growth <= 0.0 {
            return None;
        }
        Some(current_disk?.available_bytes as f64 / (growth / elapsed))
    });
    Ok(MonitorReport {
        system,
        history,
        weekly_delta_bytes: delta,
        days_until_full,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn seed_scan(state: &AppState, paths: &[std::path::PathBuf]) {
        use tiny_core::{
            clean::{
                discover::{CategoryGroup, DiscoveryReport},
                types::{CleanItem, RiskLevel},
            },
            sys::SystemInfo,
        };
        let system = SystemInfo {
            os: None,
            host: None,
            uptime_seconds: None,
            cpu_count: 1,
            cpu_model: String::new(),
            cpu_usage: 0.0,
            memory_total: 1,
            memory_used: 0,
            disks: vec![],
        };
        let items = paths
            .iter()
            .map(|path| CleanItem {
                category_id: "user-logs".into(),
                category_label: "Logs".into(),
                path: path.clone(),
                size: dir_size_safe(path),
                risk: RiskLevel::Safe,
            })
            .collect();
        let discovery = DiscoveryReport {
            groups: vec![CategoryGroup {
                id: "user-logs".into(),
                label: "Logs".into(),
                risk: RiskLevel::Safe,
                items,
                total_size: 3,
            }],
            skipped_running: vec![],
        };
        let health = tiny_core::health::score(&system, &discovery);
        *lock(&state.latest).unwrap() = Some(SmartScan {
            id: "scan".into(),
            created_at: chrono::Utc::now().to_rfc3339(),
            system,
            discovery,
            health,
        });
    }
    #[test]
    fn preview_rejects_unscanned_paths_and_collapses_parent_child_overlap() {
        let dir = crate::test_directory();
        let state = AppState::new(&dir.path().join("app-data")).unwrap();
        let cache = dir.path().join("cache");
        std::fs::create_dir(&cache).unwrap();
        let file = cache.join("file");
        std::fs::write(&file, b"abc").unwrap();
        seed_scan(&state, &[cache.clone(), file.clone()]);
        assert_eq!(
            preview_cleanup(&state, "scan", &["/unscanned".into()], 30)
                .unwrap_err()
                .code,
            "unknown_path"
        );
        let preview = preview_cleanup(
            &state,
            "scan",
            &[cache.display().to_string(), file.display().to_string()],
            30,
        )
        .unwrap();
        assert_eq!(preview.items.len(), 1);
        assert_eq!(preview.total_bytes, 3);
    }
    #[test]
    fn expired_and_consumed_previews_cannot_be_replayed() {
        let dir = crate::test_directory();
        let state = AppState::new(&dir.path().join("app-data")).unwrap();
        let file = dir.path().join("file");
        std::fs::write(&file, b"abc").unwrap();
        seed_scan(&state, std::slice::from_ref(&file));
        let preview = preview_cleanup(&state, "scan", &[file.display().to_string()], 30).unwrap();
        lock(&state.plans)
            .unwrap()
            .get_mut(&preview.id)
            .unwrap()
            .created_at = Instant::now() - Duration::from_secs(601);
        assert_eq!(
            execute_cleanup(&state, &preview.id, true, None)
                .unwrap_err()
                .code,
            "stale_preview"
        );
        assert_eq!(
            execute_cleanup(&state, &preview.id, true, None)
                .unwrap_err()
                .code,
            "stale_preview"
        );
        assert!(file.exists());
        assert!(lock(&state.db).unwrap().entries().unwrap().is_empty());
    }
    #[test]
    fn forged_preview_and_missing_confirmation_never_reach_delete() {
        let dir = crate::test_directory();
        let state = AppState::new(dir.path()).unwrap();
        assert_eq!(
            execute_cleanup(&state, "forged", false, None)
                .unwrap_err()
                .code,
            "confirmation_required"
        );
        assert_eq!(
            execute_cleanup(&state, "forged", true, None)
                .unwrap_err()
                .code,
            "stale_preview"
        );
        assert!(lock(&state.db).unwrap().entries().unwrap().is_empty());
    }
    #[test]
    fn operation_guard_releases_on_error() {
        let dir = crate::test_directory();
        let state = AppState::new(dir.path()).unwrap();
        let guard = state.begin().unwrap();
        assert!(state.begin().is_err());
        drop(guard);
        assert!(state.begin().is_ok());
    }
}
