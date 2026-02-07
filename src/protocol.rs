use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Common response envelope for all commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub ok: bool,
    #[serde(rename = "type")]
    pub response_type: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

impl<T> Envelope<T> {
    /// Create a successful response
    pub fn success(response_type: impl Into<String>, data: T) -> Self {
        Self {
            ok: true,
            response_type: response_type.into(),
            schema_version: 1,
            data: Some(data),
            error: None,
            meta: None,
        }
    }

    /// Create a successful response with metadata
    pub fn success_with_meta(
        response_type: impl Into<String>,
        data: T,
        meta: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            ok: true,
            response_type: response_type.into(),
            schema_version: 1,
            data: Some(data),
            error: None,
            meta: Some(meta),
        }
    }
}

impl Envelope<()> {
    /// Create an error response
    pub fn error(response_type: impl Into<String>, error: ErrorDetails) -> Self {
        Self {
            ok: false,
            response_type: response_type.into(),
            schema_version: 1,
            data: None,
            error: Some(error),
            meta: None,
        }
    }

    /// Create an error response with metadata
    pub fn error_with_meta(
        response_type: impl Into<String>,
        error: ErrorDetails,
        meta: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            ok: false,
            response_type: response_type.into(),
            schema_version: 1,
            data: None,
            error: Some(error),
            meta: Some(meta),
        }
    }
}

/// Structured error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: ErrorCode,
    pub message: String,
    #[serde(rename = "isRetryable")]
    pub is_retryable: bool,
    #[serde(rename = "retryAfterMs", skip_serializing_if = "Option::is_none")]
    pub retry_after_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, serde_json::Value>>,
}

impl ErrorDetails {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        let is_retryable = code.is_retryable();
        Self {
            code,
            message: message.into(),
            is_retryable,
            retry_after_ms: None,
            details: None,
        }
    }

    pub fn with_details(
        code: ErrorCode,
        message: impl Into<String>,
        details: HashMap<String, serde_json::Value>,
    ) -> Self {
        let is_retryable = code.is_retryable();
        Self {
            code,
            message: message.into(),
            is_retryable,
            retry_after_ms: None,
            details: Some(details),
        }
    }

    pub fn with_retry_after(
        code: ErrorCode,
        message: impl Into<String>,
        retry_after_ms: u64,
    ) -> Self {
        let is_retryable = code.is_retryable();
        Self {
            code,
            message: message.into(),
            is_retryable,
            retry_after_ms: Some(retry_after_ms),
            details: None,
        }
    }

    /// Create an interaction required error with next steps guidance
    pub fn interaction_required(message: impl Into<String>, next_steps: Vec<String>) -> Self {
        let mut details = HashMap::new();
        details.insert("nextSteps".to_string(), serde_json::json!(next_steps));
        Self::with_details(ErrorCode::InteractionRequired, message, details)
    }

    /// Create an auth required error with next steps guidance
    pub fn auth_required(message: impl Into<String>, next_steps: Vec<String>) -> Self {
        let mut details = HashMap::new();
        details.insert("nextSteps".to_string(), serde_json::json!(next_steps));
        Self::with_details(ErrorCode::AuthRequired, message, details)
    }
}

/// Error code vocabulary
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidArgument,
    MissingArgument,
    UnknownCommand,
    AuthenticationFailed,
    AuthorizationFailed,
    AuthRequired,
    #[serde(rename = "rate_limited")]
    RateLimitExceeded,
    NetworkError,
    ServiceUnavailable,
    InternalError,
    NotFound,
    InvalidState,
    InteractionRequired,
    IdempotencyConflict,
    CostLimitExceeded,
    DailyBudgetExceeded,
}

impl ErrorCode {
    /// Determine if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ErrorCode::RateLimitExceeded | ErrorCode::NetworkError | ErrorCode::ServiceUnavailable
        )
    }

    /// Get the exit code for this error
    pub fn exit_code(&self) -> i32 {
        match self {
            ErrorCode::InvalidArgument | ErrorCode::MissingArgument | ErrorCode::UnknownCommand => {
                ExitCode::InvalidArgument.into()
            }
            ErrorCode::AuthenticationFailed
            | ErrorCode::AuthorizationFailed
            | ErrorCode::AuthRequired => ExitCode::AuthenticationError.into(),
            ErrorCode::RateLimitExceeded
            | ErrorCode::NetworkError
            | ErrorCode::ServiceUnavailable
            | ErrorCode::NotFound
            | ErrorCode::InvalidState
            | ErrorCode::InteractionRequired
            | ErrorCode::IdempotencyConflict
            | ErrorCode::CostLimitExceeded
            | ErrorCode::DailyBudgetExceeded => ExitCode::OperationFailed.into(),
            ErrorCode::InternalError => ExitCode::OperationFailed.into(),
        }
    }
}

/// Exit code policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    InvalidArgument = 2,
    AuthenticationError = 3,
    OperationFailed = 4,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> i32 {
        code as i32
    }
}

impl ExitCode {
    pub fn from_error_code(error_code: ErrorCode) -> Self {
        match error_code {
            ErrorCode::InvalidArgument | ErrorCode::MissingArgument | ErrorCode::UnknownCommand => {
                ExitCode::InvalidArgument
            }
            ErrorCode::AuthenticationFailed
            | ErrorCode::AuthorizationFailed
            | ErrorCode::AuthRequired => ExitCode::AuthenticationError,
            _ => ExitCode::OperationFailed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_envelope() {
        let envelope = Envelope::success("test", "data");
        assert!(envelope.ok);
        assert_eq!(envelope.schema_version, 1);
        assert_eq!(envelope.response_type, "test");
        assert_eq!(envelope.data, Some("data"));
        assert!(envelope.error.is_none());
    }

    #[test]
    fn test_error_envelope() {
        let error = ErrorDetails::new(ErrorCode::InvalidArgument, "test error");
        let envelope = Envelope::<()>::error("test", error);
        assert!(!envelope.ok);
        assert_eq!(envelope.schema_version, 1);
        assert!(envelope.data.is_none());
        assert!(envelope.error.is_some());
    }

    #[test]
    fn test_error_retryable() {
        assert!(ErrorCode::RateLimitExceeded.is_retryable());
        assert!(ErrorCode::NetworkError.is_retryable());
        assert!(ErrorCode::ServiceUnavailable.is_retryable());
        assert!(!ErrorCode::InvalidArgument.is_retryable());
        assert!(!ErrorCode::AuthenticationFailed.is_retryable());
    }

    #[test]
    fn test_exit_codes() {
        assert_eq!(ErrorCode::InvalidArgument.exit_code(), 2);
        assert_eq!(ErrorCode::AuthenticationFailed.exit_code(), 3);
        assert_eq!(ErrorCode::AuthRequired.exit_code(), 3);
        assert_eq!(ErrorCode::NetworkError.exit_code(), 4);
        assert_eq!(ErrorCode::InteractionRequired.exit_code(), 4);
        assert_eq!(ErrorCode::CostLimitExceeded.exit_code(), 4);
    }

    #[test]
    fn test_interaction_required_error() {
        let error = ErrorDetails::interaction_required(
            "Authentication required",
            vec!["Run 'xcom-rs auth login' first".to_string()],
        );
        assert_eq!(error.code, ErrorCode::InteractionRequired);
        assert!(!error.is_retryable);
        assert!(error.details.is_some());
        let details = error.details.unwrap();
        assert!(details.contains_key("nextSteps"));
    }
}
