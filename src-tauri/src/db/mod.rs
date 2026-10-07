mod schema;
use rusqlite::{params, Connection};
use std::path::Path;
fn sql_bytes(value: u64) -> crate::error::Result<i64> {
    i64::try_from(value).map_err(|_| {
        crate::error::ErrorPayload::new("invalid_bytes", "Byte value exceeds SQLite INTEGER range.")
    })
}

use crate::{
    error::{ErrorPayload, Result},
    models::*,
};

pub struct Database {
    connection: Connection,
}
impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let connection = Connection::open(path)?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        connection.execute_batch(schema::SCHEMA)?;
        Ok(Self { connection })
    }
    pub fn save_scan(&self, scan: &SmartScan) -> Result<()> {
        let disk = primary_disk(&scan.system);
        let used = disk
            .map(|d| d.total_bytes.saturating_sub(d.available_bytes))
            .unwrap_or(0);
        self.connection.execute(
            "INSERT INTO scan_snapshots VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                scan.id,
                chrono::DateTime::parse_from_rfc3339(&scan.created_at)
                    .map_err(|e| ErrorPayload::new("invalid_date", e))?
                    .timestamp(),
                sql_bytes(used)?,
                sql_bytes(scan.health.reclaimable_bytes)?,
                scan.health.score,
                serde_json::to_string(scan)?
            ],
        )?;
        // Bound local history size while preserving the latest 120 complete snapshots.
        self.connection.execute("DELETE FROM scan_snapshots WHERE id NOT IN (SELECT id FROM scan_snapshots ORDER BY created_at DESC, rowid DESC LIMIT 120)", [])?;
        Ok(())
    }
    pub fn latest_scan(&self) -> Result<Option<SmartScan>> {
        use rusqlite::OptionalExtension;
        let raw: Option<String> = self
            .connection
            .query_row(
                "SELECT payload FROM scan_snapshots ORDER BY created_at DESC, rowid DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()?;
        raw.map(|s| serde_json::from_str(&s).map_err(Into::into))
            .transpose()
    }
    pub fn history(&self) -> Result<Vec<ScanSummary>> {
        let mut stmt = self.connection.prepare("SELECT id, created_at, used_bytes, reclaimable_bytes, health_score FROM scan_snapshots ORDER BY created_at DESC, rowid DESC LIMIT 120")?;
        let rows = stmt.query_map([], |r| {
            Ok(ScanSummary {
                id: r.get(0)?,
                created_at: iso_time(r.get(1)?),
                used_bytes: r.get::<_, i64>(2)? as u64,
                reclaimable_bytes: r.get::<_, i64>(3)? as u64,
                health_score: r.get(4)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }
    pub fn record_entry(&self, entry: &QuarantineEntry) -> Result<()> {
        self.connection.execute(
            "INSERT INTO quarantine_entries VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                entry.id,
                entry.operation_id,
                entry.original_path,
                entry.stored_path,
                entry.category_id,
                sql_bytes(entry.bytes)?,
                chrono::DateTime::parse_from_rfc3339(&entry.created_at)
                    .map_err(|e| ErrorPayload::new("invalid_date", e))?
                    .timestamp(),
                chrono::DateTime::parse_from_rfc3339(&entry.expires_at)
                    .map_err(|e| ErrorPayload::new("invalid_date", e))?
                    .timestamp(),
                entry.method,
                entry.status,
                entry.error
            ],
        )?;
        Ok(())
    }
    pub fn update_entry(
        &self,
        id: &str,
        method: &str,
        status: &str,
        error: Option<&str>,
    ) -> Result<()> {
        self.connection.execute(
            "UPDATE quarantine_entries SET method=?2, status=?3, error=?4 WHERE id=?1",
            params![id, method, status, error],
        )?;
        Ok(())
    }
    pub fn entries(&self) -> Result<Vec<QuarantineEntry>> {
        let mut stmt = self.connection.prepare("SELECT id, operation_id, original_path, stored_path, category_id, bytes, created_at, expires_at, method, status, error FROM quarantine_entries ORDER BY created_at DESC, rowid DESC LIMIT 500")?;
        let rows = stmt.query_map([], |r| {
            Ok(QuarantineEntry {
                id: r.get(0)?,
                operation_id: r.get(1)?,
                original_path: r.get(2)?,
                stored_path: r.get(3)?,
                category_id: r.get(4)?,
                bytes: r.get::<_, i64>(5)? as u64,
                created_at: iso_time(r.get(6)?),
                expires_at: iso_time(r.get(7)?),
                method: r.get(8)?,
                status: r.get(9)?,
                error: r.get(10)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }
    pub fn rules(&self) -> Result<Vec<Rule>> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, category_id, min_bytes, enabled FROM rules ORDER BY id")?;
        let rows = stmt.query_map([], |r| {
            Ok(Rule {
                id: r.get(0)?,
                category_id: r.get(1)?,
                min_bytes: r.get::<_, i64>(2)? as u64,
                enabled: r.get(3)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }
    pub fn save_rule(&self, rule: &Rule) -> Result<()> {
        if !tiny_core::clean::providers::known_category_ids().contains(&rule.category_id.as_str()) {
            return Err(ErrorPayload::new(
                "invalid_rule",
                "Unknown cleanup category.",
            ));
        }
        self.connection.execute("INSERT INTO rules VALUES (?1, ?2, ?3, ?4) ON CONFLICT(id) DO UPDATE SET category_id=excluded.category_id, min_bytes=excluded.min_bytes, enabled=excluded.enabled", params![rule.id, rule.category_id, sql_bytes(rule.min_bytes)?, rule.enabled])?;
        Ok(())
    }
    pub fn delete_rule(&self, id: &str) -> Result<()> {
        self.connection
            .execute("DELETE FROM rules WHERE id=?1", [id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn journal_and_rules_survive_reopening_the_database() {
        let directory = crate::test_directory();
        let path = directory.path().join("tiny.sqlite3");
        let db = Database::open(&path).unwrap();
        let entry = QuarantineEntry {
            id: "entry".into(),
            operation_id: "operation".into(),
            original_path: "/test/cache".into(),
            stored_path: Some("/test/quarantine/entry".into()),
            category_id: "user-logs".into(),
            bytes: 1234,
            created_at: "2026-10-06T00:00:00Z".into(),
            expires_at: "2026-11-05T00:00:00Z".into(),
            method: "pending".into(),
            status: "pending".into(),
            error: None,
        };
        db.record_entry(&entry).unwrap();
        db.update_entry("entry", "quarantine", "moved", None)
            .unwrap();
        db.save_rule(&Rule {
            id: "rule".into(),
            category_id: "user-logs".into(),
            min_bytes: 1024,
            enabled: true,
        })
        .unwrap();
        assert!(db
            .save_rule(&Rule {
                id: "invalid".into(),
                category_id: "unknown".into(),
                min_bytes: 0,
                enabled: true
            })
            .is_err());
        drop(db);
        let db = Database::open(&path).unwrap();
        assert_eq!(db.entries().unwrap()[0].status, "moved");
        assert_eq!(db.rules().unwrap()[0].min_bytes, 1024);
        db.delete_rule("rule").unwrap();
        assert!(db.rules().unwrap().is_empty());
        assert!(sql_bytes(u64::MAX).is_err());
    }
}
