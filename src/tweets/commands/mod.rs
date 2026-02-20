//! Tweets command handlers organized by feature.
//!
//! Each sub-module contains argument types, result types, and the implementation
//! for a specific feature area. All public types are re-exported from this module
//! for backward compatibility.

pub mod create;
pub mod engagement;
pub mod list;
pub mod show;
pub mod thread;
pub mod types;

// Re-export all public types for backward compatibility
pub use types::{
    ClassifiedError, ConversationArgs, CreateArgs, CreateResult, EngagementArgs, EngagementResult,
    ErrorKind, IdempotencyConflictError, IfExistsPolicy, ListArgs, ListResult, ListResultMeta,
    PaginationMeta, ReplyArgs, ReplyResult, ShowArgs, ShowResult, ThreadArgs, ThreadMeta,
    ThreadPartialFailureError, ThreadResult,
};

use crate::tweets::{
    client::TweetApiClient, http_client::XApiClient, ledger::IdempotencyLedger,
    models::ConversationResult,
};
use anyhow::Result;

/// Main tweets command handler.
///
/// Delegates to feature-specific modules for each operation while providing
/// a unified entry point for the CLI.
pub struct TweetCommand {
    ledger: IdempotencyLedger,
    api_client: Box<dyn TweetApiClient>,
    http_client: XApiClient,
}

impl TweetCommand {
    /// Create a new tweet command handler with a default stub API client
    pub fn new(ledger: IdempotencyLedger) -> Self {
        Self {
            ledger,
            api_client: Box::new(crate::tweets::client::MockTweetApiClient::new()),
            http_client: XApiClient::new(),
        }
    }

    /// Create a new tweet command handler with a custom API client
    pub fn with_client(ledger: IdempotencyLedger, client: Box<dyn TweetApiClient>) -> Self {
        Self {
            ledger,
            api_client: client,
            http_client: XApiClient::new(),
        }
    }

    /// Create a tweet with idempotency support
    pub fn create(&self, args: CreateArgs) -> Result<CreateResult> {
        create::create(&self.ledger, &self.http_client, args)
    }

    /// Like a tweet
    pub fn like(&self, args: EngagementArgs) -> Result<EngagementResult> {
        engagement::like(&self.http_client, args)
    }

    /// Unlike a tweet
    pub fn unlike(&self, args: EngagementArgs) -> Result<EngagementResult> {
        engagement::unlike(&self.http_client, args)
    }

    /// Retweet a tweet
    pub fn retweet(&self, args: EngagementArgs) -> Result<EngagementResult> {
        engagement::retweet(&self.http_client, args)
    }

    /// Unretweet a tweet
    pub fn unretweet(&self, args: EngagementArgs) -> Result<EngagementResult> {
        engagement::unretweet(&self.http_client, args)
    }

    /// List tweets with field projection and pagination
    pub fn list(&self, args: ListArgs) -> Result<ListResult> {
        list::list(args)
    }

    /// Reply to a tweet with idempotency support
    pub fn reply(&self, args: ReplyArgs) -> Result<ReplyResult> {
        thread::reply(&self.ledger, &self.http_client, args)
    }

    /// Post a thread of tweets (sequential replies)
    pub fn thread(&self, args: ThreadArgs) -> Result<ThreadResult> {
        thread::thread(&self.ledger, &self.http_client, args)
    }

    /// Show a single tweet by ID
    pub fn show(&self, args: ShowArgs) -> Result<ShowResult> {
        show::show(self.api_client.as_ref(), args)
    }

    /// Retrieve a conversation tree starting from a tweet
    pub fn conversation(&self, args: ConversationArgs) -> Result<ConversationResult> {
        show::conversation(self.api_client.as_ref(), args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweets::{ledger::IdempotencyLedger, models::TweetFields};
    use tempfile::TempDir;

    fn create_test_command() -> (TweetCommand, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let ledger = IdempotencyLedger::new(Some(&db_path)).unwrap();
        let cmd = TweetCommand::new(ledger);
        (cmd, temp_dir)
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

    // --- Create tests ---

    #[test]
    #[ignore] // Requires mock server setup
    fn test_create_generates_client_request_id() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        std::env::remove_var("XCOM_RETRY_AFTER_MS");

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
    #[ignore] // Requires mock server setup
    fn test_create_with_explicit_client_request_id() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        std::env::remove_var("XCOM_RETRY_AFTER_MS");

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
    #[ignore] // Requires mock server setup
    fn test_create_idempotency_return_policy() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let (cmd, _temp) = create_test_command();
        let args = CreateArgs {
            text: "Hello world".to_string(),
            client_request_id: Some("test-123".to_string()),
            if_exists: IfExistsPolicy::Return,
        };
        let result1 = cmd.create(args.clone()).unwrap();
        let tweet_id1 = result1.tweet.id.clone();
        let result2 = cmd.create(args).unwrap();
        assert_eq!(result2.tweet.id, tweet_id1);
        assert_eq!(result2.meta.from_cache, Some(true));
    }

    #[test]
    #[ignore] // Requires mock server setup
    fn test_create_idempotency_error_policy() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let (cmd, _temp) = create_test_command();
        let args = CreateArgs {
            text: "Hello world".to_string(),
            client_request_id: Some("test-456".to_string()),
            if_exists: IfExistsPolicy::Error,
        };
        cmd.create(args.clone()).unwrap();
        let result = cmd.create(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    // --- List tests ---

    #[test]
    fn test_list_with_field_projection() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
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
        for tweet in &result.tweets {
            assert!(!tweet.id.is_empty());
            assert!(tweet.text.is_some());
            assert!(tweet.author_id.is_none());
        }
    }

    #[test]
    fn test_list_pagination() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        std::env::remove_var("XCOM_RETRY_AFTER_MS");

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
        assert_eq!(meta.pagination.next_cursor, Some("cursor_10".to_string()));
        assert!(meta.pagination.prev_cursor.is_none());
    }

    // --- Error classification tests ---

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
        use std::str::FromStr;
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

    // --- Engagement tests ---

    #[test]
    #[ignore] // Requires mock server setup
    fn test_like_tweet() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        let (cmd, _temp) = create_test_command();
        let result = cmd
            .like(EngagementArgs {
                tweet_id: "tweet_123".to_string(),
            })
            .unwrap();
        assert_eq!(result.tweet_id, "tweet_123");
        assert!(result.success);
    }

    #[test]
    #[ignore] // Requires mock server setup
    fn test_unlike_tweet() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        let (cmd, _temp) = create_test_command();
        let result = cmd
            .unlike(EngagementArgs {
                tweet_id: "tweet_456".to_string(),
            })
            .unwrap();
        assert_eq!(result.tweet_id, "tweet_456");
        assert!(result.success);
    }

    #[test]
    #[ignore] // Requires mock server setup
    fn test_retweet() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        let (cmd, _temp) = create_test_command();
        let result = cmd
            .retweet(EngagementArgs {
                tweet_id: "tweet_789".to_string(),
            })
            .unwrap();
        assert_eq!(result.tweet_id, "tweet_789");
        assert!(result.success);
    }

    #[test]
    #[ignore] // Requires mock server setup
    fn test_unretweet() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        let (cmd, _temp) = create_test_command();
        let result = cmd
            .unretweet(EngagementArgs {
                tweet_id: "tweet_101".to_string(),
            })
            .unwrap();
        assert_eq!(result.tweet_id, "tweet_101");
        assert!(result.success);
    }

    #[test]
    #[ignore] // Requires mock server setup
    fn test_like_rate_limit_simulation() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::set_var("XCOM_SIMULATE_ERROR", "rate_limit");
        std::env::set_var("XCOM_RETRY_AFTER_MS", "5000");
        let (cmd, _temp) = create_test_command();
        let result = cmd.like(EngagementArgs {
            tweet_id: "tweet_123".to_string(),
        });
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

    // --- Reply / Thread tests ---

    #[test]
    #[ignore] // Requires mock server setup
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
        assert!(result.tweet.referenced_tweets.is_some());
        let refs = result.tweet.referenced_tweets.unwrap();
        assert_eq!(refs[0].ref_type, "replied_to");
        assert_eq!(refs[0].id, "tweet_root");
    }

    #[test]
    #[ignore] // Requires mock server setup
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
        assert_eq!(result2.meta.from_cache, Some(true));
        assert_eq!(
            result1.meta.client_request_id,
            result2.meta.client_request_id
        );
    }

    #[test]
    #[ignore] // Requires mock server setup
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
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let ledger = IdempotencyLedger::new(Some(&db_path)).unwrap();
        let mut error_client = crate::tweets::client::MockTweetApiClient::new();
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
            "Expected ThreadPartialFailureError"
        );
        let pf = partial_failure.unwrap();
        assert_eq!(pf.failed_index, 0);
        assert!(pf.created_tweet_ids.is_empty());
    }

    #[test]
    fn test_thread_partial_failure_after_some_success() {
        use crate::tweets::client::MockTweetApiClient;
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let ledger = IdempotencyLedger::new(Some(&db_path)).unwrap();
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
        assert!(partial_failure.is_some());
        let pf = partial_failure.unwrap();
        assert_eq!(pf.failed_index, 1);
        assert_eq!(pf.created_tweet_ids.len(), 1);
        assert_eq!(pf.created_tweet_ids[0], "tweet_pre_created_0");
    }

    // --- Show / Conversation tests ---

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
        assert!(!result.posts.is_empty());
        assert!(result.posts.iter().any(|t| t.id == "tweet_root"));
        assert!(!result.edges.is_empty());
        assert!(
            !result.conversation_id.is_empty(),
            "conversation_id should be present"
        );
        assert_eq!(result.conversation_id, "conv_root_001");
    }

    #[test]
    fn test_conversation_edges_structure() {
        let (cmd, _temp) = create_test_command_with_fixture();
        let args = ConversationArgs {
            tweet_id: "tweet_root".to_string(),
        };
        let result = cmd.conversation(args).unwrap();
        let root_edge = result
            .edges
            .iter()
            .find(|e| e.parent_id == "tweet_root" && e.child_id == "tweet_reply1");
        assert!(root_edge.is_some(), "Expected edge from root to reply1");
        let reply_edge = result
            .edges
            .iter()
            .find(|e| e.parent_id == "tweet_reply1" && e.child_id == "tweet_reply2");
        assert!(reply_edge.is_some(), "Expected edge from reply1 to reply2");
    }
}
