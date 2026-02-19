/// Test utilities for coordinating test execution and creating test fixtures
pub mod env_lock {
    use std::sync::Mutex;

    /// Global lock for environment variable tests
    /// This ensures that tests modifying XDG_* env vars don't interfere with each other
    pub static ENV_LOCK: Mutex<()> = Mutex::new(());
}

/// Mock fixtures for engagement operations (like/retweet/bookmarks)
pub mod engagement_fixtures {
    use crate::tweets::models::Tweet;

    /// Create a fixture tweet for testing
    pub fn mock_tweet(id: &str) -> Tweet {
        let mut tweet = Tweet::new(id.to_string());
        tweet.text = Some(format!("Test tweet content for {}", id));
        tweet.author_id = Some("user_test123".to_string());
        tweet.created_at = Some("2024-01-01T00:00:00Z".to_string());
        tweet
    }

    /// Create a fixture list of tweets for bookmark list testing
    pub fn mock_bookmark_tweets(count: usize) -> Vec<Tweet> {
        (0..count)
            .map(|i| mock_tweet(&format!("bookmark_tweet_{}", i)))
            .collect()
    }

    /// Create mock engagement result data for like operation
    pub fn mock_like_result(tweet_id: &str) -> serde_json::Value {
        serde_json::json!({
            "tweet_id": tweet_id,
            "success": true
        })
    }

    /// Create mock engagement result data for retweet operation
    pub fn mock_retweet_result(tweet_id: &str) -> serde_json::Value {
        serde_json::json!({
            "tweet_id": tweet_id,
            "success": true
        })
    }

    /// Create mock bookmark list result with pagination
    pub fn mock_bookmark_list_result(limit: usize, offset: usize) -> serde_json::Value {
        let tweets: Vec<serde_json::Value> = (offset..(offset + limit))
            .map(|i| {
                serde_json::json!({
                    "id": format!("bookmark_tweet_{}", i),
                    "text": format!("Bookmarked tweet text {}", i),
                    "author_id": format!("user_{}", i),
                    "created_at": "2024-01-01T00:00:00Z"
                })
            })
            .collect();

        serde_json::json!({
            "tweets": tweets,
            "meta": {
                "pagination": {
                    "next_token": format!("bookmark_cursor_{}", offset + limit)
                }
            }
        })
    }
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
