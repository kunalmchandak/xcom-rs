use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::tweets::models::Tweet;

/// Arguments for bookmark add/remove operations
#[derive(Debug, Clone)]
pub struct BookmarkArgs {
    pub tweet_id: String,
}

/// Arguments for listing bookmarks
#[derive(Debug, Clone)]
pub struct BookmarkListArgs {
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

/// Result of bookmark add/remove operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkResult {
    pub tweet_id: String,
    pub success: bool,
}

/// Pagination metadata for bookmark list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkPaginationMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// Metadata for bookmark list results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkListMeta {
    pub pagination: BookmarkPaginationMeta,
}

/// Result of listing bookmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkListResult {
    pub tweets: Vec<Tweet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<BookmarkListMeta>,
}

/// Bookmark command handler
pub struct BookmarkCommand;

impl BookmarkCommand {
    /// Create a new bookmark command handler
    pub fn new() -> Self {
        Self
    }

    /// Add a tweet to bookmarks
    /// In real implementation, calls POST /2/users/{id}/bookmarks
    pub fn add(&self, args: BookmarkArgs) -> Result<BookmarkResult> {
        Ok(BookmarkResult {
            tweet_id: args.tweet_id,
            success: true,
        })
    }

    /// Remove a tweet from bookmarks
    /// In real implementation, calls DELETE /2/users/{id}/bookmarks/{tweet_id}
    pub fn remove(&self, args: BookmarkArgs) -> Result<BookmarkResult> {
        Ok(BookmarkResult {
            tweet_id: args.tweet_id,
            success: true,
        })
    }

    /// List bookmarked tweets with pagination
    /// In real implementation, calls GET /2/users/{id}/bookmarks
    pub fn list(&self, args: BookmarkListArgs) -> Result<BookmarkListResult> {
        let limit = args.limit.unwrap_or(10);

        // Parse cursor to determine starting offset
        let offset = if let Some(cursor) = &args.cursor {
            // Cursor format is "bookmark_cursor_{offset}"
            cursor
                .strip_prefix("bookmark_cursor_")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0)
        } else {
            0
        };

        // Simulate fetching bookmarked tweets
        let mut tweets = Vec::new();
        for i in offset..(offset + limit) {
            let mut tweet = Tweet::new(format!("bookmark_tweet_{}", i));
            tweet.text = Some(format!("Bookmarked tweet text {}", i));
            tweet.author_id = Some(format!("user_{}", i));
            tweet.created_at = Some("2024-01-01T00:00:00Z".to_string());
            tweets.push(tweet);
        }

        // Create pagination metadata
        let next_token = if tweets.len() == limit {
            Some(format!("bookmark_cursor_{}", offset + limit))
        } else {
            None
        };

        let meta = Some(BookmarkListMeta {
            pagination: BookmarkPaginationMeta { next_token },
        });

        Ok(BookmarkListResult { tweets, meta })
    }
}

impl Default for BookmarkCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_add() {
        let cmd = BookmarkCommand::new();
        let args = BookmarkArgs {
            tweet_id: "tweet123".to_string(),
        };

        let result = cmd.add(args).unwrap();
        assert_eq!(result.tweet_id, "tweet123");
        assert!(result.success);
    }

    #[test]
    fn test_bookmark_remove() {
        let cmd = BookmarkCommand::new();
        let args = BookmarkArgs {
            tweet_id: "tweet456".to_string(),
        };

        let result = cmd.remove(args).unwrap();
        assert_eq!(result.tweet_id, "tweet456");
        assert!(result.success);
    }

    #[test]
    fn test_bookmark_list_default() {
        let cmd = BookmarkCommand::new();
        let args = BookmarkListArgs {
            limit: None,
            cursor: None,
        };

        let result = cmd.list(args).unwrap();
        assert_eq!(result.tweets.len(), 10);
        assert!(result.meta.is_some());
        let meta = result.meta.unwrap();
        assert!(meta.pagination.next_token.is_some());
        assert_eq!(
            meta.pagination.next_token,
            Some("bookmark_cursor_10".to_string())
        );
    }

    #[test]
    fn test_bookmark_list_with_limit() {
        let cmd = BookmarkCommand::new();
        let args = BookmarkListArgs {
            limit: Some(5),
            cursor: None,
        };

        let result = cmd.list(args).unwrap();
        assert_eq!(result.tweets.len(), 5);
        assert!(result.meta.is_some());
        let meta = result.meta.unwrap();
        assert_eq!(
            meta.pagination.next_token,
            Some("bookmark_cursor_5".to_string())
        );
    }

    #[test]
    fn test_bookmark_list_with_cursor() {
        let cmd = BookmarkCommand::new();
        let args = BookmarkListArgs {
            limit: Some(5),
            cursor: Some("bookmark_cursor_5".to_string()),
        };

        let result = cmd.list(args).unwrap();
        assert_eq!(result.tweets.len(), 5);
        // First tweet should start at offset 5
        assert_eq!(result.tweets[0].id, "bookmark_tweet_5");
    }
}
