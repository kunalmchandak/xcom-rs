use serde::{Deserialize, Serialize};

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
