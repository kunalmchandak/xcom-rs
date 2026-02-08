use crate::{
    output::{print_envelope, OutputFormat},
    protocol::{Envelope, ErrorCode, ErrorDetails, ExitCode},
};
use std::collections::HashMap;

/// Helper struct for emitting error responses with consistent formatting and metadata
pub struct ErrorResponder;

impl ErrorResponder {
    /// Emit an error response and exit with the appropriate exit code
    ///
    /// This centralizes error response generation by:
    /// - Creating an error envelope with optional metadata
    /// - Printing the response in the specified format
    /// - Exiting with the appropriate exit code
    pub fn emit(
        error: ErrorDetails,
        output_format: OutputFormat,
        meta: Option<HashMap<String, serde_json::Value>>,
        exit_code: ExitCode,
    ) -> ! {
        let envelope = if let Some(meta) = meta {
            Envelope::<()>::error_with_meta("error", error, meta)
        } else {
            Envelope::<()>::error("error", error)
        };
        let _ = print_envelope(&envelope, output_format);
        std::process::exit(exit_code.into());
    }

    /// Create metadata map from trace_id if present
    pub fn create_meta(trace_id: Option<&String>) -> Option<HashMap<String, serde_json::Value>> {
        trace_id.map(|trace_id| {
            let mut m = HashMap::new();
            m.insert("traceId".to_string(), serde_json::json!(trace_id));
            m
        })
    }

    /// Create a simple error with code and message
    pub fn error(code: ErrorCode, message: impl Into<String>) -> ErrorDetails {
        ErrorDetails::new(code, message)
    }

    /// Create an error with retry_after_ms
    pub fn error_with_retry(
        code: ErrorCode,
        message: impl Into<String>,
        retry_after_ms: u64,
    ) -> ErrorDetails {
        ErrorDetails::with_retry_after(code, message, retry_after_ms)
    }

    /// Create an error with additional details
    pub fn error_with_details(
        code: ErrorCode,
        message: impl Into<String>,
        details: HashMap<String, serde_json::Value>,
    ) -> ErrorDetails {
        ErrorDetails::with_details(code, message, details)
    }

    /// Create an auth required error with next steps
    pub fn auth_required_error(
        message: impl Into<String>,
        next_steps: Vec<String>,
    ) -> ErrorDetails {
        ErrorDetails::auth_required(message, next_steps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_meta_with_trace_id() {
        let trace_id = "test-trace-123".to_string();
        let meta = ErrorResponder::create_meta(Some(&trace_id));
        assert!(meta.is_some());
        let meta = meta.unwrap();
        assert_eq!(meta.get("traceId").unwrap(), "test-trace-123");
    }

    #[test]
    fn test_create_meta_without_trace_id() {
        let meta = ErrorResponder::create_meta(None);
        assert!(meta.is_none());
    }

    #[test]
    fn test_error_builder() {
        let error = ErrorResponder::error(ErrorCode::InvalidArgument, "test message");
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert_eq!(error.message, "test message");
        assert!(!error.is_retryable);
        assert!(error.retry_after_ms.is_none());
        assert!(error.details.is_none());
    }

    #[test]
    fn test_error_with_retry_builder() {
        let error =
            ErrorResponder::error_with_retry(ErrorCode::RateLimitExceeded, "rate limited", 5000);
        assert_eq!(error.code, ErrorCode::RateLimitExceeded);
        assert_eq!(error.message, "rate limited");
        assert!(error.is_retryable);
        assert_eq!(error.retry_after_ms, Some(5000));
        assert!(error.details.is_none());
    }

    #[test]
    fn test_error_with_details_builder() {
        let mut details = HashMap::new();
        details.insert("key".to_string(), serde_json::json!("value"));
        let error =
            ErrorResponder::error_with_details(ErrorCode::InternalError, "error", details.clone());
        assert_eq!(error.code, ErrorCode::InternalError);
        assert_eq!(error.message, "error");
        assert!(error.details.is_some());
        assert_eq!(error.details.unwrap().get("key").unwrap(), "value");
    }

    #[test]
    fn test_auth_required_error_builder() {
        let next_steps = vec!["Run auth login".to_string()];
        let error = ErrorResponder::auth_required_error("auth needed", next_steps.clone());
        assert_eq!(error.code, ErrorCode::AuthRequired);
        assert_eq!(error.message, "auth needed");
        assert!(error.details.is_some());
        let details = error.details.unwrap();
        assert!(details.contains_key("nextSteps"));
    }

    // Note: Cannot test emit() as it calls std::process::exit
    // Integration tests should cover the end-to-end behavior
}
