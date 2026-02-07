use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cost estimate for an operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostEstimate {
    pub credits: u32,
    #[serde(rename = "usdEstimated")]
    pub usd_estimated: f64,
}

impl CostEstimate {
    pub fn new(credits: u32, usd_estimated: f64) -> Self {
        Self {
            credits,
            usd_estimated,
        }
    }

    /// Zero cost (for dry-run)
    pub fn zero() -> Self {
        Self {
            credits: 0,
            usd_estimated: 0.0,
        }
    }
}

/// Billing estimate response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingEstimate {
    pub operation: String,
    pub cost: CostEstimate,
}

/// Daily budget tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetTracker {
    daily_limit: Option<u32>,
    usage: HashMap<String, u32>, // date -> credits used
}

impl BudgetTracker {
    pub fn new(daily_limit: Option<u32>) -> Self {
        Self {
            daily_limit,
            usage: HashMap::new(),
        }
    }

    /// Get today's date as a string (YYYY-MM-DD)
    fn today() -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let days = now / 86400;
        // Format as YYYY-MM-DD
        format!("day-{}", days)
    }

    /// Check if an operation would exceed daily budget
    pub fn check_budget(&self, cost: u32) -> Result<()> {
        if let Some(limit) = self.daily_limit {
            let today = Self::today();
            let used = self.usage.get(&today).copied().unwrap_or(0);
            if used + cost > limit {
                return Err(anyhow::anyhow!(
                    "Daily budget exceeded: {} + {} > {}",
                    used,
                    cost,
                    limit
                ));
            }
        }
        Ok(())
    }

    /// Record usage for today
    pub fn record_usage(&mut self, cost: u32) {
        let today = Self::today();
        let used = self.usage.entry(today).or_insert(0);
        *used += cost;
    }

    /// Get today's usage
    pub fn today_usage(&self) -> u32 {
        let today = Self::today();
        self.usage.get(&today).copied().unwrap_or(0)
    }
}

/// Cost estimator for X API operations
pub struct CostEstimator {
    // In real implementation, this would load from a config file or API
    rate_table: HashMap<String, u32>,
    usd_per_credit: f64,
}

impl CostEstimator {
    pub fn new() -> Self {
        let mut rate_table = HashMap::new();
        // Example rates (stub data)
        rate_table.insert("tweets.create".to_string(), 5);
        rate_table.insert("tweets.read".to_string(), 1);
        rate_table.insert("users.read".to_string(), 1);
        rate_table.insert("search.tweets".to_string(), 3);

        Self {
            rate_table,
            usd_per_credit: 0.001, // $0.001 per credit
        }
    }

    /// Estimate cost for an operation
    pub fn estimate(&self, operation: &str, params: &HashMap<String, String>) -> CostEstimate {
        let base_credits = self.rate_table.get(operation).copied().unwrap_or(1);

        // Adjust based on parameters (stub logic)
        let credits = if let Some(text) = params.get("text") {
            // Example: longer text costs more
            let length_multiplier = (text.len() / 100).max(1) as u32;
            base_credits * length_multiplier
        } else {
            base_credits
        };

        CostEstimate::new(credits, credits as f64 * self.usd_per_credit)
    }
}

impl Default for CostEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_estimate_creation() {
        let cost = CostEstimate::new(10, 0.01);
        assert_eq!(cost.credits, 10);
        assert_eq!(cost.usd_estimated, 0.01);
    }

    #[test]
    fn test_cost_estimate_zero() {
        let cost = CostEstimate::zero();
        assert_eq!(cost.credits, 0);
        assert_eq!(cost.usd_estimated, 0.0);
    }

    #[test]
    fn test_estimator_basic() {
        let estimator = CostEstimator::new();
        let params = HashMap::new();
        let cost = estimator.estimate("tweets.create", &params);
        assert_eq!(cost.credits, 5);
        assert_eq!(cost.usd_estimated, 0.005);
    }

    #[test]
    fn test_estimator_with_params() {
        let estimator = CostEstimator::new();
        let mut params = HashMap::new();
        params.insert("text".to_string(), "a".repeat(200));
        let cost = estimator.estimate("tweets.create", &params);
        assert_eq!(cost.credits, 10); // 5 * 2 (length multiplier)
    }

    #[test]
    fn test_estimator_unknown_operation() {
        let estimator = CostEstimator::new();
        let params = HashMap::new();
        let cost = estimator.estimate("unknown.operation", &params);
        assert_eq!(cost.credits, 1);
    }

    #[test]
    fn test_budget_tracker_no_limit() {
        let tracker = BudgetTracker::new(None);
        assert!(tracker.check_budget(1000).is_ok());
    }

    #[test]
    fn test_budget_tracker_within_limit() {
        let tracker = BudgetTracker::new(Some(100));
        assert!(tracker.check_budget(50).is_ok());
    }

    #[test]
    fn test_budget_tracker_exceeds_limit() {
        let tracker = BudgetTracker::new(Some(100));
        assert!(tracker.check_budget(101).is_err());
    }

    #[test]
    fn test_budget_tracker_accumulation() {
        let mut tracker = BudgetTracker::new(Some(100));
        tracker.record_usage(30);
        tracker.record_usage(40);
        assert_eq!(tracker.today_usage(), 70);
        assert!(tracker.check_budget(30).is_ok());
        assert!(tracker.check_budget(31).is_err());
    }
}
