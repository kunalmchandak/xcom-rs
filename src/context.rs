use crate::billing::{BudgetTracker, CostEstimate};
use crate::protocol::{ErrorCode, ErrorDetails};

/// Execution context for commands
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Whether running in non-interactive mode
    pub non_interactive: bool,
    /// Optional trace ID for request correlation
    pub trace_id: Option<String>,
    /// Maximum cost in credits for a single operation
    pub max_cost_credits: Option<u32>,
    /// Daily budget in credits
    pub budget_daily_credits: Option<u32>,
    /// Dry run mode
    pub dry_run: bool,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(
        non_interactive: bool,
        trace_id: Option<String>,
        max_cost_credits: Option<u32>,
        budget_daily_credits: Option<u32>,
        dry_run: bool,
    ) -> Self {
        Self {
            non_interactive,
            trace_id,
            max_cost_credits,
            budget_daily_credits,
            dry_run,
        }
    }

    /// Check if authentication is required and return an error if in non-interactive mode
    ///
    /// This helper should be called by commands that need authentication or user interaction.
    /// If in non-interactive mode, it returns an AUTH_REQUIRED error with next steps.
    /// Otherwise, it allows the command to proceed with interactive prompts.
    ///
    /// # Example
    /// ```
    /// use xcom_rs::context::ExecutionContext;
    ///
    /// let ctx = ExecutionContext::new(true, None, None, None, false);
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
            Some(ErrorDetails::auth_required(message, next_steps))
        } else {
            None
        }
    }

    /// Check if cost exceeds maximum allowed
    pub fn check_max_cost(&self, cost: &CostEstimate) -> Option<ErrorDetails> {
        if let Some(max) = self.max_cost_credits {
            if cost.credits > max {
                let mut details = std::collections::HashMap::new();
                details.insert("cost".to_string(), serde_json::json!(cost.credits));
                details.insert("limit".to_string(), serde_json::json!(max));
                return Some(ErrorDetails::with_details(
                    ErrorCode::CostLimitExceeded,
                    format!(
                        "Operation cost {} credits exceeds maximum {} credits",
                        cost.credits, max
                    ),
                    details,
                ));
            }
        }
        None
    }

    /// Check if cost would exceed daily budget
    pub fn check_daily_budget(
        &self,
        cost: &CostEstimate,
        tracker: &BudgetTracker,
    ) -> Option<ErrorDetails> {
        if tracker.check_budget(cost.credits).is_err() {
            let mut details = std::collections::HashMap::new();
            details.insert("cost".to_string(), serde_json::json!(cost.credits));
            details.insert(
                "todayUsage".to_string(),
                serde_json::json!(tracker.today_usage()),
            );
            if let Some(limit) = self.budget_daily_credits {
                details.insert("dailyLimit".to_string(), serde_json::json!(limit));
            }
            return Some(ErrorDetails::with_details(
                ErrorCode::DailyBudgetExceeded,
                "Daily budget exceeded".to_string(),
                details,
            ));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::ErrorCode;

    #[test]
    fn test_context_creation() {
        let ctx = ExecutionContext::new(true, Some("trace-123".to_string()), None, None, false);
        assert!(ctx.non_interactive);
        assert_eq!(ctx.trace_id, Some("trace-123".to_string()));
    }

    #[test]
    fn test_check_interaction_required_non_interactive() {
        let ctx = ExecutionContext::new(true, None, None, None, false);
        let error =
            ctx.check_interaction_required("Auth required", vec!["Run login command".to_string()]);
        assert!(error.is_some());
        let err = error.unwrap();
        assert_eq!(err.code, ErrorCode::AuthRequired);
        assert!(!err.is_retryable);
        assert!(err.details.is_some());
    }

    #[test]
    fn test_check_interaction_required_interactive() {
        let ctx = ExecutionContext::new(false, None, None, None, false);
        let error =
            ctx.check_interaction_required("Auth required", vec!["Run login command".to_string()]);
        assert!(error.is_none());
    }

    #[test]
    fn test_check_max_cost_within_limit() {
        let ctx = ExecutionContext::new(false, None, Some(100), None, false);
        let cost = CostEstimate::new(50, 0.05);
        let error = ctx.check_max_cost(&cost);
        assert!(error.is_none());
    }

    #[test]
    fn test_check_max_cost_exceeds_limit() {
        let ctx = ExecutionContext::new(false, None, Some(100), None, false);
        let cost = CostEstimate::new(101, 0.101);
        let error = ctx.check_max_cost(&cost);
        assert!(error.is_some());
        let err = error.unwrap();
        assert_eq!(err.code, ErrorCode::CostLimitExceeded);
    }

    #[test]
    fn test_check_daily_budget_within_limit() {
        let ctx = ExecutionContext::new(false, None, None, Some(100), false);
        let tracker = BudgetTracker::new(Some(100));
        let cost = CostEstimate::new(50, 0.05);
        let error = ctx.check_daily_budget(&cost, &tracker);
        assert!(error.is_none());
    }

    #[test]
    fn test_check_daily_budget_exceeds_limit() {
        let ctx = ExecutionContext::new(false, None, None, Some(100), false);
        let tracker = BudgetTracker::new(Some(100));
        let cost = CostEstimate::new(101, 0.101);
        let error = ctx.check_daily_budget(&cost, &tracker);
        assert!(error.is_some());
        let err = error.unwrap();
        assert_eq!(err.code, ErrorCode::DailyBudgetExceeded);
    }
}
