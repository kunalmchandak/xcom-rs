/// Test utilities for coordinating test execution and creating test fixtures
pub mod env_lock {
    use std::sync::Mutex;

    /// Global lock for environment variable tests
    /// This ensures that tests modifying XDG_* env vars don't interfere with each other
    pub static ENV_LOCK: Mutex<()> = Mutex::new(());
}

pub mod helpers {
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    /// Creates a temporary directory for testing and returns it.
    /// The directory will be automatically cleaned up when the TempDir is dropped.
    pub fn create_test_dir(prefix: &str) -> TempDir {
        TempDir::new().unwrap_or_else(|e| {
            panic!(
                "Failed to create test directory with prefix '{}': {}",
                prefix, e
            )
        })
    }

    /// Creates a test database path in a temporary directory
    pub fn create_test_db_path(temp_dir: &Path) -> PathBuf {
        temp_dir.join("test.db")
    }

    /// Creates a test HOME directory structure in temp and returns the path
    pub fn create_test_home() -> TempDir {
        let test_dir = std::env::temp_dir().join(format!("xcom-rs-test-{}", std::process::id()));
        std::fs::create_dir_all(&test_dir)
            .unwrap_or_else(|e| panic!("Failed to create test HOME directory: {}", e));

        TempDir::new().unwrap_or_else(|e| panic!("Failed to create test HOME TempDir: {}", e))
    }

    /// Creates a test IdempotencyLedger with an in-memory database
    pub fn create_test_ledger() -> crate::tweets::IdempotencyLedger {
        crate::tweets::IdempotencyLedger::new(None)
            .expect("Failed to create test IdempotencyLedger")
    }

    /// Creates a test IdempotencyLedger with a file-based database
    pub fn create_test_ledger_with_db(db_path: &Path) -> crate::tweets::IdempotencyLedger {
        crate::tweets::IdempotencyLedger::new(Some(db_path))
            .expect("Failed to create test IdempotencyLedger with database")
    }

    /// Helper to parse JSON from command output
    pub fn parse_json_output(output: &[u8]) -> serde_json::Value {
        let stdout = String::from_utf8_lossy(output);
        serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("Failed to parse JSON output: {}\nOutput: {}", e, stdout))
    }

    /// Helper to assert command succeeded and return parsed JSON
    pub fn assert_success_json(output: &std::process::Output) -> serde_json::Value {
        assert!(
            output.status.success(),
            "Command failed with status: {:?}\nStdout: {}\nStderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        parse_json_output(&output.stdout)
    }

    /// Helper to assert command failed with expected exit code and return parsed JSON
    pub fn assert_error_json(
        output: &std::process::Output,
        expected_code: i32,
    ) -> serde_json::Value {
        assert_eq!(
            output.status.code(),
            Some(expected_code),
            "Expected exit code {} but got {:?}\nStdout: {}\nStderr: {}",
            expected_code,
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        parse_json_output(&output.stdout)
    }
}
