/// Integration tests for tweets operations with HTTP mocking
use tempfile::TempDir;
use xcom_rs::tweets::{CreateArgs, IdempotencyLedger, IfExistsPolicy, TweetCommand};

/// Test simulating a timeout followed by successful retry
#[test]
fn test_timeout_retry_with_ledger() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let ledger = IdempotencyLedger::new(Some(&db_path)).unwrap();
    let cmd = TweetCommand::new(ledger);

    let client_request_id = "retry-test-123";

    // First attempt - simulate creating tweet
    let args1 = CreateArgs {
        text: "Hello retry world".to_string(),
        client_request_id: Some(client_request_id.to_string()),
        if_exists: IfExistsPolicy::Return,
    };

    let result1 = cmd.create(args1).unwrap();
    let tweet_id1 = result1.tweet.id.clone();
    assert_eq!(result1.meta.client_request_id, client_request_id);
    assert_eq!(result1.meta.from_cache, None);

    // Simulate timeout - client doesn't receive response but ledger was updated

    // Second attempt - retry with same parameters
    let args2 = CreateArgs {
        text: "Hello retry world".to_string(),
        client_request_id: Some(client_request_id.to_string()),
        if_exists: IfExistsPolicy::Return,
    };

    let result2 = cmd.create(args2).unwrap();
    assert_eq!(result2.tweet.id, tweet_id1); // Should return same tweet
    assert_eq!(result2.meta.from_cache, Some(true)); // Indicates cached result
    assert_eq!(result2.meta.client_request_id, client_request_id);
}

/// Test that different request content with same client_request_id creates new operation
#[test]
fn test_different_content_same_client_request_id() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let ledger = IdempotencyLedger::new(Some(&db_path)).unwrap();
    let cmd = TweetCommand::new(ledger);

    let client_request_id = "test-456";

    // First request
    let args1 = CreateArgs {
        text: "First message".to_string(),
        client_request_id: Some(client_request_id.to_string()),
        if_exists: IfExistsPolicy::Return,
    };
    let result1 = cmd.create(args1).unwrap();
    let tweet_id1 = result1.tweet.id.clone();

    // Second request with DIFFERENT text but SAME client_request_id
    // This should NOT return cached result because request hash differs
    let args2 = CreateArgs {
        text: "Second message".to_string(),
        client_request_id: Some(client_request_id.to_string()),
        if_exists: IfExistsPolicy::Return,
    };
    let result2 = cmd.create(args2).unwrap();
    let tweet_id2 = result2.tweet.id.clone();

    // Should create new tweet because content is different
    assert_ne!(tweet_id1, tweet_id2);
    assert_eq!(result2.meta.from_cache, None);
}

/// Test error policy when duplicate detected
#[test]
fn test_if_exists_error_policy() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let ledger = IdempotencyLedger::new(Some(&db_path)).unwrap();
    let cmd = TweetCommand::new(ledger);

    let client_request_id = "error-test-789";

    // First request succeeds
    let args1 = CreateArgs {
        text: "Test message".to_string(),
        client_request_id: Some(client_request_id.to_string()),
        if_exists: IfExistsPolicy::Error,
    };
    cmd.create(args1).unwrap();

    // Second request with same params should error
    let args2 = CreateArgs {
        text: "Test message".to_string(),
        client_request_id: Some(client_request_id.to_string()),
        if_exists: IfExistsPolicy::Error,
    };
    let result = cmd.create(args2);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

/// Test NDJSON output format for list
#[test]
fn test_ndjson_output_format() {
    use xcom_rs::tweets::{ListArgs, TweetFields};

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let ledger = IdempotencyLedger::new(Some(&db_path)).unwrap();
    let cmd = TweetCommand::new(ledger);

    let args = ListArgs {
        fields: vec![TweetFields::Id, TweetFields::Text],
        limit: Some(3),
        cursor: None,
    };

    let result = cmd.list(args).unwrap();

    // Verify each tweet can be serialized to JSON
    for tweet in &result.tweets {
        let json = serde_json::to_string(tweet).unwrap();
        assert!(json.contains("\"id\""));
        assert!(!json.contains('\n')); // NDJSON should be single line
    }
}

/// Test field projection
#[test]
fn test_field_projection() {
    use xcom_rs::tweets::{ListArgs, TweetFields};

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let ledger = IdempotencyLedger::new(Some(&db_path)).unwrap();
    let cmd = TweetCommand::new(ledger);

    let args = ListArgs {
        fields: vec![TweetFields::Id, TweetFields::Text],
        limit: Some(5),
        cursor: None,
    };

    let result = cmd.list(args).unwrap();
    assert_eq!(result.tweets.len(), 5);

    // Verify only requested fields are populated
    for tweet in &result.tweets {
        assert!(!tweet.id.is_empty());
        assert!(tweet.text.is_some());
        assert!(tweet.author_id.is_none());
        assert!(tweet.created_at.is_none());
    }
}

/// Test retryable error classification
#[test]
fn test_error_classification() {
    use xcom_rs::tweets::commands::{ClassifiedError, ErrorKind};

    // Test 429 (rate limit) - retryable
    let err_429 = ClassifiedError::from_status_code(429, "Rate limited".to_string());
    assert_eq!(err_429.kind, ErrorKind::Retryable);
    assert!(err_429.is_retryable);
    assert_eq!(err_429.status_code, Some(429));

    // Test 5xx (server error) - retryable
    let err_500 = ClassifiedError::from_status_code(500, "Server error".to_string());
    assert_eq!(err_500.kind, ErrorKind::Retryable);
    assert!(err_500.is_retryable);

    let err_503 = ClassifiedError::from_status_code(503, "Service unavailable".to_string());
    assert_eq!(err_503.kind, ErrorKind::Retryable);
    assert!(err_503.is_retryable);

    // Test 4xx (client error) - non-retryable
    let err_400 = ClassifiedError::from_status_code(400, "Bad request".to_string());
    assert_eq!(err_400.kind, ErrorKind::NonRetryable);
    assert!(!err_400.is_retryable);

    let err_404 = ClassifiedError::from_status_code(404, "Not found".to_string());
    assert_eq!(err_404.kind, ErrorKind::NonRetryable);
    assert!(!err_404.is_retryable);

    // Test timeout - retryable
    let err_timeout = ClassifiedError::timeout("Request timeout".to_string());
    assert_eq!(err_timeout.kind, ErrorKind::Timeout);
    assert!(err_timeout.is_retryable);
    assert_eq!(err_timeout.status_code, None);
}
