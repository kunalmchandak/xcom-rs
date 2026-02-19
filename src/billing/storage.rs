use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::models::CostEstimate;

/// Compare two JSON strings for equality by parsing and re-serializing
/// This handles key ordering differences
fn json_content_equal(json1: &str, json2: &str) -> bool {
    match (
        serde_json::from_str::<serde_json::Value>(json1),
        serde_json::from_str::<serde_json::Value>(json2),
    ) {
        (Ok(v1), Ok(v2)) => v1 == v2,
        _ => false, // If parsing fails, treat as different
    }
}

/// Daily budget tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetTracker {
    daily_limit: Option<u32>,
    usage: HashMap<String, u32>, // date -> credits used
    #[serde(skip)]
    storage_path: Option<PathBuf>,
}

impl BudgetTracker {
    pub fn new(daily_limit: Option<u32>) -> Self {
        Self {
            daily_limit,
            usage: HashMap::new(),
            storage_path: None,
        }
    }

    /// Create a budget tracker with persistent storage at the given path
    pub fn with_storage(daily_limit: Option<u32>, path: PathBuf) -> Result<Self> {
        let mut tracker = Self {
            daily_limit,
            usage: HashMap::new(),
            storage_path: Some(path.clone()),
        };

        // Try to load existing usage from storage
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(loaded) = serde_json::from_str::<BudgetTracker>(&content) {
                    tracker.usage = loaded.usage;
                    // Update daily_limit from parameter, not from storage
                }
            }
        }

        Ok(tracker)
    }

    /// Get default storage path: respects XDG_DATA_HOME, falls back to ~/.local/share/xcom-rs/budget.json
    pub fn default_storage_path() -> Result<PathBuf> {
        let data_dir = if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
            PathBuf::from(xdg_data).join("xcom-rs")
        } else {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map_err(|_| anyhow::anyhow!("Could not determine home directory"))?;
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("xcom-rs")
        };
        std::fs::create_dir_all(&data_dir)?;
        Ok(data_dir.join("budget.json"))
    }

    /// Create a budget tracker with default storage location
    pub fn with_default_storage(daily_limit: Option<u32>) -> Result<Self> {
        Self::with_storage(daily_limit, Self::default_storage_path()?)
    }

    /// Save the current usage to persistent storage
    /// Only writes if the content has changed (prevents unnecessary file modifications)
    fn save_to_storage(&self) -> Result<()> {
        if let Some(path) = &self.storage_path {
            let new_json = serde_json::to_string_pretty(self)?;

            // Check if file exists and compare content
            let should_write = if path.exists() {
                match std::fs::read_to_string(path) {
                    Ok(existing_json) => {
                        // Compare normalized JSON to handle key ordering differences
                        !json_content_equal(&existing_json, &new_json)
                    }
                    Err(_) => true, // If we can't read, write anyway
                }
            } else {
                true // File doesn't exist, write it
            };

            if should_write {
                std::fs::write(path, new_json)?;
            }
        }
        Ok(())
    }

    /// Get today's date as a string (YYYY-MM-DD)
    /// Returns a default value if system time is invalid
    fn today() -> String {
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => {
                let now = duration.as_secs();
                let days = now / 86400;
                // Format as YYYY-MM-DD
                format!("day-{}", days)
            }
            Err(_) => {
                // System time is before UNIX_EPOCH - use a fallback value
                // This should never happen in practice, but we handle it gracefully
                "day-0".to_string()
            }
        }
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
        let _ = self.save_to_storage(); // Ignore errors here to not break the flow
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
        // Timeline operation rates
        rate_table.insert("timeline.home".to_string(), 2);
        rate_table.insert("timeline.mentions".to_string(), 2);
        rate_table.insert("timeline.user".to_string(), 1);

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

    #[test]
    fn test_stable_writes_no_modification_on_same_content() {
        use std::fs;

        // Create a temporary directory for testing
        let temp_dir = std::env::temp_dir().join(format!("xcom-rs-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();
        let storage_path = temp_dir.join("budget.json");

        // Create a tracker with storage
        let mut tracker = BudgetTracker::with_storage(Some(100), storage_path.clone()).unwrap();

        // Record some usage
        tracker.record_usage(30);

        // Get the initial modification time
        let metadata1 = fs::metadata(&storage_path).unwrap();
        let mtime1 = metadata1.modified().unwrap();

        // Wait a small amount to ensure timestamps would differ if file was rewritten
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Save again without changing content (by loading and immediately saving)
        let tracker2 = BudgetTracker::with_storage(Some(100), storage_path.clone()).unwrap();
        // The loaded tracker has the same usage, so saving should not modify the file
        drop(tracker2); // This won't save since we didn't call record_usage

        // Instead, let's create a new tracker with the same state and force a save
        let mut tracker3 = BudgetTracker::with_storage(Some(100), storage_path.clone()).unwrap();
        // Record 0 usage to trigger save without changing content
        tracker3.record_usage(0);

        // Get the modification time again
        let metadata2 = fs::metadata(&storage_path).unwrap();
        let mtime2 = metadata2.modified().unwrap();

        // The modification time should be the same (file was not rewritten)
        assert_eq!(
            mtime1, mtime2,
            "File modification time changed despite identical content"
        );

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_stable_writes_modification_on_different_content() {
        use std::fs;

        // Create a temporary directory for testing
        let temp_dir = std::env::temp_dir().join(format!("xcom-rs-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();
        let storage_path = temp_dir.join("budget.json");

        // Create a tracker with storage
        let mut tracker = BudgetTracker::with_storage(Some(100), storage_path.clone()).unwrap();

        // Record some usage
        tracker.record_usage(30);

        // Get the initial modification time
        let metadata1 = fs::metadata(&storage_path).unwrap();
        let mtime1 = metadata1.modified().unwrap();

        // Wait a small amount to ensure timestamps would differ
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Record different usage (should rewrite)
        tracker.record_usage(20);

        // Get the modification time again
        let metadata2 = fs::metadata(&storage_path).unwrap();
        let mtime2 = metadata2.modified().unwrap();

        // The modification time should be different (file was rewritten)
        assert!(
            mtime1 < mtime2,
            "File modification time did not change despite different content"
        );

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_default_storage_path_with_xdg_data_home() {
        // Use a shared global mutex to prevent parallel test execution from interfering
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();

        // Save current value
        let original = std::env::var("XDG_DATA_HOME").ok();

        // Set XDG_DATA_HOME
        let xdg_path = std::env::temp_dir().join(format!("test-xdg-data-{}", std::process::id()));
        std::env::set_var("XDG_DATA_HOME", &xdg_path);

        let path = BudgetTracker::default_storage_path();

        // Restore original value
        match original {
            Some(val) => std::env::set_var("XDG_DATA_HOME", val),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }

        assert!(path.is_ok());
        let path = path.unwrap();
        assert_eq!(path, xdg_path.join("xcom-rs").join("budget.json"));
    }

    #[test]
    fn test_default_storage_path_without_xdg() {
        // Use a shared global mutex to prevent parallel test execution from interfering
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap();

        // Save current value
        let original = std::env::var("XDG_DATA_HOME").ok();

        // Ensure XDG_DATA_HOME is not set
        std::env::remove_var("XDG_DATA_HOME");

        let path = BudgetTracker::default_storage_path();

        // Restore original value
        if let Some(val) = original {
            std::env::set_var("XDG_DATA_HOME", val);
        }

        assert!(path.is_ok());
        let path = path.unwrap();
        // Should fall back to ~/.local/share/xcom-rs/budget.json
        let expected_suffix = std::path::Path::new(".local")
            .join("share")
            .join("xcom-rs")
            .join("budget.json");
        assert!(path.ends_with(&expected_suffix));
    }
}
