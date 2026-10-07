pub const SCHEMA: &str = "
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;
CREATE TABLE IF NOT EXISTS scan_snapshots (
    id TEXT PRIMARY KEY, created_at INTEGER NOT NULL, used_bytes INTEGER NOT NULL,
    reclaimable_bytes INTEGER NOT NULL, health_score INTEGER NOT NULL, payload TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS scan_snapshots_created_at ON scan_snapshots(created_at);
CREATE TABLE IF NOT EXISTS quarantine_entries (
    id TEXT PRIMARY KEY, operation_id TEXT NOT NULL, original_path TEXT NOT NULL,
    stored_path TEXT, category_id TEXT NOT NULL, bytes INTEGER NOT NULL,
    created_at INTEGER NOT NULL, expires_at INTEGER NOT NULL,
    method TEXT NOT NULL, status TEXT NOT NULL, error TEXT
);
CREATE TABLE IF NOT EXISTS rules (
    id TEXT PRIMARY KEY, category_id TEXT NOT NULL, min_bytes INTEGER NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1
);
";
