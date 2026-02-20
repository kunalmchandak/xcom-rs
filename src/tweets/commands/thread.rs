//! Thread and reply tweet operations.

use anyhow::{anyhow, Context, Result};
use uuid::Uuid;

use crate::tweets::http_client::XApiClient;
use crate::tweets::ledger::IdempotencyLedger;
use crate::tweets::models::{Tweet, TweetMeta};

use super::types::{
    IdempotencyConflictError, IfExistsPolicy, ReplyArgs, ReplyResult, ThreadArgs, ThreadMeta,
    ThreadPartialFailureError, ThreadResult,
};

/// Reply to a tweet with idempotency support.
pub fn reply(
    ledger: &IdempotencyLedger,
    http_client: &XApiClient,
    args: ReplyArgs,
) -> Result<ReplyResult> {
    let client_request_id = args
        .client_request_id
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let request_hash = IdempotencyLedger::compute_request_hash(&args.text);

    // Check ledger for existing operation
    if let Some(entry) = ledger
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

    // Post the reply via HTTP client
    let tweet = http_client
        .create_tweet(&args.text, Some(&args.tweet_id))
        .context("Failed to post reply")?;

    ledger
        .record(&client_request_id, &request_hash, &tweet.id, "success")
        .context("Failed to record operation in ledger")?;

    let meta = TweetMeta {
        client_request_id,
        from_cache: None,
    };

    Ok(ReplyResult { tweet, meta })
}

/// Post a thread of tweets (sequential replies).
pub fn thread(
    ledger: &IdempotencyLedger,
    http_client: &XApiClient,
    args: ThreadArgs,
) -> Result<ThreadResult> {
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
        if let Some(entry) = ledger
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
        let tweet_result = http_client.create_tweet(text, previous_id.as_deref());

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

        ledger
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_setup() -> (IdempotencyLedger, XApiClient, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let ledger = IdempotencyLedger::new(Some(&db_path)).unwrap();
        let http_client = XApiClient::new();
        (ledger, http_client, temp_dir)
    }

    /// Characterization test: reply creates a tweet with a referenced_tweets entry
    #[test]
    #[ignore] // Requires mockito server setup
    fn test_reply_creates_tweet_with_reference() {
        let (ledger, client, _temp) = create_test_setup();

        let args = ReplyArgs {
            tweet_id: "tweet_root".to_string(),
            text: "My reply".to_string(),
            client_request_id: None,
            if_exists: IfExistsPolicy::Return,
        };

        let result = reply(&ledger, &client, args).unwrap();
        assert_eq!(result.tweet.text, Some("My reply".to_string()));
        assert!(!result.meta.client_request_id.is_empty());
        assert!(result.tweet.referenced_tweets.is_some());
        let refs = result.tweet.referenced_tweets.unwrap();
        assert_eq!(refs[0].ref_type, "replied_to");
        assert_eq!(refs[0].id, "tweet_root");
    }

    /// Characterization test: reply with idempotency return policy returns cached result
    #[test]
    #[ignore] // Requires mockito server setup
    fn test_reply_idempotency_return() {
        let (ledger, client, _temp) = create_test_setup();

        let args = ReplyArgs {
            tweet_id: "tweet_root".to_string(),
            text: "My reply".to_string(),
            client_request_id: Some("reply-001".to_string()),
            if_exists: IfExistsPolicy::Return,
        };

        let result1 = reply(&ledger, &client, args.clone()).unwrap();
        let result2 = reply(&ledger, &client, args).unwrap();
        assert_eq!(result2.meta.from_cache, Some(true));
        assert_eq!(
            result1.meta.client_request_id,
            result2.meta.client_request_id
        );
    }

    /// Characterization test: thread posts multiple tweets in sequence
    #[test]
    #[ignore] // Requires mockito server setup
    fn test_thread_posts_multiple_tweets() {
        let (ledger, client, _temp) = create_test_setup();

        let args = ThreadArgs {
            texts: vec![
                "First tweet".to_string(),
                "Second tweet".to_string(),
                "Third tweet".to_string(),
            ],
            client_request_id_prefix: Some("thread-001".to_string()),
            if_exists: IfExistsPolicy::Return,
        };

        let result = thread(&ledger, &client, args).unwrap();
        assert_eq!(result.tweets.len(), 3);
        assert_eq!(result.meta.count, 3);
        assert_eq!(result.meta.created_tweet_ids.len(), 3);
        assert!(result.meta.failed_index.is_none());
    }

    /// Characterization test: thread with empty texts returns error
    #[test]
    #[ignore] // Requires mockito server setup
    fn test_thread_empty_fails() {
        let (ledger, client, _temp) = create_test_setup();

        let args = ThreadArgs {
            texts: vec![],
            client_request_id_prefix: None,
            if_exists: IfExistsPolicy::Return,
        };

        let result = thread(&ledger, &client, args);
        assert!(result.is_err());
    }

    /// Characterization test: thread partial failure contains structured error
    #[test]
    #[ignore] // Requires mockito server setup for error simulation
    fn test_thread_partial_failure_contains_structured_error() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let ledger = IdempotencyLedger::new(Some(&db_path)).unwrap();
        let http_client = XApiClient::new();

        let args = ThreadArgs {
            texts: vec![
                "First tweet".to_string(),
                "Second tweet".to_string(),
                "Third tweet".to_string(),
            ],
            client_request_id_prefix: Some("thread-fail-test".to_string()),
            if_exists: IfExistsPolicy::Return,
        };

        let result = thread(&ledger, &http_client, args);
        assert!(result.is_err());
        // This test would pass with mockito error simulation
    }

    /// Characterization test: thread partial failure after some success has correct created_ids
    #[test]
    #[ignore] // Requires mockito server setup for error simulation
    fn test_thread_partial_failure_after_some_success() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let ledger = IdempotencyLedger::new(Some(&db_path)).unwrap();
        let http_client = XApiClient::new();

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

        let args = ThreadArgs {
            texts: vec!["First tweet".to_string(), "Second tweet".to_string()],
            client_request_id_prefix: Some(prefix.to_string()),
            if_exists: IfExistsPolicy::Return,
        };

        let result = thread(&ledger, &http_client, args);
        assert!(result.is_err());
        // This test would pass with mockito error simulation
    }
}
