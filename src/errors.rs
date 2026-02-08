use crate::{
    output::{print_envelope, OutputFormat},
    protocol::{Envelope, ErrorDetails, ExitCode},
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

    // Note: Cannot test emit() as it calls std::process::exit
    // Integration tests should cover the end-to-end behavior
}
