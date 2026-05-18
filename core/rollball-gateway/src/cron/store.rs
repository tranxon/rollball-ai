//! Cron entry persistence store (rusqlite backend)
//!
//! Stores cron entries to survive Gateway restarts.
//! Uses the same rusqlite approach as PermissionStore.
//!
//! Schema versioning:
//! - v1: Initial schema (cron_entries table)

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection};

/// Current schema version
const SCHEMA_VERSION: u32 = 2;

/// Persistent cron entry stored in the database (S5.8 enhanced)
#[derive(Debug, Clone)]
pub struct StoredCronEntry {
    /// Unique ID for this cron entry
    pub id: String,
    /// Agent that owns this cron entry
    pub agent_id: String,
    /// Cron schedule expression (5-field)
    pub schedule: String,
    /// Action to fire when the schedule triggers
    pub action: String,
    /// Params to include in the IntentReceived (JSON)
    pub params: String,
    /// Timezone for schedule interpretation (None = UTC)
    pub timezone: Option<String>,
    /// Max retry count on failure (0 = no retry)
    pub retry_count: u32,
    /// Retry backoff interval in seconds
    pub retry_interval_secs: u64,
    /// Max total executions (None = unlimited)
    pub max_runs: Option<u32>,
    /// Current execution count
    pub run_count: u32,
    /// Expiry timestamp in Unix millis (None = never expires)
    pub expires_at: Option<i64>,
}

impl StoredCronEntry {
    /// Create a simple StoredCronEntry with defaults for optional S5.8 fields.
    /// timezone=None, retry_count=0, retry_interval_secs=60,
    /// max_runs=None, run_count=0, expires_at=None.
    pub fn simple(id: &str, agent_id: &str, schedule: &str, action: &str, params: &str) -> Self {
        Self {
            id: id.to_string(),
            agent_id: agent_id.to_string(),
            schedule: schedule.to_string(),
            action: action.to_string(),
            params: params.to_string(),
            timezone: None,
            retry_count: 0,
            retry_interval_secs: 60,
            max_runs: None,
            run_count: 0,
            expires_at: None,
        }
    }
}

/// Error type for CronStore operations
#[derive(Debug, thiserror::Error)]
pub enum CronStoreError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid cron entry: {0}")]
    InvalidEntry(String),
}

/// Persistent store for cron entries.
///
/// Schema (v1):
/// ```sql
/// CREATE TABLE cron_entries (
///     id          TEXT PRIMARY KEY,
///     agent_id    TEXT NOT NULL,
///     schedule    TEXT NOT NULL,
///     action      TEXT NOT NULL,
///     params      TEXT NOT NULL DEFAULT '{}',
///     created_at  INTEGER NOT NULL
/// );
/// CREATE INDEX idx_cron_agent ON cron_entries(agent_id);
/// ```
#[derive(Debug)]
pub struct CronStore {
    conn: Mutex<Connection>,
}

impl CronStore {
    /// Open (or create) the cron store at the given path.
    pub fn open(path: &Path) -> Result<Self, CronStoreError> {
        let conn = Connection::open(path)?;
        let store = Self { conn: Mutex::new(conn) };
        store.init_schema()?;
        Ok(store)
    }

    /// Open an in-memory store (for testing).
    pub fn open_in_memory() -> Result<Self, CronStoreError> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn: Mutex::new(conn) };
        store.init_schema()?;
        Ok(store)
    }

    /// Check if the database connection is alive.
    /// Performs a lightweight `SELECT 1` query.
    pub fn health_check(&self) -> Result<(), CronStoreError> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT 1", [], |_row| Ok(()))?;
        Ok(())
    }

    fn init_schema(&self) -> Result<(), CronStoreError> {
        let conn = self.conn.lock().unwrap();

        // Create schema_version table if not exists
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER NOT NULL
            );"
        )?;

        // Check current version
        let current_version: Option<u32> = conn
            .query_row(
                "SELECT version FROM schema_version LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok();

        match current_version {
            None => {
                // Fresh database — create v2 schema with S5.8 enhanced columns
                conn.execute_batch(
                    "CREATE TABLE cron_entries (
                        id          TEXT PRIMARY KEY,
                        agent_id    TEXT NOT NULL,
                        schedule    TEXT NOT NULL,
                        action      TEXT NOT NULL,
                        params      TEXT NOT NULL DEFAULT '{}',
                        created_at  INTEGER NOT NULL,
                        timezone    TEXT,
                        retry_count INTEGER NOT NULL DEFAULT 0,
                        retry_interval_secs INTEGER NOT NULL DEFAULT 60,
                        max_runs    INTEGER,
                        run_count   INTEGER NOT NULL DEFAULT 0,
                        expires_at  INTEGER
                    );
                    CREATE INDEX idx_cron_agent ON cron_entries(agent_id);

                    INSERT INTO schema_version (version) VALUES (2);"
                )?;
            }
            Some(v) if v < SCHEMA_VERSION => {
                // Run migrations from v to SCHEMA_VERSION
                self.run_migrations(&conn, v)?;
            }
            Some(_) => {
                // Already at latest version
            }
        }

        Ok(())
    }

    fn run_migrations(
        &self,
        conn: &Connection,
        from: u32,
    ) -> Result<(), CronStoreError> {
        // Migration v1→v2: Add S5.8 enhanced columns
        if from < 2 {
            conn.execute_batch(
                "ALTER TABLE cron_entries ADD COLUMN timezone TEXT;
                 ALTER TABLE cron_entries ADD COLUMN retry_count INTEGER NOT NULL DEFAULT 0;
                 ALTER TABLE cron_entries ADD COLUMN retry_interval_secs INTEGER NOT NULL DEFAULT 60;
                 ALTER TABLE cron_entries ADD COLUMN max_runs INTEGER;
                 ALTER TABLE cron_entries ADD COLUMN run_count INTEGER NOT NULL DEFAULT 0;
                 ALTER TABLE cron_entries ADD COLUMN expires_at INTEGER;
                 UPDATE schema_version SET version = 2;"
            )?;
        }
        Ok(())
    }

    /// Insert a new cron entry
    pub fn insert(&self, entry: &StoredCronEntry) -> Result<(), CronStoreError> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "INSERT INTO cron_entries (id, agent_id, schedule, action, params,
             timezone, retry_count, retry_interval_secs, max_runs, run_count, expires_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                entry.id, entry.agent_id, entry.schedule, entry.action, entry.params,
                entry.timezone, entry.retry_count, entry.retry_interval_secs,
                entry.max_runs, entry.run_count, entry.expires_at, now,
            ],
        )?;
        Ok(())
    }

    /// Delete a cron entry by ID
    pub fn delete(&self, cron_id: &str) -> Result<bool, CronStoreError> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "DELETE FROM cron_entries WHERE id = ?1",
            params![cron_id],
        )?;
        Ok(rows > 0)
    }

    /// Delete all cron entries for an agent
    pub fn delete_by_agent(&self, agent_id: &str) -> Result<usize, CronStoreError> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "DELETE FROM cron_entries WHERE agent_id = ?1",
            params![agent_id],
        )?;
        Ok(rows)
    }

    /// List all cron entries for an agent
    pub fn list_by_agent(&self, agent_id: &str) -> Result<Vec<StoredCronEntry>, CronStoreError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, agent_id, schedule, action, params,
                    timezone, retry_count, retry_interval_secs, max_runs, run_count, expires_at
             FROM cron_entries WHERE agent_id = ?1"
        )?;
        let entries = stmt.query_map(params![agent_id], |row| {
            Ok(StoredCronEntry {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                schedule: row.get(2)?,
                action: row.get(3)?,
                params: row.get(4)?,
                timezone: row.get(5)?,
                retry_count: row.get(6)?,
                retry_interval_secs: row.get(7)?,
                max_runs: row.get(8)?,
                run_count: row.get(9)?,
                expires_at: row.get(10)?,
            })
        })?.filter_map(|e| e.ok()).collect();
        Ok(entries)
    }

    /// List all cron entries
    pub fn list_all(&self) -> Result<Vec<StoredCronEntry>, CronStoreError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, agent_id, schedule, action, params,
                    timezone, retry_count, retry_interval_secs, max_runs, run_count, expires_at
             FROM cron_entries"
        )?;
        let entries = stmt.query_map([], |row| {
            Ok(StoredCronEntry {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                schedule: row.get(2)?,
                action: row.get(3)?,
                params: row.get(4)?,
                timezone: row.get(5)?,
                retry_count: row.get(6)?,
                retry_interval_secs: row.get(7)?,
                max_runs: row.get(8)?,
                run_count: row.get(9)?,
                expires_at: row.get(10)?,
            })
        })?.filter_map(|e| e.ok()).collect();
        Ok(entries)
    }

    /// Get a single cron entry by ID
    pub fn get(&self, cron_id: &str) -> Result<Option<StoredCronEntry>, CronStoreError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, agent_id, schedule, action, params,
                    timezone, retry_count, retry_interval_secs, max_runs, run_count, expires_at
             FROM cron_entries WHERE id = ?1"
        )?;
        let result = stmt.query_row(params![cron_id], |row| {
            Ok(StoredCronEntry {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                schedule: row.get(2)?,
                action: row.get(3)?,
                params: row.get(4)?,
                timezone: row.get(5)?,
                retry_count: row.get(6)?,
                retry_interval_secs: row.get(7)?,
                max_runs: row.get(8)?,
                run_count: row.get(9)?,
                expires_at: row.get(10)?,
            })
        }).ok();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cron_store_open_in_memory() {
        let store = CronStore::open_in_memory().unwrap();
        let entries = store.list_all().unwrap();
        assert!(entries.is_empty());
    }

    fn make_entry(id: &str, agent_id: &str, schedule: &str, action: &str, params: &str) -> StoredCronEntry {
        StoredCronEntry::simple(id, agent_id, schedule, action, params)
    }

    #[test]
    fn test_cron_store_insert_and_list() {
        let store = CronStore::open_in_memory().unwrap();
        let entry = make_entry("cron-1", "com.example.weather", "0 * * * *", "hourly_check", "{}");
        store.insert(&entry).unwrap();

        let entries = store.list_by_agent("com.example.weather").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "cron-1");
        assert_eq!(entries[0].schedule, "0 * * * *");
    }

    #[test]
    fn test_cron_store_delete() {
        let store = CronStore::open_in_memory().unwrap();
        let entry = make_entry("cron-2", "com.example.weather", "0 9 * * *", "morning_report", "{}");
        store.insert(&entry).unwrap();
        assert!(store.delete("cron-2").unwrap());
        assert!(!store.delete("cron-2").unwrap()); // Already deleted
    }

    #[test]
    fn test_cron_store_delete_by_agent() {
        let store = CronStore::open_in_memory().unwrap();
        for i in 1..=3 {
            let entry = make_entry(
                &format!("cron-{}", i),
                "com.example.weather",
                "0 * * * *",
                &format!("task-{}", i),
                "{}",
            );
            store.insert(&entry).unwrap();
        }
        let entry = make_entry("cron-4", "com.example.calendar", "0 0 * * *", "daily", "{}");
        store.insert(&entry).unwrap();

        let count = store.delete_by_agent("com.example.weather").unwrap();
        assert_eq!(count, 3);

        let all = store.list_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].agent_id, "com.example.calendar");
    }

    #[test]
    fn test_cron_store_get() {
        let store = CronStore::open_in_memory().unwrap();
        let entry = make_entry("cron-10", "com.example.test", "*/15 * * * *", "health_check", r#"{"type":"ping"}"#);
        store.insert(&entry).unwrap();

        let got = store.get("cron-10").unwrap().unwrap();
        assert_eq!(got.schedule, "*/15 * * * *");
        assert_eq!(got.params, r#"{"type":"ping"}"#);

        assert!(store.get("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_cron_store_list_all() {
        let store = CronStore::open_in_memory().unwrap();
        for i in 1..=5 {
            let entry = make_entry(
                &format!("cron-{}", i),
                &format!("com.example.agent{}", i % 2),
                "0 * * * *",
                &format!("task-{}", i),
                "{}",
            );
            store.insert(&entry).unwrap();
        }
        let all = store.list_all().unwrap();
        assert_eq!(all.len(), 5);
    }
}
