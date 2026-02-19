use serde::{Deserialize, Serialize};

/// A tweet returned by search results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchTweet {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

impl SearchTweet {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: None,
            author_id: None,
            created_at: None,
        }
    }
}

/// A user returned by user search results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchUser {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl SearchUser {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: None,
            username: None,
            description: None,
        }
    }
}

/// Pagination metadata for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPaginationMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_token: Option<String>,
    pub result_count: usize,
}

/// Result of a recent tweet search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRecentResult {
    pub tweets: Vec<SearchTweet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<SearchResultMeta>,
}

/// Result of a user search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchUsersResult {
    pub users: Vec<SearchUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<SearchResultMeta>,
}

/// Metadata for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultMeta {
    pub pagination: SearchPaginationMeta,
}
