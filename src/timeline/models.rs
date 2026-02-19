use serde::{Deserialize, Serialize};

use crate::tweets::Tweet;

/// The kind of timeline to retrieve
#[derive(Debug, Clone)]
pub enum TimelineKind {
    /// Home timeline (reverse chronological)
    Home,
    /// Mentions timeline
    Mentions,
    /// Tweets from a specific user handle
    User { handle: String },
}

/// Arguments for timeline retrieval
#[derive(Debug, Clone)]
pub struct TimelineArgs {
    pub kind: TimelineKind,
    pub limit: usize,
    pub cursor: Option<String>,
}

/// Pagination information for timeline results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelinePagination {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_token: Option<String>,
}

/// Metadata returned with timeline results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineMeta {
    pub pagination: TimelinePagination,
}

/// Result of a timeline retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineResult {
    pub tweets: Vec<Tweet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<TimelineMeta>,
}
