//! Shared types for tweets command arguments, results, and errors.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::tweets::models::{Tweet, TweetFields, TweetMeta};

/// Custom error type for idempotency conflicts
#[derive(Debug)]
pub struct IdempotencyConflictError {
    pub client_request_id: String,
}

/// Custom error type for partial thread failures.
/// Contains information about which tweet in the thread failed,
/// and which tweets were successfully created before the failure.
#[derive(Debug)]
pub struct ThreadPartialFailureError {
    /// Index of the tweet that failed (0-based)
    pub failed_index: usize,
    /// IDs of tweets successfully created before the failure
    pub created_tweet_ids: Vec<String>,
    /// Underlying error message
    pub message: String,
}

impl std::fmt::Display for ThreadPartialFailureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Thread posting failed at index {}: {} ({} tweets created before failure)",
            self.failed_index,
            self.message,
            self.created_tweet_ids.len()
        )
    }
}

impl std::error::Error for ThreadPartialFailureError {}

impl std::fmt::Display for IdempotencyConflictError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Operation with client_request_id '{}' already exists",
            self.client_request_id
        )
    }
}

impl std::error::Error for IdempotencyConflictError {}

/// Policy for handling existing operations with the same client_request_id
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IfExistsPolicy {
    /// Return the existing result without error
    Return,
    /// Return an error if operation already exists
    Error,
}

impl FromStr for IfExistsPolicy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "return" => Ok(Self::Return),
            "error" => Ok(Self::Error),
            _ => Err(anyhow!(
                "Invalid if-exists policy: {}. Valid values: return, error",
                s
            )),
        }
    }
}

impl IfExistsPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Return => "return",
            Self::Error => "error",
        }
    }
}

/// Arguments for creating a tweet
#[derive(Debug, Clone)]
pub struct CreateArgs {
    pub text: String,
    pub client_request_id: Option<String>,
    pub if_exists: IfExistsPolicy,
}

/// Arguments for listing tweets
#[derive(Debug, Clone)]
pub struct ListArgs {
    pub fields: Vec<TweetFields>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

/// Arguments for engagement operations (like/unlike/retweet/unretweet)
#[derive(Debug, Clone)]
pub struct EngagementArgs {
    pub tweet_id: String,
}

/// Result of an engagement operation (like/unlike/retweet/unretweet)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementResult {
    pub tweet_id: String,
    pub success: bool,
}

/// Arguments for replying to a tweet
#[derive(Debug, Clone)]
pub struct ReplyArgs {
    pub tweet_id: String,
    pub text: String,
    pub client_request_id: Option<String>,
    pub if_exists: IfExistsPolicy,
}

/// Arguments for creating a thread of tweets
#[derive(Debug, Clone)]
pub struct ThreadArgs {
    pub texts: Vec<String>,
    pub client_request_id_prefix: Option<String>,
    pub if_exists: IfExistsPolicy,
}

/// Arguments for showing a single tweet
#[derive(Debug, Clone)]
pub struct ShowArgs {
    pub tweet_id: String,
}

/// Arguments for retrieving a conversation tree
#[derive(Debug, Clone)]
pub struct ConversationArgs {
    pub tweet_id: String,
}

/// Result of a create operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResult {
    pub tweet: Tweet,
    pub meta: TweetMeta,
}

/// Result of a reply operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyResult {
    pub tweet: Tweet,
    pub meta: TweetMeta,
}

/// Result of a thread operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadResult {
    pub tweets: Vec<Tweet>,
    pub meta: ThreadMeta,
}

/// Metadata for thread results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadMeta {
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_index: Option<usize>,
    pub created_tweet_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_cache: Option<bool>,
}

/// Result of a show operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowResult {
    pub tweet: Tweet,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_cursor: Option<String>,
}

/// Result of a list operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResult {
    pub tweets: Vec<Tweet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ListResultMeta>,
}

/// Metadata for list results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResultMeta {
    pub pagination: PaginationMeta,
}

/// Error classification for retry logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Retryable errors (429, 5xx)
    Retryable,
    /// Non-retryable client errors (4xx except 429)
    NonRetryable,
    /// Network/timeout errors
    Timeout,
}

/// Classified error with retry information
#[derive(Debug)]
pub struct ClassifiedError {
    pub kind: ErrorKind,
    pub status_code: Option<u16>,
    pub message: String,
    pub is_retryable: bool,
    pub retry_after_ms: Option<u64>,
}

impl ClassifiedError {
    pub fn from_status_code(status_code: u16, message: String) -> Self {
        let (kind, is_retryable) = match status_code {
            429 => (ErrorKind::Retryable, true),
            500..=599 => (ErrorKind::Retryable, true),
            400..=499 => (ErrorKind::NonRetryable, false),
            _ => (ErrorKind::NonRetryable, false),
        };

        Self {
            kind,
            status_code: Some(status_code),
            message,
            is_retryable,
            retry_after_ms: None,
        }
    }

    pub fn timeout(message: String) -> Self {
        Self {
            kind: ErrorKind::Timeout,
            status_code: None,
            message,
            is_retryable: true,
            retry_after_ms: None,
        }
    }

    pub fn with_retry_after(mut self, retry_after_ms: u64) -> Self {
        self.retry_after_ms = Some(retry_after_ms);
        self
    }

    /// Convert to ErrorCode for protocol
    pub fn to_error_code(&self) -> crate::protocol::ErrorCode {
        use crate::protocol::ErrorCode;
        match self.kind {
            ErrorKind::Retryable => {
                if let Some(429) = self.status_code {
                    ErrorCode::RateLimitExceeded
                } else {
                    ErrorCode::ServiceUnavailable
                }
            }
            ErrorKind::Timeout => ErrorCode::NetworkError,
            ErrorKind::NonRetryable => ErrorCode::InternalError,
        }
    }
}

impl std::fmt::Display for ClassifiedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ClassifiedError {}
