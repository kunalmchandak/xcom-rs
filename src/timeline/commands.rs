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
            TimelineError::AuthRequired => {
                write!(f, "Authentication required.")
            }
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

/// Resolved user ID for a given handle
#[derive(Debug, Clone)]
struct ResolvedUser {
    id: String,
    handle: String,
}

/// Main timeline command handler
pub struct TimelineCommand;

impl TimelineCommand {
    /// Create a new timeline command handler
    pub fn new() -> Self {
        Self
    }

    /// Resolve a user by handle to get their user ID.
    /// In production this would call GET /2/users/by/username/{handle}.
    /// For testing, the resolved user ID can be overridden via XCOM_TEST_RESOLVE_USER_{HANDLE}_ID
    /// environment variable. If not set, a deterministic stub ID is generated.
    fn resolve_user_by_handle(&self, handle: &str) -> Result<ResolvedUser, TimelineError> {
        // Allow test overrides via environment variable for specific handles
        let env_key = format!("XCOM_TEST_RESOLVE_USER_{}_ID", handle.to_uppercase());
        let user_id = if let Ok(id) = std::env::var(&env_key) {
            id
        } else {
            // Generate a deterministic stub ID for the handle (production would fetch from API)
            format!("user_id_for_{}", handle.to_lowercase())
        };

        tracing::debug!(handle = %handle, user_id = %user_id, "Resolved user handle to ID");

        Ok(ResolvedUser {
            id: user_id,
            handle: handle.to_string(),
        })
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
        // Returning AuthRequired when no credentials are present
        // Check for XCOM_RS_BEARER_TOKEN using AuthStore
        let auth_store = crate::auth::storage::AuthStore::new();
        if !auth_store.is_authenticated() {
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
        // Step 1: Resolve handle to user ID
        // In production: GET /2/users/by/username/{handle}
        // For testing: uses XCOM_TEST_RESOLVE_USER_{HANDLE}_ID env var or deterministic stub
        let resolved = self.resolve_user_by_handle(&handle)?;

        tracing::info!(
            handle = %handle,
            user_id = %resolved.id,
            "Resolved handle to user ID, fetching tweets"
        );

        // Step 2: Fetch tweets for the resolved user ID
        // In production: GET /2/users/{id}/tweets
        let offset = Self::parse_cursor_offset(&args.cursor);
        let tweets = Self::build_stub_tweets(&resolved.handle, offset, args.limit);
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
        std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");
        std::env::set_var("XCOM_TEST_USER_ID", "test_user_id");
        std::env::set_var("XCOM_TEST_USER_HANDLE", "testhandle");
    }

    fn unset_authenticated() {
        std::env::remove_var("XCOM_RS_BEARER_TOKEN");
        std::env::remove_var("XCOM_TEST_USER_ID");
        std::env::remove_var("XCOM_TEST_USER_HANDLE");
        std::env::remove_var("XCOM_SIMULATE_ERROR");
        std::env::remove_var("XCOM_RETRY_AFTER_MS");
    }

    #[test]
    fn test_home_timeline_basic() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
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
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
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
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
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
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
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
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
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
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
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
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
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
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
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

    #[test]
    fn test_user_timeline_resolves_handle_to_id() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unset_authenticated();

        let cmd = TimelineCommand::new();
        // Verify handle resolution: tweets should use the resolved handle
        let args = TimelineArgs {
            kind: TimelineKind::User {
                handle: "XDev".to_string(),
            },
            limit: 5,
            cursor: None,
        };

        let result = cmd.get(args).unwrap();
        assert_eq!(result.tweets.len(), 5);
        // Tweets are built from the resolved handle (case preserved)
        // The resolve step maps handle -> user_id, then uses handle for stub tweets
        assert!(
            result.tweets.iter().all(|t| t.id.starts_with("XDev_")),
            "Expected tweet IDs to start with 'XDev_', got: {:?}",
            result.tweets.iter().map(|t| &t.id).collect::<Vec<_>>()
        );

        unset_authenticated();
    }

    #[test]
    fn test_user_timeline_resolves_handle_with_env_override() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unset_authenticated();
        // Override the resolved user ID for a specific handle
        std::env::set_var(
            "XCOM_TEST_RESOLVE_USER_TESTHANDLE_ID",
            "overridden_user_id_123",
        );

        let cmd = TimelineCommand::new();
        let args = TimelineArgs {
            kind: TimelineKind::User {
                handle: "testhandle".to_string(),
            },
            limit: 3,
            cursor: None,
        };

        let result = cmd.get(args).unwrap();
        assert_eq!(result.tweets.len(), 3);

        std::env::remove_var("XCOM_TEST_RESOLVE_USER_TESTHANDLE_ID");
        unset_authenticated();
    }

    #[test]
    fn test_pagination_response_uses_snake_case_tokens() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
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
        // Verify that pagination tokens are present (snake_case in JSON via serde field names)
        assert!(meta.pagination.next_token.is_some());
        assert_eq!(
            meta.pagination.next_token,
            Some("next_token_10".to_string())
        );

        // Serialize to JSON and verify field names are snake_case
        let json = serde_json::to_string(&meta).unwrap();
        assert!(
            json.contains("next_token"),
            "JSON should use next_token (snake_case), got: {}",
            json
        );
        assert!(
            !json.contains("nextToken"),
            "JSON should NOT use nextToken (camelCase), got: {}",
            json
        );

        unset_authenticated();
    }
}
