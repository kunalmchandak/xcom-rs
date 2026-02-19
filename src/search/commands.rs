use super::models::{
    SearchPaginationMeta, SearchRecentResult, SearchResultMeta, SearchTweet, SearchUser,
    SearchUsersResult,
};
use anyhow::Result;

/// Arguments for recent tweet search
#[derive(Debug, Clone)]
pub struct SearchRecentArgs {
    pub query: String,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

/// Arguments for user search
#[derive(Debug, Clone)]
pub struct SearchUsersArgs {
    pub query: String,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

/// Trait for X API search client (enables testing via mocking)
pub trait SearchClient {
    /// Search recent tweets
    fn search_recent(&self, args: &SearchRecentArgs) -> Result<SearchRecentResult>;
    /// Search users
    fn search_users(&self, args: &SearchUsersArgs) -> Result<SearchUsersResult>;
}

/// Mock implementation of SearchClient for testing
pub struct MockSearchClient {
    pub tweets: Vec<SearchTweet>,
    pub users: Vec<SearchUser>,
}

impl MockSearchClient {
    /// Create a new mock with empty data
    pub fn new() -> Self {
        Self {
            tweets: Vec::new(),
            users: Vec::new(),
        }
    }

    /// Create a mock with sample tweet fixtures
    pub fn with_tweet_fixtures(count: usize) -> Self {
        let tweets = (0..count)
            .map(|i| {
                let mut tweet = SearchTweet::new(format!("fixture_tweet_{}", i));
                tweet.text = Some(format!("Fixture tweet text {}", i));
                tweet.author_id = Some(format!("fixture_user_{}", i));
                tweet.created_at = Some("2024-01-01T00:00:00Z".to_string());
                tweet
            })
            .collect();
        Self {
            tweets,
            users: Vec::new(),
        }
    }

    /// Create a mock with sample user fixtures
    pub fn with_user_fixtures(count: usize) -> Self {
        let users = (0..count)
            .map(|i| {
                let mut user = SearchUser::new(format!("fixture_user_{}", i));
                user.name = Some(format!("Fixture User {}", i));
                user.username = Some(format!("fixture_user_{}", i));
                user.description = Some(format!("A fixture user {}", i));
                user
            })
            .collect();
        Self {
            tweets: Vec::new(),
            users,
        }
    }
}

impl Default for MockSearchClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchClient for MockSearchClient {
    fn search_recent(&self, args: &SearchRecentArgs) -> Result<SearchRecentResult> {
        let limit = args.limit.unwrap_or(10);
        let offset = parse_cursor(&args.cursor);
        let tweets: Vec<SearchTweet> = self
            .tweets
            .iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();
        let result_count = tweets.len();
        let next_cursor = if result_count == limit && offset + limit < self.tweets.len() {
            Some(format!("cursor_{}", offset + limit))
        } else {
            None
        };
        let prev_cursor = if offset > 0 {
            Some(format!("cursor_{}", offset.saturating_sub(limit)))
        } else {
            None
        };
        Ok(SearchRecentResult {
            tweets,
            meta: Some(SearchResultMeta {
                pagination: SearchPaginationMeta {
                    next_cursor,
                    prev_cursor,
                    result_count,
                },
            }),
        })
    }

    fn search_users(&self, args: &SearchUsersArgs) -> Result<SearchUsersResult> {
        let limit = args.limit.unwrap_or(10);
        let offset = parse_cursor(&args.cursor);
        let users: Vec<SearchUser> = self
            .users
            .iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();
        let result_count = users.len();
        let next_cursor = if result_count == limit && offset + limit < self.users.len() {
            Some(format!("cursor_{}", offset + limit))
        } else {
            None
        };
        let prev_cursor = if offset > 0 {
            Some(format!("cursor_{}", offset.saturating_sub(limit)))
        } else {
            None
        };
        Ok(SearchUsersResult {
            users,
            meta: Some(SearchResultMeta {
                pagination: SearchPaginationMeta {
                    next_cursor,
                    prev_cursor,
                    result_count,
                },
            }),
        })
    }
}

/// Search command handler
pub struct SearchCommand;

impl SearchCommand {
    pub fn new() -> Self {
        Self
    }

    /// Search recent tweets matching the query
    pub fn search_recent(&self, args: SearchRecentArgs) -> Result<SearchRecentResult> {
        // Check for simulated errors via environment variables (for testing)
        if let Ok(error_type) = std::env::var("XCOM_SIMULATE_ERROR") {
            if error_type == "rate_limit" {
                return Err(anyhow::anyhow!("Rate limit exceeded"));
            }
        }

        let limit = args.limit.unwrap_or(10);

        // Parse cursor to determine starting offset
        let offset = parse_cursor(&args.cursor);

        // Simulate fetching recent tweets matching query (in real implementation, would call X API)
        let tweets: Vec<SearchTweet> = (offset..(offset + limit))
            .map(|i| {
                let mut tweet = SearchTweet::new(format!("tweet_{}", i));
                tweet.text = Some(format!("{}: Tweet text {}", args.query, i));
                tweet.author_id = Some(format!("user_{}", i));
                tweet.created_at = Some("2024-01-01T00:00:00Z".to_string());
                tweet
            })
            .collect();

        let result_count = tweets.len();
        let next_cursor = if result_count == limit {
            Some(format!("cursor_{}", offset + limit))
        } else {
            None
        };
        let prev_cursor = if offset > 0 {
            Some(format!("cursor_{}", offset.saturating_sub(limit)))
        } else {
            None
        };

        let meta = Some(SearchResultMeta {
            pagination: SearchPaginationMeta {
                next_cursor,
                prev_cursor,
                result_count,
            },
        });

        Ok(SearchRecentResult { tweets, meta })
    }

    /// Search users matching the query
    pub fn search_users(&self, args: SearchUsersArgs) -> Result<SearchUsersResult> {
        // Check for simulated errors via environment variables (for testing)
        if let Ok(error_type) = std::env::var("XCOM_SIMULATE_ERROR") {
            if error_type == "rate_limit" {
                return Err(anyhow::anyhow!("Rate limit exceeded"));
            }
        }

        let limit = args.limit.unwrap_or(10);

        // Parse cursor to determine starting offset
        let offset = parse_cursor(&args.cursor);

        // Simulate fetching users matching query (in real implementation, would call X API)
        let users: Vec<SearchUser> = (offset..(offset + limit))
            .map(|i| {
                let mut user = SearchUser::new(format!("user_{}", i));
                user.name = Some(format!("{} User {}", args.query, i));
                user.username = Some(format!(
                    "{}_user_{}",
                    args.query.to_lowercase().replace(' ', "_"),
                    i
                ));
                user.description = Some(format!("A user matching '{}' query", args.query));
                user
            })
            .collect();

        let result_count = users.len();
        let next_cursor = if result_count == limit {
            Some(format!("cursor_{}", offset + limit))
        } else {
            None
        };
        let prev_cursor = if offset > 0 {
            Some(format!("cursor_{}", offset.saturating_sub(limit)))
        } else {
            None
        };

        let meta = Some(SearchResultMeta {
            pagination: SearchPaginationMeta {
                next_cursor,
                prev_cursor,
                result_count,
            },
        });

        Ok(SearchUsersResult { users, meta })
    }
}

impl Default for SearchCommand {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse cursor token into an offset value
fn parse_cursor(cursor: &Option<String>) -> usize {
    if let Some(cursor) = cursor {
        cursor
            .strip_prefix("cursor_")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_recent_basic() {
        let cmd = SearchCommand::new();
        let args = SearchRecentArgs {
            query: "hello world".to_string(),
            limit: Some(5),
            cursor: None,
        };

        let result = cmd.search_recent(args).unwrap();
        assert_eq!(result.tweets.len(), 5);
        assert!(result.meta.is_some());
        let meta = result.meta.unwrap();
        assert_eq!(meta.pagination.result_count, 5);
        assert_eq!(meta.pagination.next_cursor, Some("cursor_5".to_string()));
        assert!(meta.pagination.prev_cursor.is_none());
    }

    #[test]
    fn test_search_recent_with_cursor() {
        let cmd = SearchCommand::new();
        let args = SearchRecentArgs {
            query: "rust".to_string(),
            limit: Some(5),
            cursor: Some("cursor_10".to_string()),
        };

        let result = cmd.search_recent(args).unwrap();
        assert_eq!(result.tweets.len(), 5);
        let meta = result.meta.unwrap();
        assert_eq!(meta.pagination.next_cursor, Some("cursor_15".to_string()));
        assert_eq!(meta.pagination.prev_cursor, Some("cursor_5".to_string()));
    }

    #[test]
    fn test_search_recent_tweet_contains_query() {
        let cmd = SearchCommand::new();
        let args = SearchRecentArgs {
            query: "rustlang".to_string(),
            limit: Some(3),
            cursor: None,
        };

        let result = cmd.search_recent(args).unwrap();
        for tweet in &result.tweets {
            let text = tweet.text.as_ref().unwrap();
            assert!(
                text.contains("rustlang"),
                "Tweet text should contain query: {}",
                text
            );
        }
    }

    #[test]
    fn test_search_recent_default_limit() {
        let cmd = SearchCommand::new();
        let args = SearchRecentArgs {
            query: "test".to_string(),
            limit: None,
            cursor: None,
        };

        let result = cmd.search_recent(args).unwrap();
        assert_eq!(result.tweets.len(), 10); // default limit
    }

    #[test]
    fn test_search_users_basic() {
        let cmd = SearchCommand::new();
        let args = SearchUsersArgs {
            query: "alice".to_string(),
            limit: Some(5),
            cursor: None,
        };

        let result = cmd.search_users(args).unwrap();
        assert_eq!(result.users.len(), 5);
        assert!(result.meta.is_some());
        let meta = result.meta.unwrap();
        assert_eq!(meta.pagination.result_count, 5);
        assert_eq!(meta.pagination.next_cursor, Some("cursor_5".to_string()));
    }

    #[test]
    fn test_search_users_with_cursor() {
        let cmd = SearchCommand::new();
        let args = SearchUsersArgs {
            query: "bob".to_string(),
            limit: Some(5),
            cursor: Some("cursor_5".to_string()),
        };

        let result = cmd.search_users(args).unwrap();
        assert_eq!(result.users.len(), 5);
        let meta = result.meta.unwrap();
        assert_eq!(meta.pagination.next_cursor, Some("cursor_10".to_string()));
        assert_eq!(meta.pagination.prev_cursor, Some("cursor_0".to_string()));
    }

    #[test]
    fn test_search_users_user_fields() {
        let cmd = SearchCommand::new();
        let args = SearchUsersArgs {
            query: "developer".to_string(),
            limit: Some(3),
            cursor: None,
        };

        let result = cmd.search_users(args).unwrap();
        for user in &result.users {
            assert!(!user.id.is_empty());
            assert!(user.name.is_some());
            assert!(user.username.is_some());
            assert!(user.description.is_some());
        }
    }

    #[test]
    fn test_search_users_default_limit() {
        let cmd = SearchCommand::new();
        let args = SearchUsersArgs {
            query: "test".to_string(),
            limit: None,
            cursor: None,
        };

        let result = cmd.search_users(args).unwrap();
        assert_eq!(result.users.len(), 10); // default limit
    }

    #[test]
    fn test_parse_cursor_none() {
        assert_eq!(parse_cursor(&None), 0);
    }

    #[test]
    fn test_parse_cursor_valid() {
        assert_eq!(parse_cursor(&Some("cursor_42".to_string())), 42);
    }

    #[test]
    fn test_parse_cursor_invalid() {
        assert_eq!(parse_cursor(&Some("invalid".to_string())), 0);
    }
}
