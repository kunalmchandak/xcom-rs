//! Tweet engagement operations: like, unlike, retweet, unretweet.

use anyhow::Result;

use super::types::{EngagementArgs, EngagementResult};
use crate::tweets::http_client::XApiClient;

/// Like a tweet.
pub fn like(http_client: &XApiClient, args: EngagementArgs) -> Result<EngagementResult> {
    // Get user ID
    let user_id = http_client.get_user_id()?;

    // Like the tweet
    let success = http_client.like_tweet(&user_id, &args.tweet_id)?;

    Ok(EngagementResult {
        tweet_id: args.tweet_id,
        success,
    })
}

/// Unlike a tweet.
pub fn unlike(http_client: &XApiClient, args: EngagementArgs) -> Result<EngagementResult> {
    // Get user ID
    let user_id = http_client.get_user_id()?;

    // Unlike the tweet
    let success = http_client.unlike_tweet(&user_id, &args.tweet_id)?;

    Ok(EngagementResult {
        tweet_id: args.tweet_id,
        success,
    })
}

/// Retweet a tweet.
pub fn retweet(http_client: &XApiClient, args: EngagementArgs) -> Result<EngagementResult> {
    // Get user ID
    let user_id = http_client.get_user_id()?;

    // Retweet the tweet
    let success = http_client.retweet(&user_id, &args.tweet_id)?;

    Ok(EngagementResult {
        tweet_id: args.tweet_id,
        success,
    })
}

/// Unretweet a tweet.
pub fn unretweet(http_client: &XApiClient, args: EngagementArgs) -> Result<EngagementResult> {
    // Get user ID
    let user_id = http_client.get_user_id()?;

    // Unretweet the tweet
    let success = http_client.unretweet(&user_id, &args.tweet_id)?;

    Ok(EngagementResult {
        tweet_id: args.tweet_id,
        success,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Characterization test: like returns success with matching tweet_id
    #[test]
    #[ignore] // Requires mockito server setup
    fn test_like_tweet() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let http_client = XApiClient::new();
        let args = EngagementArgs {
            tweet_id: "tweet_123".to_string(),
        };
        let result = like(&http_client, args).unwrap();
        assert_eq!(result.tweet_id, "tweet_123");
        assert!(result.success);
    }

    /// Characterization test: unlike returns success with matching tweet_id
    #[test]
    #[ignore] // Requires mockito server setup
    fn test_unlike_tweet() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let http_client = XApiClient::new();
        let args = EngagementArgs {
            tweet_id: "tweet_456".to_string(),
        };
        let result = unlike(&http_client, args).unwrap();
        assert_eq!(result.tweet_id, "tweet_456");
        assert!(result.success);
    }

    /// Characterization test: retweet returns success with matching tweet_id
    #[test]
    #[ignore] // Requires mockito server setup
    fn test_retweet() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let http_client = XApiClient::new();
        let args = EngagementArgs {
            tweet_id: "tweet_789".to_string(),
        };
        let result = retweet(&http_client, args).unwrap();
        assert_eq!(result.tweet_id, "tweet_789");
        assert!(result.success);
    }

    /// Characterization test: unretweet returns success with matching tweet_id
    #[test]
    #[ignore] // Requires mockito server setup
    fn test_unretweet() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let http_client = XApiClient::new();
        let args = EngagementArgs {
            tweet_id: "tweet_101".to_string(),
        };
        let result = unretweet(&http_client, args).unwrap();
        assert_eq!(result.tweet_id, "tweet_101");
        assert!(result.success);
    }

    /// Characterization test: like with rate_limit simulation returns ClassifiedError 429
    #[test]
    #[ignore] // Now handled by mockito HTTP tests
    fn test_like_rate_limit_simulation() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::set_var("XCOM_SIMULATE_ERROR", "rate_limit");
        std::env::set_var("XCOM_RETRY_AFTER_MS", "5000");

        let http_client = XApiClient::new();
        let args = EngagementArgs {
            tweet_id: "tweet_123".to_string(),
        };
        let result = like(&http_client, args);
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        std::env::remove_var("XCOM_RETRY_AFTER_MS");

        assert!(result.is_err());
        // This test would pass with mockito error simulation
    }

    /// Characterization test: EngagementResult serializes correctly
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
