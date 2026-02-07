use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tweet data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tweet {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_history_tweet_ids: Option<Vec<String>>,
    #[serde(flatten)]
    pub additional_fields: HashMap<String, serde_json::Value>,
}

/// Metadata returned with tweet operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TweetMeta {
    pub client_request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_cache: Option<bool>,
}

/// Fields that can be requested for tweets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweetFields {
    Id,
    Text,
    AuthorId,
    CreatedAt,
    EditHistoryTweetIds,
}

impl TweetFields {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "id" => Some(Self::Id),
            "text" => Some(Self::Text),
            "author_id" => Some(Self::AuthorId),
            "created_at" => Some(Self::CreatedAt),
            "edit_history_tweet_ids" => Some(Self::EditHistoryTweetIds),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Text => "text",
            Self::AuthorId => "author_id",
            Self::CreatedAt => "created_at",
            Self::EditHistoryTweetIds => "edit_history_tweet_ids",
        }
    }

    /// Default fields for list operations (minimal set)
    pub fn default_fields() -> Vec<Self> {
        vec![Self::Id, Self::Text]
    }
}

impl Tweet {
    /// Create a new tweet with minimal fields
    pub fn new(id: String) -> Self {
        Self {
            id,
            text: None,
            author_id: None,
            created_at: None,
            edit_history_tweet_ids: None,
            additional_fields: HashMap::new(),
        }
    }

    /// Project tweet to only include requested fields
    pub fn project(&self, fields: &[TweetFields]) -> Self {
        let mut tweet = Tweet::new(String::new());

        for field in fields {
            match field {
                TweetFields::Id => tweet.id = self.id.clone(),
                TweetFields::Text => tweet.text = self.text.clone(),
                TweetFields::AuthorId => tweet.author_id = self.author_id.clone(),
                TweetFields::CreatedAt => tweet.created_at = self.created_at.clone(),
                TweetFields::EditHistoryTweetIds => {
                    tweet.edit_history_tweet_ids = self.edit_history_tweet_ids.clone()
                }
            }
        }

        tweet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tweet_projection() {
        let mut tweet = Tweet::new("123".to_string());
        tweet.text = Some("Hello world".to_string());
        tweet.author_id = Some("user_123".to_string());

        let projected = tweet.project(&[TweetFields::Id, TweetFields::Text]);
        assert_eq!(projected.id, "123");
        assert_eq!(projected.text, Some("Hello world".to_string()));
        assert_eq!(projected.author_id, None);
    }

    #[test]
    fn test_default_fields() {
        let fields = TweetFields::default_fields();
        assert_eq!(fields.len(), 2);
        assert!(fields.contains(&TweetFields::Id));
        assert!(fields.contains(&TweetFields::Text));
    }

    #[test]
    fn test_field_parse() {
        assert_eq!(TweetFields::parse("id"), Some(TweetFields::Id));
        assert_eq!(TweetFields::parse("text"), Some(TweetFields::Text));
        assert_eq!(TweetFields::parse("invalid"), None);
    }
}
