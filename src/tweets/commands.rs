use super::{
    ledger::IdempotencyLedger,
    models::{Tweet, TweetFields, TweetMeta},
};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// Custom error type for idempotency conflicts
#[derive(Debug)]
pub struct IdempotencyConflictError {
    pub client_request_id: String,
}

impl std::fmt::Display for IdempotencyConflictError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Operation with client_request_id '{}' already exists",
            self.client_request_id
        )
    }
}

impl std::error::Error for IdempotencyConflictError {}

/// Policy for handling existing operations with the same client_request_id
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IfExistsPolicy {
    /// Return the existing result without error
    Return,
    /// Return an error if operation already exists
    Error,
}

impl FromStr for IfExistsPolicy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "return" => Ok(Self::Return),
            "error" => Ok(Self::Error),
            _ => Err(anyhow!(
                "Invalid if-exists policy: {}. Valid values: return, error",
                s
            )),
        }
    }
}

impl IfExistsPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Return => "return",
            Self::Error => "error",
        }
    }
}

/// Arguments for creating a tweet
#[derive(Debug, Clone)]
pub struct CreateArgs {
    pub text: String,
    pub client_request_id: Option<String>,
    pub if_exists: IfExistsPolicy,
}

/// Arguments for listing tweets
#[derive(Debug, Clone)]
pub struct ListArgs {
    pub fields: Vec<TweetFields>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

/// Arguments for engagement operations (like/unlike/retweet/unretweet)
#[derive(Debug, Clone)]
pub struct EngagementArgs {
    pub tweet_id: String,
}

/// Result of an engagement operation (like/unlike/retweet/unretweet)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementResult {
    pub tweet_id: String,
    pub success: bool,
}

/// Result of a create operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResult {
    pub tweet: Tweet,
    pub meta: TweetMeta,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_cursor: Option<String>,
}

/// Result of a list operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResult {
    pub tweets: Vec<Tweet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ListResultMeta>,
}

/// Metadata for list results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResultMeta {
    pub pagination: PaginationMeta,
}

/// Error classification for retry logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Retryable errors (429, 5xx)
    Retryable,
    /// Non-retryable client errors (4xx except 429)
    NonRetryable,
    /// Network/timeout errors
    Timeout,
}

/// Classified error with retry information
#[derive(Debug)]
pub struct ClassifiedError {
    pub kind: ErrorKind,
    pub status_code: Option<u16>,
    pub message: String,
    pub is_retryable: bool,
    pub retry_after_ms: Option<u64>,
}

impl ClassifiedError {
    pub fn from_status_code(status_code: u16, message: String) -> Self {
        let (kind, is_retryable) = match status_code {
            429 => (ErrorKind::Retryable, true),
            500..=599 => (ErrorKind::Retryable, true),
            400..=499 => (ErrorKind::NonRetryable, false),
            _ => (ErrorKind::NonRetryable, false),
        };

        Self {
            kind,
            status_code: Some(status_code),
            message,
            is_retryable,
            retry_after_ms: None,
        }
    }

    pub fn timeout(message: String) -> Self {
        Self {
            kind: ErrorKind::Timeout,
            status_code: None,
            message,
            is_retryable: true,
            retry_after_ms: None,
        }
    }

    pub fn with_retry_after(mut self, retry_after_ms: u64) -> Self {
        self.retry_after_ms = Some(retry_after_ms);
        self
    }

    /// Convert to ErrorCode for protocol
    pub fn to_error_code(&self) -> crate::protocol::ErrorCode {
        use crate::protocol::ErrorCode;
        match self.kind {
            ErrorKind::Retryable => {
                if let Some(429) = self.status_code {
                    ErrorCode::RateLimitExceeded
                } else {
                    ErrorCode::ServiceUnavailable
                }
            }
            ErrorKind::Timeout => ErrorCode::NetworkError,
            ErrorKind::NonRetryable => ErrorCode::InternalError,
        }
    }
}

impl std::fmt::Display for ClassifiedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ClassifiedError {}

/// Main tweets command handler
pub struct TweetCommand {
    ledger: IdempotencyLedger,
}

impl TweetCommand {
    /// Create a new tweet command handler
    pub fn new(ledger: IdempotencyLedger) -> Self {
        Self { ledger }
    }

    /// Create a tweet with idempotency support
    pub fn create(&self, args: CreateArgs) -> Result<CreateResult> {
        // Check for simulated errors via environment variables (for testing)
        if let Ok(error_type) = std::env::var("XCOM_SIMULATE_ERROR") {
            match error_type.as_str() {
                "rate_limit" => {
                    let retry_after = std::env::var("XCOM_RETRY_AFTER_MS")
                        .ok()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(60000);
                    return Err(ClassifiedError::from_status_code(
                        429,
                        "Rate limit exceeded".to_string(),
                    )
                    .with_retry_after(retry_after)
                    .into());
                }
                "server_error" => {
                    return Err(ClassifiedError::from_status_code(
                        500,
                        "Internal server error".to_string(),
                    )
                    .into());
                }
                "timeout" => {
                    return Err(ClassifiedError::timeout("Request timeout".to_string()).into());
                }
                _ => {
                    // Continue with normal flow for unknown error types
                }
            }
        }

        // Generate client_request_id if not provided
        let client_request_id = args
            .client_request_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Compute request hash for storing (but not for lookup key)
        let request_hash = IdempotencyLedger::compute_request_hash(&args.text);

        // Check ledger for existing operation by client_request_id only
        if let Some(entry) = self
            .ledger
            .lookup(&client_request_id)
            .context("Failed to lookup operation in ledger")?
        {
            // Found existing operation with this client_request_id
            match args.if_exists {
                IfExistsPolicy::Return => {
                    // Return cached result (even if parameters differ)
                    let mut tweet = Tweet::new(entry.tweet_id.clone());
                    tweet.text = Some(args.text.clone());

                    let meta = TweetMeta {
                        client_request_id: client_request_id.clone(),
                        from_cache: Some(true),
                    };

                    return Ok(CreateResult { tweet, meta });
                }
                IfExistsPolicy::Error => {
                    // Return error for duplicate client_request_id
                    return Err(IdempotencyConflictError {
                        client_request_id: client_request_id.clone(),
                    }
                    .into());
                }
            }
        }

        // Simulate tweet creation (in real implementation, would call X API)
        let tweet_id = format!("tweet_{}", Uuid::new_v4());
        let mut tweet = Tweet::new(tweet_id.clone());
        tweet.text = Some(args.text);

        // Record successful operation in ledger
        self.ledger
            .record(&client_request_id, &request_hash, &tweet_id, "success")
            .context("Failed to record operation in ledger")?;

        let meta = TweetMeta {
            client_request_id,
            from_cache: None,
        };

        Ok(CreateResult { tweet, meta })
    }

    /// Like a tweet
    pub fn like(&self, args: EngagementArgs) -> Result<EngagementResult> {
        // Check for simulated errors via environment variables (for testing)
        if let Ok(error_type) = std::env::var("XCOM_SIMULATE_ERROR") {
            match error_type.as_str() {
                "rate_limit" => {
                    let retry_after = std::env::var("XCOM_RETRY_AFTER_MS")
                        .ok()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(60000);
                    return Err(ClassifiedError::from_status_code(
                        429,
                        "Rate limit exceeded".to_string(),
                    )
                    .with_retry_after(retry_after)
                    .into());
                }
                "auth_error" => {
                    return Err(ClassifiedError::from_status_code(
                        403,
                        "Insufficient permissions to like tweet".to_string(),
                    )
                    .into());
                }
                _ => {}
            }
        }

        // Simulate like operation (in real implementation, would call POST /2/users/{id}/likes)
        Ok(EngagementResult {
            tweet_id: args.tweet_id,
            success: true,
        })
    }

    /// Unlike a tweet
    pub fn unlike(&self, args: EngagementArgs) -> Result<EngagementResult> {
        // Check for simulated errors via environment variables (for testing)
        if let Ok(error_type) = std::env::var("XCOM_SIMULATE_ERROR") {
            if error_type.as_str() == "rate_limit" {
                let retry_after = std::env::var("XCOM_RETRY_AFTER_MS")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(60000);
                return Err(ClassifiedError::from_status_code(
                    429,
                    "Rate limit exceeded".to_string(),
                )
                .with_retry_after(retry_after)
                .into());
            }
        }

        // Simulate unlike operation (in real implementation, would call DELETE /2/users/{id}/likes/{tweet_id})
        Ok(EngagementResult {
            tweet_id: args.tweet_id,
            success: true,
        })
    }

    /// Retweet a tweet
    pub fn retweet(&self, args: EngagementArgs) -> Result<EngagementResult> {
        // Check for simulated errors via environment variables (for testing)
        if let Ok(error_type) = std::env::var("XCOM_SIMULATE_ERROR") {
            match error_type.as_str() {
                "rate_limit" => {
                    let retry_after = std::env::var("XCOM_RETRY_AFTER_MS")
                        .ok()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(60000);
                    return Err(ClassifiedError::from_status_code(
                        429,
                        "Rate limit exceeded".to_string(),
                    )
                    .with_retry_after(retry_after)
                    .into());
                }
                "auth_error" => {
                    return Err(ClassifiedError::from_status_code(
                        403,
                        "Insufficient permissions to retweet".to_string(),
                    )
                    .into());
                }
                _ => {}
            }
        }

        // Simulate retweet operation (in real implementation, would call POST /2/users/{id}/retweets)
        Ok(EngagementResult {
            tweet_id: args.tweet_id,
            success: true,
        })
    }

    /// Unretweet a tweet
    pub fn unretweet(&self, args: EngagementArgs) -> Result<EngagementResult> {
        // Check for simulated errors via environment variables (for testing)
        if let Ok(error_type) = std::env::var("XCOM_SIMULATE_ERROR") {
            if error_type.as_str() == "rate_limit" {
                let retry_after = std::env::var("XCOM_RETRY_AFTER_MS")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(60000);
                return Err(ClassifiedError::from_status_code(
                    429,
                    "Rate limit exceeded".to_string(),
                )
                .with_retry_after(retry_after)
                .into());
            }
        }

        // Simulate unretweet operation (in real implementation, would call DELETE /2/users/{id}/retweets/{source_tweet_id})
        Ok(EngagementResult {
            tweet_id: args.tweet_id,
            success: true,
        })
    }

    /// List tweets with field projection and pagination
    pub fn list(&self, args: ListArgs) -> Result<ListResult> {
        // Check for simulated errors via environment variables (for testing)
        if let Ok(error_type) = std::env::var("XCOM_SIMULATE_ERROR") {
            match error_type.as_str() {
                "rate_limit" => {
                    let retry_after = std::env::var("XCOM_RETRY_AFTER_MS")
                        .ok()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(60000);
                    return Err(ClassifiedError::from_status_code(
                        429,
                        "Rate limit exceeded".to_string(),
                    )
                    .with_retry_after(retry_after)
                    .into());
                }
                "server_error" => {
                    return Err(ClassifiedError::from_status_code(
                        500,
                        "Internal server error".to_string(),
                    )
                    .into());
                }
                "timeout" => {
                    return Err(ClassifiedError::timeout("Request timeout".to_string()).into());
                }
                _ => {
                    // Continue with normal flow for unknown error types
                }
            }
        }

        // Simulate fetching tweets (in real implementation, would call X API)
        let limit = args.limit.unwrap_or(10);

        // Parse cursor to determine starting offset
        let offset = if let Some(cursor) = &args.cursor {
            // Cursor format is "cursor_{offset}"
            cursor
                .strip_prefix("cursor_")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0)
        } else {
            0
        };

        let mut tweets = Vec::new();
        for i in offset..(offset + limit) {
            let mut tweet = Tweet::new(format!("tweet_{}", i));
            tweet.text = Some(format!("Tweet text {}", i));
            tweet.author_id = Some(format!("user_{}", i));
            tweet.created_at = Some("2024-01-01T00:00:00Z".to_string());

            // Apply field projection
            let projected = tweet.project(&args.fields);
            tweets.push(projected);
        }

        // Create pagination metadata
        let next_cursor = if tweets.len() == limit {
            Some(format!("cursor_{}", offset + limit))
        } else {
            None
        };

        let prev_cursor = if offset > 0 {
            Some(format!("cursor_{}", offset.saturating_sub(limit)))
        } else {
            None
        };

        let meta = Some(ListResultMeta {
            pagination: PaginationMeta {
                next_cursor,
                prev_cursor,
            },
        });

        Ok(ListResult { tweets, meta })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_command() -> (TweetCommand, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let ledger = IdempotencyLedger::new(Some(&db_path)).unwrap();
        let cmd = TweetCommand::new(ledger);
        (cmd, temp_dir)
    }

    #[test]
    fn test_create_generates_client_request_id() {
        let (cmd, _temp) = create_test_command();

        let args = CreateArgs {
            text: "Hello world".to_string(),
            client_request_id: None,
            if_exists: IfExistsPolicy::Return,
        };

        let result = cmd.create(args).unwrap();
        assert!(!result.meta.client_request_id.is_empty());
        assert_eq!(result.tweet.text, Some("Hello world".to_string()));
    }

    #[test]
    fn test_create_with_explicit_client_request_id() {
        let (cmd, _temp) = create_test_command();

        let args = CreateArgs {
            text: "Hello world".to_string(),
            client_request_id: Some("my-request-id".to_string()),
            if_exists: IfExistsPolicy::Return,
        };

        let result = cmd.create(args).unwrap();
        assert_eq!(result.meta.client_request_id, "my-request-id");
    }

    #[test]
    fn test_create_idempotency_return_policy() {
        // Use ENV_LOCK to ensure XCOM_SIMULATE_ERROR is not set from other tests
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let (cmd, _temp) = create_test_command();

        let args = CreateArgs {
            text: "Hello world".to_string(),
            client_request_id: Some("test-123".to_string()),
            if_exists: IfExistsPolicy::Return,
        };

        // First call
        let result1 = cmd.create(args.clone()).unwrap();
        let tweet_id1 = result1.tweet.id.clone();

        // Second call with same ID and text should return cached result
        let result2 = cmd.create(args).unwrap();
        assert_eq!(result2.tweet.id, tweet_id1);
        assert_eq!(result2.meta.from_cache, Some(true));
    }

    #[test]
    fn test_create_idempotency_error_policy() {
        // Use ENV_LOCK to ensure XCOM_SIMULATE_ERROR is not set from other tests
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let (cmd, _temp) = create_test_command();

        let args = CreateArgs {
            text: "Hello world".to_string(),
            client_request_id: Some("test-456".to_string()),
            if_exists: IfExistsPolicy::Error,
        };

        // First call succeeds
        cmd.create(args.clone()).unwrap();

        // Second call should error
        let result = cmd.create(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_list_with_field_projection() {
        // Use ENV_LOCK to ensure XCOM_SIMULATE_ERROR is not set from other tests
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        std::env::remove_var("XCOM_RETRY_AFTER_MS");

        let (cmd, _temp) = create_test_command();

        let args = ListArgs {
            fields: vec![TweetFields::Id, TweetFields::Text],
            limit: Some(5),
            cursor: None,
        };

        let result = cmd.list(args).unwrap();
        assert_eq!(result.tweets.len(), 5);

        // Check that only requested fields are present
        for tweet in &result.tweets {
            assert!(!tweet.id.is_empty());
            assert!(tweet.text.is_some());
            assert!(tweet.author_id.is_none()); // Not requested
        }
    }

    #[test]
    fn test_list_pagination() {
        let (cmd, _temp) = create_test_command();

        let args = ListArgs {
            fields: TweetFields::default_fields(),
            limit: Some(10),
            cursor: None,
        };

        let result = cmd.list(args).unwrap();
        assert_eq!(result.tweets.len(), 10);
        assert!(result.meta.is_some());
        let meta = result.meta.unwrap();
        assert!(meta.pagination.next_cursor.is_some());
        assert_eq!(meta.pagination.next_cursor, Some("cursor_10".to_string()));
        assert!(meta.pagination.prev_cursor.is_none());
    }

    #[test]
    fn test_error_classification() {
        let err_429 = ClassifiedError::from_status_code(429, "Rate limit".to_string());
        assert_eq!(err_429.kind, ErrorKind::Retryable);
        assert!(err_429.is_retryable);

        let err_500 = ClassifiedError::from_status_code(500, "Server error".to_string());
        assert_eq!(err_500.kind, ErrorKind::Retryable);
        assert!(err_500.is_retryable);

        let err_400 = ClassifiedError::from_status_code(400, "Bad request".to_string());
        assert_eq!(err_400.kind, ErrorKind::NonRetryable);
        assert!(!err_400.is_retryable);

        let err_timeout = ClassifiedError::timeout("Timeout".to_string());
        assert_eq!(err_timeout.kind, ErrorKind::Timeout);
        assert!(err_timeout.is_retryable);
    }

    #[test]
    fn test_if_exists_policy_from_str() {
        assert_eq!(
            IfExistsPolicy::from_str("return").unwrap(),
            IfExistsPolicy::Return
        );
        assert_eq!(
            IfExistsPolicy::from_str("error").unwrap(),
            IfExistsPolicy::Error
        );
        assert!(IfExistsPolicy::from_str("invalid").is_err());
    }

    #[test]
    fn test_like_tweet() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let (cmd, _temp) = create_test_command();

        let args = EngagementArgs {
            tweet_id: "tweet_123".to_string(),
        };

        let result = cmd.like(args).unwrap();
        assert_eq!(result.tweet_id, "tweet_123");
        assert!(result.success);
    }

    #[test]
    fn test_unlike_tweet() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let (cmd, _temp) = create_test_command();

        let args = EngagementArgs {
            tweet_id: "tweet_456".to_string(),
        };

        let result = cmd.unlike(args).unwrap();
        assert_eq!(result.tweet_id, "tweet_456");
        assert!(result.success);
    }

    #[test]
    fn test_retweet() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let (cmd, _temp) = create_test_command();

        let args = EngagementArgs {
            tweet_id: "tweet_789".to_string(),
        };

        let result = cmd.retweet(args).unwrap();
        assert_eq!(result.tweet_id, "tweet_789");
        assert!(result.success);
    }

    #[test]
    fn test_unretweet() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let (cmd, _temp) = create_test_command();

        let args = EngagementArgs {
            tweet_id: "tweet_101".to_string(),
        };

        let result = cmd.unretweet(args).unwrap();
        assert_eq!(result.tweet_id, "tweet_101");
        assert!(result.success);
    }

    #[test]
    fn test_like_rate_limit_simulation() {
        // Use the global ENV_LOCK to avoid interfering with other tests
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();

        std::env::set_var("XCOM_SIMULATE_ERROR", "rate_limit");
        std::env::set_var("XCOM_RETRY_AFTER_MS", "5000");

        let (cmd, _temp) = create_test_command();
        let args = EngagementArgs {
            tweet_id: "tweet_123".to_string(),
        };

        let result = cmd.like(args);

        std::env::remove_var("XCOM_SIMULATE_ERROR");
        std::env::remove_var("XCOM_RETRY_AFTER_MS");

        assert!(result.is_err());
        let err = result.unwrap_err();
        let classified = err.downcast_ref::<ClassifiedError>().unwrap();
        assert_eq!(classified.status_code, Some(429));
        assert!(classified.is_retryable);
    }

    #[test]
    fn test_engagement_result_serialization() {
        let result = EngagementResult {
            tweet_id: "tweet_123".to_string(),
            success: true,
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["tweet_id"], "tweet_123");
        assert_eq!(parsed["success"], true);
    }
}
