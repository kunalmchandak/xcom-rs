//! Tweet listing with field projection and pagination.

use anyhow::Result;

use crate::tweets::models::Tweet;

use super::types::{ClassifiedError, ListArgs, ListResult, ListResultMeta, PaginationMeta};

/// List tweets with field projection and pagination.
pub fn list(args: ListArgs) -> Result<ListResult> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweets::models::TweetFields;

    /// Characterization test: list with field projection returns only requested fields
    #[test]
    fn test_list_with_field_projection() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        std::env::remove_var("XCOM_RETRY_AFTER_MS");

        let args = ListArgs {
            fields: vec![TweetFields::Id, TweetFields::Text],
            limit: Some(5),
            cursor: None,
        };

        let result = list(args).unwrap();
        assert_eq!(result.tweets.len(), 5);

        for tweet in &result.tweets {
            assert!(!tweet.id.is_empty());
            assert!(tweet.text.is_some());
            assert!(tweet.author_id.is_none()); // Not requested
        }
    }

    /// Characterization test: list returns pagination metadata with next_cursor
    #[test]
    fn test_list_pagination() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        std::env::remove_var("XCOM_RETRY_AFTER_MS");

        let args = ListArgs {
            fields: TweetFields::default_fields(),
            limit: Some(10),
            cursor: None,
        };

        let result = list(args).unwrap();
        assert_eq!(result.tweets.len(), 10);
        assert!(result.meta.is_some());
        let meta = result.meta.unwrap();
        assert!(meta.pagination.next_cursor.is_some());
        assert_eq!(meta.pagination.next_cursor, Some("cursor_10".to_string()));
        assert!(meta.pagination.prev_cursor.is_none());
    }

    /// Characterization test: pagination with cursor returns offset tweets
    #[test]
    fn test_list_pagination_with_cursor() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        std::env::remove_var("XCOM_RETRY_AFTER_MS");

        let args = ListArgs {
            fields: TweetFields::default_fields(),
            limit: Some(5),
            cursor: Some("cursor_10".to_string()),
        };

        let result = list(args).unwrap();
        assert_eq!(result.tweets.len(), 5);
        // First tweet should start at offset 10
        assert_eq!(result.tweets[0].id, "tweet_10");
        let meta = result.meta.unwrap();
        assert_eq!(meta.pagination.prev_cursor, Some("cursor_5".to_string()));
        assert_eq!(meta.pagination.next_cursor, Some("cursor_15".to_string()));
    }
}
