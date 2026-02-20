use anyhow::{Context, Result};
use serde::Deserialize;

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

/// X API v2 response for timeline endpoints
#[derive(Debug, Deserialize)]
struct ApiTimelineResponse {
    data: Option<Vec<ApiTweet>>,
    meta: Option<ApiMeta>,
}

/// X API v2 tweet data
#[derive(Debug, Deserialize)]
struct ApiTweet {
    id: String,
    text: Option<String>,
    author_id: Option<String>,
    created_at: Option<String>,
}

/// X API v2 metadata
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ApiMeta {
    result_count: Option<usize>,
    next_token: Option<String>,
    previous_token: Option<String>,
}

/// X API v2 user lookup response
#[derive(Debug, Deserialize)]
struct ApiUserLookupResponse {
    data: Option<ApiUserData>,
}

#[derive(Debug, Deserialize)]
struct ApiUserData {
    id: String,
    username: String,
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
#[allow(dead_code)]
struct ResolvedUser {
    id: String,
    handle: String,
}

/// Trait for timeline operations (enables testing via mocking)
pub trait TimelineClient {
    /// Get timeline based on arguments
    fn get(&self, args: &TimelineArgs) -> Result<TimelineResult, TimelineError>;
}

/// HTTP-based implementation of TimelineClient using X API
pub struct HttpTimelineClient {
    bearer_token: String,
}

impl HttpTimelineClient {
    /// Create a new HTTP timeline client with the given bearer token
    pub fn new(bearer_token: String) -> Self {
        Self { bearer_token }
    }

    /// Create from environment variable (XCOM_RS_BEARER_TOKEN)
    pub fn from_env() -> Result<Self, TimelineError> {
        let auth_store = crate::auth::storage::AuthStore::new();
        let status = auth_store.status();
        if !status.authenticated {
            return Err(TimelineError::AuthRequired);
        }
        let bearer_token =
            std::env::var("XCOM_RS_BEARER_TOKEN").map_err(|_| TimelineError::AuthRequired)?;
        Ok(Self::new(bearer_token))
    }

    /// Resolve user by handle to get their user ID
    fn resolve_user_by_handle(&self, handle: &str) -> Result<ResolvedUser, TimelineError> {
        // Allow test overrides via environment variable
        let env_key = format!("XCOM_TEST_RESOLVE_USER_{}_ID", handle.to_uppercase());
        if let Ok(id) = std::env::var(&env_key) {
            return Ok(ResolvedUser {
                id,
                handle: handle.to_string(),
            });
        }

        // Call X API to resolve handle
        let url = format!(
            "https://api.twitter.com/2/users/by/username/{}",
            urlencoding::encode(handle)
        );

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.bearer_token))
            .call();

        let api_response: ApiUserLookupResponse = match response {
            Ok(resp) => resp
                .into_json()
                .context("Failed to parse user lookup response")
                .map_err(|e| {
                    TimelineError::ApiError(crate::tweets::ClassifiedError::from_status_code(
                        500,
                        e.to_string(),
                    ))
                })?,
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                return Err(TimelineError::ApiError(
                    crate::tweets::ClassifiedError::from_status_code(code, body),
                ));
            }
            Err(e) => {
                return Err(TimelineError::ApiError(
                    crate::tweets::ClassifiedError::from_status_code(
                        500,
                        format!("Failed to resolve user: {}", e),
                    ),
                ));
            }
        };

        let user_data = api_response.data.ok_or_else(|| {
            TimelineError::ApiError(crate::tweets::ClassifiedError::from_status_code(
                404,
                format!("User not found: {}", handle),
            ))
        })?;

        Ok(ResolvedUser {
            id: user_data.id,
            handle: user_data.username,
        })
    }

    /// Resolve authenticated user ID
    fn resolve_me(&self) -> Result<UserInfo, TimelineError> {
        // Allow test override
        if let Ok(user_id) = std::env::var("XCOM_TEST_USER_ID") {
            let handle =
                std::env::var("XCOM_TEST_USER_HANDLE").unwrap_or_else(|_| "testuser".to_string());
            return Ok(UserInfo {
                id: user_id,
                handle,
            });
        }

        // Call X API to get authenticated user
        let url = "https://api.twitter.com/2/users/me";

        let response = ureq::get(url)
            .set("Authorization", &format!("Bearer {}", self.bearer_token))
            .call();

        let api_response: ApiUserLookupResponse = match response {
            Ok(resp) => resp
                .into_json()
                .context("Failed to parse user/me response")
                .map_err(|e| {
                    TimelineError::ApiError(crate::tweets::ClassifiedError::from_status_code(
                        500,
                        e.to_string(),
                    ))
                })?,
            Err(ureq::Error::Status(code, resp)) if code == 401 || code == 403 => {
                // Classify 403 errors to detect UAT requirement
                if code == 403 {
                    let error_details = crate::x_api::classify_response_error(resp);
                    if error_details.code == crate::protocol::ErrorCode::AuthRequired {
                        return Err(TimelineError::AuthRequired);
                    }
                }
                return Err(TimelineError::AuthRequired);
            }
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                return Err(TimelineError::ApiError(
                    crate::tweets::ClassifiedError::from_status_code(code, body),
                ));
            }
            Err(e) => {
                return Err(TimelineError::ApiError(
                    crate::tweets::ClassifiedError::from_status_code(
                        500,
                        format!("Failed to get user/me: {}", e),
                    ),
                ));
            }
        };

        let user_data = api_response.data.ok_or(TimelineError::AuthRequired)?;

        Ok(UserInfo {
            id: user_data.id,
            handle: user_data.username,
        })
    }

    /// Fetch timeline from API endpoint
    fn fetch_timeline(&self, url: &str) -> Result<TimelineResult, TimelineError> {
        let response = ureq::get(url)
            .set("Authorization", &format!("Bearer {}", self.bearer_token))
            .call();

        let api_response: ApiTimelineResponse = match response {
            Ok(resp) => resp
                .into_json()
                .context("Failed to parse timeline response")
                .map_err(|e| {
                    TimelineError::ApiError(crate::tweets::ClassifiedError::from_status_code(
                        500,
                        e.to_string(),
                    ))
                })?,
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                return Err(TimelineError::ApiError(
                    crate::tweets::ClassifiedError::from_status_code(code, body),
                ));
            }
            Err(e) => {
                return Err(TimelineError::ApiError(
                    crate::tweets::ClassifiedError::from_status_code(
                        500,
                        format!("Failed to fetch timeline: {}", e),
                    ),
                ));
            }
        };

        let tweets = api_response
            .data
            .unwrap_or_default()
            .into_iter()
            .map(|t| {
                let mut tweet = Tweet::new(t.id);
                tweet.text = t.text;
                tweet.author_id = t.author_id;
                tweet.created_at = t.created_at;
                tweet
            })
            .collect();

        let meta = if let Some(api_meta) = api_response.meta {
            if api_meta.next_token.is_some() || api_meta.previous_token.is_some() {
                Some(TimelineMeta {
                    pagination: TimelinePagination {
                        next_token: api_meta.next_token,
                        previous_token: api_meta.previous_token,
                    },
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok(TimelineResult { tweets, meta })
    }
}

impl TimelineClient for HttpTimelineClient {
    fn get(&self, args: &TimelineArgs) -> Result<TimelineResult, TimelineError> {
        match &args.kind {
            TimelineKind::Home => {
                let user = self.resolve_me()?;
                let mut url = format!(
                    "https://api.twitter.com/2/users/{}/timelines/reverse_chronological?max_results={}",
                    user.id, args.limit
                );
                url.push_str("&tweet.fields=id,text,author_id,created_at");
                if let Some(cursor) = &args.cursor {
                    url.push_str(&format!(
                        "&pagination_token={}",
                        urlencoding::encode(cursor)
                    ));
                }
                self.fetch_timeline(&url)
            }
            TimelineKind::Mentions => {
                let user = self.resolve_me()?;
                let mut url = format!(
                    "https://api.twitter.com/2/users/{}/mentions?max_results={}",
                    user.id, args.limit
                );
                url.push_str("&tweet.fields=id,text,author_id,created_at");
                if let Some(cursor) = &args.cursor {
                    url.push_str(&format!(
                        "&pagination_token={}",
                        urlencoding::encode(cursor)
                    ));
                }
                self.fetch_timeline(&url)
            }
            TimelineKind::User { handle } => {
                let resolved = self.resolve_user_by_handle(handle)?;
                let mut url = format!(
                    "https://api.twitter.com/2/users/{}/tweets?max_results={}",
                    resolved.id, args.limit
                );
                url.push_str("&tweet.fields=id,text,author_id,created_at");
                if let Some(cursor) = &args.cursor {
                    url.push_str(&format!(
                        "&pagination_token={}",
                        urlencoding::encode(cursor)
                    ));
                }
                self.fetch_timeline(&url)
            }
        }
    }
}

/// Main timeline command handler that delegates to a TimelineClient
pub struct TimelineCommand<C: TimelineClient> {
    client: C,
}

impl TimelineCommand<HttpTimelineClient> {
    /// Create a new timeline command with HTTP client from environment
    pub fn new() -> Result<Self, TimelineError> {
        let client = HttpTimelineClient::from_env()?;
        Ok(Self { client })
    }
}

impl<C: TimelineClient> TimelineCommand<C> {
    /// Create a timeline command with a custom client (for testing)
    pub fn with_client(client: C) -> Self {
        Self { client }
    }

    /// Retrieve a timeline based on the given arguments.
    pub fn get(&self, args: TimelineArgs) -> Result<TimelineResult, TimelineError> {
        self.client.get(&args)
    }
}

/// Mock implementation of TimelineClient for testing
pub struct MockTimelineClient {
    should_auth_fail: bool,
}

impl MockTimelineClient {
    /// Create a new mock client
    pub fn new() -> Self {
        Self {
            should_auth_fail: false,
        }
    }

    /// Create a mock that simulates auth failure
    pub fn with_auth_failure() -> Self {
        Self {
            should_auth_fail: true,
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
}

impl Default for MockTimelineClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelineClient for MockTimelineClient {
    fn get(&self, args: &TimelineArgs) -> Result<TimelineResult, TimelineError> {
        if self.should_auth_fail {
            return Err(TimelineError::AuthRequired);
        }

        let offset = Self::parse_cursor_offset(&args.cursor);
        let prefix = match &args.kind {
            TimelineKind::Home => "home",
            TimelineKind::Mentions => "mention",
            TimelineKind::User { handle } => handle.as_str(),
        };

        let tweets = Self::build_stub_tweets(prefix, offset, args.limit);
        let count = tweets.len();
        let meta = Self::build_pagination(offset, args.limit, count);

        Ok(TimelineResult { tweets, meta })
    }
}
