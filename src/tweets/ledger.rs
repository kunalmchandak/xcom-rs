use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Idempotency ledger for tracking tweet operations
pub struct IdempotencyLedger {
    conn: Connection,
}

/// Entry in the idempotency ledger
#[derive(Debug, Clone)]
pub struct LedgerEntry {
    pub client_request_id: String,
    pub request_hash: String,
    pub tweet_id: String,
    pub status: String,
    pub created_at: i64,
}

impl IdempotencyLedger {
    /// Create or open the idempotency ledger database
    pub fn new(db_path: Option<&Path>) -> Result<Self> {
        let path = db_path
            .map(PathBuf::from)
            .unwrap_or_else(Self::default_db_path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create ledger directory")?;
        }

        let conn = Connection::open(&path).context("Failed to open idempotency ledger database")?;

        let ledger = Self { conn };
        ledger.init_schema()?;
        Ok(ledger)
    }

    /// Get default database path: ~/.xcom-rs/idempotency.db
    fn default_db_path() -> PathBuf {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".xcom-rs");
        path.push("idempotency.db");
        path
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tweet_operations (
                client_request_id TEXT NOT NULL PRIMARY KEY,
                request_hash TEXT NOT NULL,
                tweet_id TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    /// Compute hash of request parameters for duplicate detection
    pub fn compute_request_hash(params: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(params.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Record a successful tweet operation
    pub fn record(
        &self,
        client_request_id: &str,
        request_hash: &str,
        tweet_id: &str,
        status: &str,
    ) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        self.conn.execute(
            "INSERT OR REPLACE INTO tweet_operations 
             (client_request_id, request_hash, tweet_id, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![client_request_id, request_hash, tweet_id, status, now],
        )?;

        Ok(())
    }

    /// Look up an existing operation by client_request_id
    /// Returns the entry if client_request_id exists, regardless of request_hash
    /// The caller is responsible for handling hash mismatches if needed
    pub fn lookup(&self, client_request_id: &str) -> Result<Option<LedgerEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT client_request_id, request_hash, tweet_id, status, created_at
             FROM tweet_operations
             WHERE client_request_id = ?1",
        )?;

        let entry = stmt
            .query_row(params![client_request_id], |row| {
                Ok(LedgerEntry {
                    client_request_id: row.get(0)?,
                    request_hash: row.get(1)?,
                    tweet_id: row.get(2)?,
                    status: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .optional()?;

        Ok(entry)
    }

    /// Clean up old entries (garbage collection)
    /// Removes entries older than the specified number of days
    pub fn cleanup_old_entries(&self, days: i64) -> Result<usize> {
        let cutoff = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64
            - (days * 24 * 60 * 60);

        let deleted = self.conn.execute(
            "DELETE FROM tweet_operations WHERE created_at < ?1",
            params![cutoff],
        )?;

        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_ledger() -> (IdempotencyLedger, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let ledger = IdempotencyLedger::new(Some(&db_path)).unwrap();
        (ledger, temp_dir)
    }

    #[test]
    fn test_record_and_lookup() {
        let (ledger, _temp) = create_test_ledger();

        let client_request_id = "test-123";
        let request_hash = IdempotencyLedger::compute_request_hash("text=hello");
        let tweet_id = "tweet-456";

        // Record operation
        ledger
            .record(client_request_id, &request_hash, tweet_id, "success")
            .unwrap();

        // Lookup should find it
        let entry = ledger.lookup(client_request_id).unwrap().unwrap();
        assert_eq!(entry.client_request_id, client_request_id);
        assert_eq!(entry.tweet_id, tweet_id);
        assert_eq!(entry.status, "success");
    }

    #[test]
    fn test_lookup_nonexistent() {
        let (ledger, _temp) = create_test_ledger();

        let result = ledger.lookup("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_request_hash_consistency() {
        let hash1 = IdempotencyLedger::compute_request_hash("text=hello");
        let hash2 = IdempotencyLedger::compute_request_hash("text=hello");
        assert_eq!(hash1, hash2);

        let hash3 = IdempotencyLedger::compute_request_hash("text=world");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_cleanup_old_entries() {
        let (ledger, _temp) = create_test_ledger();

        // Record an entry
        ledger
            .record("test-1", "hash-1", "tweet-1", "success")
            .unwrap();

        // Cleanup entries older than 0 days should remove nothing (entry is recent)
        let deleted = ledger.cleanup_old_entries(0).unwrap();
        assert_eq!(deleted, 0);

        // Entry should still exist
        let entry = ledger.lookup("test-1").unwrap();
        assert!(entry.is_some());
    }
}
