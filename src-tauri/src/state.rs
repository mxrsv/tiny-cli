use crate::{
    db::Database,
    delete::Fingerprint,
    error::{ErrorPayload, Result},
    models::{CleanupPreview, SmartScan},
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, MutexGuard,
    },
};

pub struct PendingPlan {
    pub preview: CleanupPreview,
    pub fingerprints: HashMap<String, Fingerprint>,
    pub created_at: std::time::Instant,
    pub idle_days: u64,
}
pub struct AppState {
    pub db: Mutex<Database>,
    pub latest: Mutex<Option<SmartScan>>,
    pub plans: Mutex<HashMap<String, PendingPlan>>,
    pub quarantine_root: PathBuf,
    busy: AtomicBool,
}
impl AppState {
    pub fn new(directory: &Path) -> Result<Self> {
        std::fs::create_dir_all(directory)?;
        let db = Database::open(&directory.join("tiny.sqlite3"))?;
        // Recover a journal interrupted after rename but before its final status update.
        for entry in db
            .entries()?
            .into_iter()
            .filter(|entry| entry.status == "pending")
        {
            if let Some(stored) = entry
                .stored_path
                .as_ref()
                .filter(|path| std::fs::symlink_metadata(path).is_ok())
            {
                tiny_core::space_lens::validate_path(Path::new(stored))?;
                db.update_entry(&entry.id, "quarantine", "moved", None)?;
            } else if std::fs::symlink_metadata(&entry.original_path).is_ok() {
                db.update_entry(
                    &entry.id,
                    "failed",
                    "failed",
                    Some("Interrupted before the move; original still exists."),
                )?;
            } else {
                db.update_entry(
                    &entry.id,
                    "trash",
                    "moved",
                    Some("Interrupted move; check Finder Trash to confirm and Put Back."),
                )?;
            }
        }
        let latest = db.latest_scan()?;
        Ok(Self {
            db: Mutex::new(db),
            latest: Mutex::new(latest),
            plans: Mutex::new(HashMap::new()),
            quarantine_root: directory.join("quarantine"),
            busy: AtomicBool::new(false),
        })
    }
    pub fn begin(&self) -> Result<OperationGuard<'_>> {
        if self
            .busy
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(ErrorPayload::new(
                "operation_busy",
                "Another scan or cleanup is running.",
            ));
        }
        Ok(OperationGuard(self))
    }
}
pub struct OperationGuard<'a>(&'a AppState);
impl Drop for OperationGuard<'_> {
    fn drop(&mut self) {
        self.0.busy.store(false, Ordering::Release);
    }
}
pub fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>> {
    mutex.lock().map_err(|_| {
        ErrorPayload::new(
            "state_error",
            "App state lock is unavailable. Restart Tiny.",
        )
    })
}
