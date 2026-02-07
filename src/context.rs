use crate::protocol::ErrorDetails;

/// Execution context for commands
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Whether running in non-interactive mode
    pub non_interactive: bool,
    /// Optional trace ID for request correlation
    pub trace_id: Option<String>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(non_interactive: bool, trace_id: Option<String>) -> Self {
        Self {
            non_interactive,
            trace_id,
        }
    }

    /// Check if interaction is required and return an error if in non-interactive mode
    ///
    /// This helper should be called by commands that need user interaction.
    /// If in non-interactive mode, it returns an INTERACTION_REQUIRED error with next steps.
    /// Otherwise, it allows the command to proceed with interactive prompts.
    ///
    /// # Example
    /// ```
    /// use xcom_rs::context::ExecutionContext;
    ///
    /// let ctx = ExecutionContext::new(true, None);
    /// let error = ctx.check_interaction_required(
    ///     "Authentication required",
    ///     vec!["Run 'xcom-rs auth login' first".to_string()]
    /// );
    /// // If error.is_some(), handle the interaction requirement
    /// assert!(error.is_some());
    /// ```
    pub fn check_interaction_required(
        &self,
        message: impl Into<String>,
        next_steps: Vec<String>,
    ) -> Option<ErrorDetails> {
        if self.non_interactive {
            Some(ErrorDetails::interaction_required(message, next_steps))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::ErrorCode;

    #[test]
    fn test_context_creation() {
        let ctx = ExecutionContext::new(true, Some("trace-123".to_string()));
        assert!(ctx.non_interactive);
        assert_eq!(ctx.trace_id, Some("trace-123".to_string()));
    }

    #[test]
    fn test_check_interaction_required_non_interactive() {
        let ctx = ExecutionContext::new(true, None);
        let error =
            ctx.check_interaction_required("Auth required", vec!["Run login command".to_string()]);
        assert!(error.is_some());
        let err = error.unwrap();
        assert_eq!(err.code, ErrorCode::InteractionRequired);
        assert!(!err.is_retryable);
        assert!(err.details.is_some());
    }

    #[test]
    fn test_check_interaction_required_interactive() {
        let ctx = ExecutionContext::new(false, None);
        let error =
            ctx.check_interaction_required("Auth required", vec!["Run login command".to_string()]);
        assert!(error.is_none());
    }
}
