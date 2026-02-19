use anyhow::Result;

use super::models::{TimelineArgs, TimelineKind, TimelineMeta, TimelinePagination, TimelineResult};
use crate::tweets::Tweet;

/// Error type for timeline operations
#[derive(Debug)]
pub enum TimelineError {
    /// Authentication is required but not available
    AuthRequired,
    /// API call failed with a classified error
    ApiError(crate::tweets::ClassifiedError),
}

impl std::fmt::Display for TimelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimelineError::AuthRequired => write!(
                f,
                "Authentication required. Run 'xcom-rs auth login' to authenticate."
            ),
            TimelineError::ApiError(e) => write!(f, "API error: {}", e),
        }
    }
}

impl std::error::Error for TimelineError {}

impl TimelineError {
    /// Convert to ErrorCode for protocol
    pub fn to_error_code(&self) -> crate::protocol::ErrorCode {
        use crate::protocol::ErrorCode;
        match self {
            TimelineError::AuthRequired => ErrorCode::AuthRequired,
            TimelineError::ApiError(e) => e.to_error_code(),
        }
    }
}

/// Simulated user info representing an authenticated user
struct UserInfo {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    handle: String,
}

/// Main timeline command handler
pub struct TimelineCommand;

impl TimelineCommand {
    /// Create a new timeline command handler
    pub fn new() -> Self {
        Self
    }

    /// Resolve the authenticated user's ID.
    /// In a real implementation this would call GET /2/users/me.
    /// For now, we simulate it via an environment variable or use a stub.
    fn resolve_me(&self) -> Result<UserInfo, TimelineError> {
        // Allow overriding via environment for testing
        if let Ok(user_id) = std::env::var("XCOM_TEST_USER_ID") {
            let handle =
                std::env::var("XCOM_TEST_USER_HANDLE").unwrap_or_else(|_| "testuser".to_string());
            return Ok(UserInfo {
                id: user_id,
                handle,
            });
        }

        // In production this would call the X API; here we simulate a default authenticated user
        // Returning AuthRequired when no credentials are present (simulated by env var absence)
        if std::env::var("XCOM_AUTHENTICATED").is_err() {
            return Err(TimelineError::AuthRequired);
        }

        Ok(UserInfo {
            id: "me_user_id".to_string(),
            handle: "me".to_string(),
        })
    }

    /// Retrieve a timeline based on the given arguments.
    pub fn get(&self, args: TimelineArgs) -> Result<TimelineResult, TimelineError> {
        // Check for simulated errors via environment variables (for testing)
        if let Ok(error_type) = std::env::var("XCOM_SIMULATE_ERROR") {
            use crate::tweets::ClassifiedError;
            match error_type.as_str() {
                "rate_limit" => {
                    let retry_after = std::env::var("XCOM_RETRY_AFTER_MS")
                        .ok()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(60000);
                    return Err(TimelineError::ApiError(
                        ClassifiedError::from_status_code(429, "Rate limit exceeded".to_string())
                            .with_retry_after(retry_after),
                    ));
                }
                "server_error" => {
                    return Err(TimelineError::ApiError(ClassifiedError::from_status_code(
                        500,
                        "Internal server error".to_string(),
                    )));
                }
                "auth_required" => {
                    return Err(TimelineError::AuthRequired);
                }
                _ => {}
            }
        }

        match &args.kind {
            TimelineKind::Home => self.get_home(&args),
            TimelineKind::Mentions => self.get_mentions(&args),
            TimelineKind::User { handle } => self.get_user_tweets(handle.clone(), &args),
        }
    }

    /// Build simulated tweets for testing/stub purposes.
    fn build_stub_tweets(prefix: &str, offset: usize, limit: usize) -> Vec<Tweet> {
        (offset..(offset + limit))
            .map(|i| {
                let mut tweet = Tweet::new(format!("{}_{}", prefix, i));
                tweet.text = Some(format!("{} tweet text {}", prefix, i));
                tweet.author_id = Some(format!("user_{}", i));
                tweet.created_at = Some("2024-01-01T00:00:00Z".to_string());
                tweet
            })
            .collect()
    }

    /// Parse cursor to extract offset.
    fn parse_cursor_offset(cursor: &Option<String>) -> usize {
        if let Some(c) = cursor {
            c.strip_prefix("next_token_")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0)
        } else {
            0
        }
    }

    /// Build pagination metadata.
    fn build_pagination(offset: usize, limit: usize, count: usize) -> Option<TimelineMeta> {
        let next_token = if count == limit {
            Some(format!("next_token_{}", offset + limit))
        } else {
            None
        };

        let previous_token = if offset > 0 {
            Some(format!("next_token_{}", offset.saturating_sub(limit)))
        } else {
            None
        };

        if next_token.is_some() || previous_token.is_some() {
            Some(TimelineMeta {
                pagination: TimelinePagination {
                    next_token,
                    previous_token,
                },
            })
        } else {
            None
        }
    }

    fn get_home(&self, args: &TimelineArgs) -> Result<TimelineResult, TimelineError> {
        // Resolve authenticated user (would call GET /2/users/me in production)
        let _user = self.resolve_me()?;

        let offset = Self::parse_cursor_offset(&args.cursor);
        let tweets = Self::build_stub_tweets("home", offset, args.limit);
        let count = tweets.len();
        let meta = Self::build_pagination(offset, args.limit, count);

        Ok(TimelineResult { tweets, meta })
    }

    fn get_mentions(&self, args: &TimelineArgs) -> Result<TimelineResult, TimelineError> {
        // Resolve authenticated user (would call GET /2/users/me in production)
        let _user = self.resolve_me()?;

        let offset = Self::parse_cursor_offset(&args.cursor);
        let tweets = Self::build_stub_tweets("mention", offset, args.limit);
        let count = tweets.len();
        let meta = Self::build_pagination(offset, args.limit, count);

        Ok(TimelineResult { tweets, meta })
    }

    fn get_user_tweets(
        &self,
        handle: String,
        args: &TimelineArgs,
    ) -> Result<TimelineResult, TimelineError> {
        // For user timeline, no auth is strictly required for public accounts,
        // but we still need to resolve the user by handle.
        // In production: GET /2/users/by/username/{handle} then GET /2/users/{id}/tweets
        let offset = Self::parse_cursor_offset(&args.cursor);
        let tweets = Self::build_stub_tweets(&handle, offset, args.limit);
        let count = tweets.len();
        let meta = Self::build_pagination(offset, args.limit, count);

        Ok(TimelineResult { tweets, meta })
    }
}

impl Default for TimelineCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::models::TimelineKind;

    fn set_authenticated() {
        std::env::set_var("XCOM_AUTHENTICATED", "1");
        std::env::set_var("XCOM_TEST_USER_ID", "test_user_id");
        std::env::set_var("XCOM_TEST_USER_HANDLE", "testhandle");
    }

    fn unset_authenticated() {
        std::env::remove_var("XCOM_AUTHENTICATED");
        std::env::remove_var("XCOM_TEST_USER_ID");
        std::env::remove_var("XCOM_TEST_USER_HANDLE");
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        std::env::remove_var("XCOM_RETRY_AFTER_MS");
    }

    #[test]
    fn test_home_timeline_basic() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        set_authenticated();

        let cmd = TimelineCommand::new();
        let args = TimelineArgs {
            kind: TimelineKind::Home,
            limit: 5,
            cursor: None,
        };

        let result = cmd.get(args).unwrap();
        assert_eq!(result.tweets.len(), 5);
        assert!(result.tweets.iter().all(|t| t.id.starts_with("home_")));

        unset_authenticated();
    }

    #[test]
    fn test_mentions_timeline_basic() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        set_authenticated();

        let cmd = TimelineCommand::new();
        let args = TimelineArgs {
            kind: TimelineKind::Mentions,
            limit: 3,
            cursor: None,
        };

        let result = cmd.get(args).unwrap();
        assert_eq!(result.tweets.len(), 3);
        assert!(result.tweets.iter().all(|t| t.id.starts_with("mention_")));

        unset_authenticated();
    }

    #[test]
    fn test_user_timeline_basic() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        unset_authenticated();

        let cmd = TimelineCommand::new();
        let args = TimelineArgs {
            kind: TimelineKind::User {
                handle: "johndoe".to_string(),
            },
            limit: 4,
            cursor: None,
        };

        let result = cmd.get(args).unwrap();
        assert_eq!(result.tweets.len(), 4);
        assert!(result.tweets.iter().all(|t| t.id.starts_with("johndoe_")));

        unset_authenticated();
    }

    #[test]
    fn test_home_timeline_with_cursor() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        set_authenticated();

        let cmd = TimelineCommand::new();
        let args = TimelineArgs {
            kind: TimelineKind::Home,
            limit: 5,
            cursor: Some("next_token_5".to_string()),
        };

        let result = cmd.get(args).unwrap();
        assert_eq!(result.tweets.len(), 5);
        // Tweets should start from offset 5
        assert_eq!(result.tweets[0].id, "home_5");

        unset_authenticated();
    }

    #[test]
    fn test_timeline_pagination_next_token() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        set_authenticated();

        let cmd = TimelineCommand::new();
        let args = TimelineArgs {
            kind: TimelineKind::Home,
            limit: 10,
            cursor: None,
        };

        let result = cmd.get(args).unwrap();
        assert!(result.meta.is_some());
        let meta = result.meta.unwrap();
        assert_eq!(
            meta.pagination.next_token,
            Some("next_token_10".to_string())
        );
        assert!(meta.pagination.previous_token.is_none());

        unset_authenticated();
    }

    #[test]
    fn test_timeline_auth_required() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        unset_authenticated();

        let cmd = TimelineCommand::new();
        let args = TimelineArgs {
            kind: TimelineKind::Home,
            limit: 10,
            cursor: None,
        };

        let result = cmd.get(args);
        assert!(result.is_err());
        match result.unwrap_err() {
            TimelineError::AuthRequired => {}
            e => panic!("Expected AuthRequired, got: {}", e),
        }
    }

    #[test]
    fn test_timeline_rate_limit_error() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        set_authenticated();
        std::env::set_var("XCOM_SIMULATE_ERROR", "rate_limit");
        std::env::set_var("XCOM_RETRY_AFTER_MS", "5000");

        let cmd = TimelineCommand::new();
        let args = TimelineArgs {
            kind: TimelineKind::Home,
            limit: 10,
            cursor: None,
        };

        let result = cmd.get(args);
        assert!(result.is_err());
        match result.unwrap_err() {
            TimelineError::ApiError(e) => {
                assert!(e.is_retryable);
                assert_eq!(e.retry_after_ms, Some(5000));
            }
            e => panic!("Expected ApiError, got: {}", e),
        }

        unset_authenticated();
    }

    #[test]
    fn test_timeline_pagination_with_previous_token() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();
        set_authenticated();

        let cmd = TimelineCommand::new();
        let args = TimelineArgs {
            kind: TimelineKind::Mentions,
            limit: 5,
            cursor: Some("next_token_10".to_string()),
        };

        let result = cmd.get(args).unwrap();
        assert!(result.meta.is_some());
        let meta = result.meta.unwrap();
        // Should have both next and previous tokens when in the middle
        assert!(meta.pagination.next_token.is_some());
        assert!(meta.pagination.previous_token.is_some());
        assert_eq!(
            meta.pagination.previous_token,
            Some("next_token_5".to_string())
        );

        unset_authenticated();
    }
}
