//! Tweet creation and idempotency handling.

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::tweets::ledger::IdempotencyLedger;
use crate::tweets::models::{Tweet, TweetMeta};

use super::types::{
    ClassifiedError, CreateArgs, CreateResult, IdempotencyConflictError, IfExistsPolicy,
};

/// Create a tweet with idempotency support.
pub fn create(ledger: &IdempotencyLedger, args: CreateArgs) -> Result<CreateResult> {
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
    if let Some(entry) = ledger
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
    ledger
        .record(&client_request_id, &request_hash, &tweet_id, "success")
        .context("Failed to record operation in ledger")?;

    let meta = TweetMeta {
        client_request_id,
        from_cache: None,
    };

    Ok(CreateResult { tweet, meta })
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

    /// Characterization test: create generates a non-empty client_request_id
    #[test]
    fn test_create_generates_client_request_id() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        std::env::remove_var("XCOM_RETRY_AFTER_MS");

        let (ledger, _temp) = create_test_ledger();
        let args = CreateArgs {
            text: "Hello world".to_string(),
            client_request_id: None,
            if_exists: IfExistsPolicy::Return,
        };
        let result = create(&ledger, args).unwrap();
        assert!(!result.meta.client_request_id.is_empty());
        assert_eq!(result.tweet.text, Some("Hello world".to_string()));
    }

    /// Characterization test: explicit client_request_id is preserved
    #[test]
    fn test_create_with_explicit_client_request_id() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        std::env::remove_var("XCOM_RETRY_AFTER_MS");

        let (ledger, _temp) = create_test_ledger();
        let args = CreateArgs {
            text: "Hello world".to_string(),
            client_request_id: Some("my-request-id".to_string()),
            if_exists: IfExistsPolicy::Return,
        };
        let result = create(&ledger, args).unwrap();
        assert_eq!(result.meta.client_request_id, "my-request-id");
    }

    /// Characterization test: idempotency return policy returns cached result
    #[test]
    fn test_create_idempotency_return_policy() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let (ledger, _temp) = create_test_ledger();
        let args = CreateArgs {
            text: "Hello world".to_string(),
            client_request_id: Some("test-123".to_string()),
            if_exists: IfExistsPolicy::Return,
        };

        let result1 = create(&ledger, args.clone()).unwrap();
        let tweet_id1 = result1.tweet.id.clone();

        let result2 = create(&ledger, args).unwrap();
        assert_eq!(result2.tweet.id, tweet_id1);
        assert_eq!(result2.meta.from_cache, Some(true));
    }

    /// Characterization test: idempotency error policy rejects duplicate
    #[test]
    fn test_create_idempotency_error_policy() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let (ledger, _temp) = create_test_ledger();
        let args = CreateArgs {
            text: "Hello world".to_string(),
            client_request_id: Some("test-456".to_string()),
            if_exists: IfExistsPolicy::Error,
        };

        create(&ledger, args.clone()).unwrap();

        let result = create(&ledger, args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    /// Characterization test: rate limit simulation returns ClassifiedError
    #[test]
    fn test_create_rate_limit_simulation() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("XCOM_SIMULATE_ERROR", "rate_limit");
        std::env::set_var("XCOM_RETRY_AFTER_MS", "5000");

        let (ledger, _temp) = create_test_ledger();
        let args = CreateArgs {
            text: "Hello world".to_string(),
            client_request_id: None,
            if_exists: IfExistsPolicy::Return,
        };

        let result = create(&ledger, args);
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        std::env::remove_var("XCOM_RETRY_AFTER_MS");

        assert!(result.is_err());
        let err = result.unwrap_err();
        let classified = err.downcast_ref::<ClassifiedError>().unwrap();
        assert_eq!(classified.status_code, Some(429));
        assert!(classified.is_retryable);
        assert_eq!(classified.retry_after_ms, Some(5000));
    }
}
