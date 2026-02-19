use super::{
    client::{fetch_conversation, TweetApiClient},
    ledger::IdempotencyLedger,
    models::{ConversationResult, Tweet, TweetFields, TweetMeta},
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

/// Custom error type for partial thread failures.
/// Contains information about which tweet in the thread failed,
/// and which tweets were successfully created before the failure.
#[derive(Debug)]
pub struct ThreadPartialFailureError {
    /// Index of the tweet that failed (0-based)
    pub failed_index: usize,
    /// IDs of tweets successfully created before the failure
    pub created_tweet_ids: Vec<String>,
    /// Underlying error message
    pub message: String,
}

impl std::fmt::Display for ThreadPartialFailureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Thread posting failed at index {}: {} ({} tweets created before failure)",
            self.failed_index,
            self.message,
            self.created_tweet_ids.len()
        )
    }
}

impl std::error::Error for ThreadPartialFailureError {}

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

/// Arguments for replying to a tweet
#[derive(Debug, Clone)]
pub struct ReplyArgs {
    pub tweet_id: String,
    pub text: String,
    pub client_request_id: Option<String>,
    pub if_exists: IfExistsPolicy,
}

/// Arguments for creating a thread of tweets
#[derive(Debug, Clone)]
pub struct ThreadArgs {
    pub texts: Vec<String>,
    pub client_request_id_prefix: Option<String>,
    pub if_exists: IfExistsPolicy,
}

/// Arguments for showing a single tweet
#[derive(Debug, Clone)]
pub struct ShowArgs {
    pub tweet_id: String,
}

/// Arguments for retrieving a conversation tree
#[derive(Debug, Clone)]
pub struct ConversationArgs {
    pub tweet_id: String,
}

/// Result of a create operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResult {
    pub tweet: Tweet,
    pub meta: TweetMeta,
}

/// Result of a reply operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyResult {
    pub tweet: Tweet,
    pub meta: TweetMeta,
}

/// Result of a thread operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadResult {
    pub tweets: Vec<Tweet>,
    pub meta: ThreadMeta,
}

/// Metadata for thread results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadMeta {
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_index: Option<usize>,
    pub created_tweet_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_cache: Option<bool>,
}

/// Result of a show operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowResult {
    pub tweet: Tweet,
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
    api_client: Box<dyn TweetApiClient>,
}

impl TweetCommand {
    /// Create a new tweet command handler with a default stub API client
    pub fn new(ledger: IdempotencyLedger) -> Self {
        Self {
            ledger,
            api_client: Box::new(super::client::MockTweetApiClient::new()),
        }
    }

    /// Create a new tweet command handler with a custom API client
    pub fn with_client(ledger: IdempotencyLedger, client: Box<dyn TweetApiClient>) -> Self {
        Self {
            ledger,
            api_client: client,
        }
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

    /// Reply to a tweet with idempotency support
    pub fn reply(&self, args: ReplyArgs) -> Result<ReplyResult> {
        let client_request_id = args
            .client_request_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let request_hash = IdempotencyLedger::compute_request_hash(&args.text);

        // Check ledger for existing operation
        if let Some(entry) = self
            .ledger
            .lookup(&client_request_id)
            .context("Failed to lookup operation in ledger")?
        {
            match args.if_exists {
                IfExistsPolicy::Return => {
                    let mut tweet = Tweet::new(entry.tweet_id.clone());
                    tweet.text = Some(args.text.clone());
                    let meta = TweetMeta {
                        client_request_id,
                        from_cache: Some(true),
                    };
                    return Ok(ReplyResult { tweet, meta });
                }
                IfExistsPolicy::Error => {
                    return Err(IdempotencyConflictError {
                        client_request_id: client_request_id.clone(),
                    }
                    .into());
                }
            }
        }

        // Post the reply via API client
        let tweet = self
            .api_client
            .post_tweet(&args.text, Some(&args.tweet_id))
            .context("Failed to post reply")?;

        self.ledger
            .record(&client_request_id, &request_hash, &tweet.id, "success")
            .context("Failed to record operation in ledger")?;

        let meta = TweetMeta {
            client_request_id,
            from_cache: None,
        };

        Ok(ReplyResult { tweet, meta })
    }

    /// Post a thread of tweets (sequential replies)
    pub fn thread(&self, args: ThreadArgs) -> Result<ThreadResult> {
        if args.texts.is_empty() {
            return Err(anyhow!("Thread must contain at least one tweet"));
        }

        let prefix = args
            .client_request_id_prefix
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let mut created_tweets: Vec<Tweet> = Vec::new();
        let mut created_ids: Vec<String> = Vec::new();
        let mut previous_id: Option<String> = None;

        for (index, text) in args.texts.iter().enumerate() {
            let client_request_id = format!("{}-{}", prefix, index);
            let request_hash = IdempotencyLedger::compute_request_hash(text);

            // Check ledger for existing operation
            if let Some(entry) = self
                .ledger
                .lookup(&client_request_id)
                .context("Failed to lookup operation in ledger")?
            {
                match args.if_exists {
                    IfExistsPolicy::Return => {
                        let mut tweet = Tweet::new(entry.tweet_id.clone());
                        tweet.text = Some(text.clone());
                        created_ids.push(tweet.id.clone());
                        previous_id = Some(tweet.id.clone());
                        created_tweets.push(tweet);
                        continue;
                    }
                    IfExistsPolicy::Error => {
                        return Err(IdempotencyConflictError {
                            client_request_id: client_request_id.clone(),
                        }
                        .into());
                    }
                }
            }

            // Post tweet (first tweet is standalone, rest are replies)
            let tweet_result = self.api_client.post_tweet(text, previous_id.as_deref());

            let tweet = match tweet_result {
                Ok(t) => t,
                Err(e) => {
                    // Return structured error with partial failure information
                    return Err(ThreadPartialFailureError {
                        failed_index: index,
                        created_tweet_ids: created_ids,
                        message: e.to_string(),
                    }
                    .into());
                }
            };

            self.ledger
                .record(&client_request_id, &request_hash, &tweet.id, "success")
                .context("Failed to record operation in ledger")?;

            created_ids.push(tweet.id.clone());
            previous_id = Some(tweet.id.clone());
            created_tweets.push(tweet);
        }

        let meta = ThreadMeta {
            count: created_tweets.len(),
            failed_index: None,
            created_tweet_ids: created_ids,
            from_cache: None,
        };

        Ok(ThreadResult {
            tweets: created_tweets,
            meta,
        })
    }

    /// Show a single tweet by ID
    pub fn show(&self, args: ShowArgs) -> Result<ShowResult> {
        let tweet = self
            .api_client
            .get_tweet(&args.tweet_id)
            .context("Failed to fetch tweet")?;
        Ok(ShowResult { tweet })
    }

    /// Retrieve a conversation tree starting from a tweet
    pub fn conversation(&self, args: ConversationArgs) -> Result<ConversationResult> {
        fetch_conversation(self.api_client.as_ref(), &args.tweet_id)
            .context("Failed to fetch conversation")
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

    fn create_test_command_with_fixture() -> (TweetCommand, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let ledger = IdempotencyLedger::new(Some(&db_path)).unwrap();
        let client =
            Box::new(crate::tweets::client::MockTweetApiClient::with_conversation_fixture());
        let cmd = TweetCommand::with_client(ledger, client);
        (cmd, temp_dir)
    }

    #[test]
    fn test_reply_creates_tweet_with_reference() {
        let (cmd, _temp) = create_test_command_with_fixture();

        let args = ReplyArgs {
            tweet_id: "tweet_root".to_string(),
            text: "My reply".to_string(),
            client_request_id: None,
            if_exists: IfExistsPolicy::Return,
        };

        let result = cmd.reply(args).unwrap();
        assert_eq!(result.tweet.text, Some("My reply".to_string()));
        assert!(!result.meta.client_request_id.is_empty());
        // The mock client adds a referenced_tweets entry for replied_to
        assert!(result.tweet.referenced_tweets.is_some());
        let refs = result.tweet.referenced_tweets.unwrap();
        assert_eq!(refs[0].ref_type, "replied_to");
        assert_eq!(refs[0].id, "tweet_root");
    }

    #[test]
    fn test_reply_idempotency_return() {
        let (cmd, _temp) = create_test_command_with_fixture();

        let args = ReplyArgs {
            tweet_id: "tweet_root".to_string(),
            text: "My reply".to_string(),
            client_request_id: Some("reply-001".to_string()),
            if_exists: IfExistsPolicy::Return,
        };

        let result1 = cmd.reply(args.clone()).unwrap();
        let result2 = cmd.reply(args).unwrap();
        // Second call returns cached result
        assert_eq!(result2.meta.from_cache, Some(true));
        assert_eq!(
            result1.meta.client_request_id,
            result2.meta.client_request_id
        );
    }

    #[test]
    fn test_thread_posts_multiple_tweets() {
        let (cmd, _temp) = create_test_command_with_fixture();

        let args = ThreadArgs {
            texts: vec![
                "First tweet".to_string(),
                "Second tweet".to_string(),
                "Third tweet".to_string(),
            ],
            client_request_id_prefix: Some("thread-001".to_string()),
            if_exists: IfExistsPolicy::Return,
        };

        let result = cmd.thread(args).unwrap();
        assert_eq!(result.tweets.len(), 3);
        assert_eq!(result.meta.count, 3);
        assert_eq!(result.meta.created_tweet_ids.len(), 3);
        assert!(result.meta.failed_index.is_none());
    }

    #[test]
    fn test_thread_empty_fails() {
        let (cmd, _temp) = create_test_command_with_fixture();

        let args = ThreadArgs {
            texts: vec![],
            client_request_id_prefix: None,
            if_exists: IfExistsPolicy::Return,
        };

        let result = cmd.thread(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_thread_partial_failure_contains_structured_error() {
        // Create a mock client that fails on the second post
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let ledger = IdempotencyLedger::new(Some(&db_path)).unwrap();

        // We use a custom mock that fails after the first tweet
        let mut error_client = crate::tweets::client::MockTweetApiClient::new();
        // The mock client with simulate_error=true will fail every call,
        // so we test with a client that starts with error enabled
        error_client.simulate_error = true;

        let cmd = TweetCommand::with_client(ledger, Box::new(error_client));

        let args = ThreadArgs {
            texts: vec![
                "First tweet".to_string(),
                "Second tweet".to_string(),
                "Third tweet".to_string(),
            ],
            client_request_id_prefix: Some("thread-fail-test".to_string()),
            if_exists: IfExistsPolicy::Return,
        };

        let result = cmd.thread(args);
        assert!(result.is_err());

        let err = result.unwrap_err();
        let partial_failure = err.downcast_ref::<ThreadPartialFailureError>();
        assert!(
            partial_failure.is_some(),
            "Expected ThreadPartialFailureError but got different error type"
        );

        let pf = partial_failure.unwrap();
        assert_eq!(
            pf.failed_index, 0,
            "Should fail at index 0 since mock errors on all calls"
        );
        assert!(
            pf.created_tweet_ids.is_empty(),
            "No tweets should be created before first failure"
        );
    }

    #[test]
    fn test_thread_partial_failure_after_some_success() {
        use crate::tweets::client::MockTweetApiClient;

        // Custom client that succeeds for first N tweets then fails
        // We test this by creating a client that fails, but we can pre-configure
        // some posts via ledger to simulate partial success
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let ledger = IdempotencyLedger::new(Some(&db_path)).unwrap();

        // Pre-populate ledger with first tweet "already created"
        let prefix = "partial-fail-prefix";
        let request_hash = IdempotencyLedger::compute_request_hash("First tweet");
        ledger
            .record(
                &format!("{}-0", prefix),
                &request_hash,
                "tweet_pre_created_0",
                "success",
            )
            .unwrap();

        // Now the api client will fail on actual calls
        let mut error_client = MockTweetApiClient::new();
        error_client.simulate_error = true;
        let cmd = TweetCommand::with_client(ledger, Box::new(error_client));

        let args = ThreadArgs {
            texts: vec!["First tweet".to_string(), "Second tweet".to_string()],
            client_request_id_prefix: Some(prefix.to_string()),
            if_exists: IfExistsPolicy::Return,
        };

        let result = cmd.thread(args);
        assert!(result.is_err());

        let err = result.unwrap_err();
        let partial_failure = err.downcast_ref::<ThreadPartialFailureError>();
        assert!(
            partial_failure.is_some(),
            "Expected ThreadPartialFailureError"
        );

        let pf = partial_failure.unwrap();
        assert_eq!(
            pf.failed_index, 1,
            "Should fail at index 1 (first was from ledger cache)"
        );
        assert_eq!(
            pf.created_tweet_ids.len(),
            1,
            "One tweet should be in created_tweet_ids"
        );
        assert_eq!(pf.created_tweet_ids[0], "tweet_pre_created_0");
    }

    #[test]
    fn test_show_returns_tweet() {
        let (cmd, _temp) = create_test_command_with_fixture();

        let args = ShowArgs {
            tweet_id: "tweet_root".to_string(),
        };

        let result = cmd.show(args).unwrap();
        assert_eq!(result.tweet.id, "tweet_root");
        assert_eq!(
            result.tweet.conversation_id,
            Some("conv_root_001".to_string())
        );
    }

    #[test]
    fn test_show_not_found() {
        let (cmd, _temp) = create_test_command_with_fixture();

        let args = ShowArgs {
            tweet_id: "nonexistent_tweet".to_string(),
        };

        let result = cmd.show(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_conversation_returns_tree() {
        let (cmd, _temp) = create_test_command_with_fixture();

        let args = ConversationArgs {
            tweet_id: "tweet_root".to_string(),
        };

        let result = cmd.conversation(args).unwrap();
        // Should include root + replies
        assert!(!result.posts.is_empty());
        assert!(result.posts.iter().any(|t| t.id == "tweet_root"));
        // Should have edges connecting replies to parents
        assert!(!result.edges.is_empty());
        // Should include conversation_id in the result
        assert!(
            !result.conversation_id.is_empty(),
            "conversation_id should be present in ConversationResult"
        );
        assert_eq!(
            result.conversation_id, "conv_root_001",
            "conversation_id should match the root tweet's conversation_id"
        );
    }

    #[test]
    fn test_conversation_edges_structure() {
        let (cmd, _temp) = create_test_command_with_fixture();

        let args = ConversationArgs {
            tweet_id: "tweet_root".to_string(),
        };

        let result = cmd.conversation(args).unwrap();
        // tweet_reply1 is a reply to tweet_root
        let root_edge = result
            .edges
            .iter()
            .find(|e| e.parent_id == "tweet_root" && e.child_id == "tweet_reply1");
        assert!(root_edge.is_some(), "Expected edge from root to reply1");

        // tweet_reply2 is a reply to tweet_reply1
        let reply_edge = result
            .edges
            .iter()
            .find(|e| e.parent_id == "tweet_reply1" && e.child_id == "tweet_reply2");
        assert!(reply_edge.is_some(), "Expected edge from reply1 to reply2");
    }
}
