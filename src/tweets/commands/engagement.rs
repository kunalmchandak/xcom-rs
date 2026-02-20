//! Tweet engagement operations: like, unlike, retweet, unretweet.

use anyhow::Result;

use super::types::{ClassifiedError, EngagementArgs, EngagementResult};

fn simulate_rate_limit_error() -> Result<EngagementResult> {
    let retry_after = std::env::var("XCOM_RETRY_AFTER_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(60000);
    Err(
        ClassifiedError::from_status_code(429, "Rate limit exceeded".to_string())
            .with_retry_after(retry_after)
            .into(),
    )
}

/// Like a tweet.
pub fn like(args: EngagementArgs) -> Result<EngagementResult> {
    if let Ok(error_type) = std::env::var("XCOM_SIMULATE_ERROR") {
        match error_type.as_str() {
            "rate_limit" => return simulate_rate_limit_error(),
            "auth_error" => {
                return Err(ClassifiedError::from_status_code(
                    403,
                    "Insufficient permissions to like tweet".to_string(),
                )
                .into())
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

/// Unlike a tweet.
pub fn unlike(args: EngagementArgs) -> Result<EngagementResult> {
    if let Ok(error_type) = std::env::var("XCOM_SIMULATE_ERROR") {
        if error_type.as_str() == "rate_limit" {
            return simulate_rate_limit_error();
        }
    }

    // Simulate unlike operation (in real implementation, would call DELETE /2/users/{id}/likes/{tweet_id})
    Ok(EngagementResult {
        tweet_id: args.tweet_id,
        success: true,
    })
}

/// Retweet a tweet.
pub fn retweet(args: EngagementArgs) -> Result<EngagementResult> {
    if let Ok(error_type) = std::env::var("XCOM_SIMULATE_ERROR") {
        match error_type.as_str() {
            "rate_limit" => return simulate_rate_limit_error(),
            "auth_error" => {
                return Err(ClassifiedError::from_status_code(
                    403,
                    "Insufficient permissions to retweet".to_string(),
                )
                .into())
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

/// Unretweet a tweet.
pub fn unretweet(args: EngagementArgs) -> Result<EngagementResult> {
    if let Ok(error_type) = std::env::var("XCOM_SIMULATE_ERROR") {
        if error_type.as_str() == "rate_limit" {
            return simulate_rate_limit_error();
        }
    }

    // Simulate unretweet operation (in real implementation, would call DELETE /2/users/{id}/retweets/{source_tweet_id})
    Ok(EngagementResult {
        tweet_id: args.tweet_id,
        success: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Characterization test: like returns success with matching tweet_id
    #[test]
    fn test_like_tweet() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let args = EngagementArgs {
            tweet_id: "tweet_123".to_string(),
        };
        let result = like(args).unwrap();
        assert_eq!(result.tweet_id, "tweet_123");
        assert!(result.success);
    }

    /// Characterization test: unlike returns success with matching tweet_id
    #[test]
    fn test_unlike_tweet() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let args = EngagementArgs {
            tweet_id: "tweet_456".to_string(),
        };
        let result = unlike(args).unwrap();
        assert_eq!(result.tweet_id, "tweet_456");
        assert!(result.success);
    }

    /// Characterization test: retweet returns success with matching tweet_id
    #[test]
    fn test_retweet() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let args = EngagementArgs {
            tweet_id: "tweet_789".to_string(),
        };
        let result = retweet(args).unwrap();
        assert_eq!(result.tweet_id, "tweet_789");
        assert!(result.success);
    }

    /// Characterization test: unretweet returns success with matching tweet_id
    #[test]
    fn test_unretweet() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_SIMULATE_ERROR");

        let args = EngagementArgs {
            tweet_id: "tweet_101".to_string(),
        };
        let result = unretweet(args).unwrap();
        assert_eq!(result.tweet_id, "tweet_101");
        assert!(result.success);
    }

    /// Characterization test: like with rate_limit simulation returns ClassifiedError 429
    #[test]
    fn test_like_rate_limit_simulation() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::set_var("XCOM_SIMULATE_ERROR", "rate_limit");
        std::env::set_var("XCOM_RETRY_AFTER_MS", "5000");

        let args = EngagementArgs {
            tweet_id: "tweet_123".to_string(),
        };
        let result = like(args);
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        std::env::remove_var("XCOM_RETRY_AFTER_MS");

        assert!(result.is_err());
        let err = result.unwrap_err();
        let classified = err.downcast_ref::<ClassifiedError>().unwrap();
        assert_eq!(classified.status_code, Some(429));
        assert!(classified.is_retryable);
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
