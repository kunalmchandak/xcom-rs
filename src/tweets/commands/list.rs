//! Tweet listing with field projection and pagination.

use anyhow::Result;

use crate::tweets::client::{HttpTweetApiClient, TweetApiClient};

use super::types::{ListArgs, ListResult};

/// List tweets with field projection and pagination using the provided client.
pub fn list_with_client(client: &dyn TweetApiClient, args: ListArgs) -> Result<ListResult> {
    client.list_tweets(&args)
}

/// List tweets with field projection and pagination.
pub fn list(args: ListArgs) -> Result<ListResult> {
    let client = HttpTweetApiClient::from_env()?;
    list_with_client(&client, args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweets::client::MockTweetApiClient;
    use crate::tweets::models::TweetFields;

    fn set_authenticated() {
        std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");
        std::env::set_var("XCOM_TEST_USER_ID", "test_user_id");
    }

    fn unset_authenticated() {
        std::env::remove_var("XCOM_RS_BEARER_TOKEN");
        std::env::remove_var("XCOM_TEST_USER_ID");
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        std::env::remove_var("XCOM_RETRY_AFTER_MS");
    }

    /// Characterization test: list with field projection returns only requested fields
    #[test]
    fn test_list_with_field_projection() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unset_authenticated();

        let client = MockTweetApiClient::new();
        let args = ListArgs {
            fields: vec![TweetFields::Id, TweetFields::Text],
            limit: Some(5),
            cursor: None,
        };

        let result = client.list_tweets(&args).unwrap();
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
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unset_authenticated();

        let client = MockTweetApiClient::new();
        let args = ListArgs {
            fields: TweetFields::default_fields(),
            limit: Some(10),
            cursor: None,
        };

        let result = client.list_tweets(&args).unwrap();
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
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unset_authenticated();

        let client = MockTweetApiClient::new();
        let args = ListArgs {
            fields: TweetFields::default_fields(),
            limit: Some(5),
            cursor: Some("cursor_10".to_string()),
        };

        let result = client.list_tweets(&args).unwrap();
        assert_eq!(result.tweets.len(), 5);
        // First tweet should start at offset 10
        assert_eq!(result.tweets[0].id, "tweet_10");
        let meta = result.meta.unwrap();
        assert_eq!(meta.pagination.prev_cursor, Some("cursor_5".to_string()));
        assert_eq!(meta.pagination.next_cursor, Some("cursor_15".to_string()));
    }
}
