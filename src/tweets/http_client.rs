//! HTTP client for X API tweet operations.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::commands::types::ClassifiedError;
use super::models::Tweet;

const X_API_BASE: &str = "https://api.x.com";

/// Request body for creating a tweet
#[derive(Debug, Serialize)]
struct CreateTweetRequest {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply: Option<ReplySettings>,
}

/// Reply settings for creating a reply tweet
#[derive(Debug, Serialize)]
struct ReplySettings {
    in_reply_to_tweet_id: String,
}

/// Request body for like/retweet operations
#[derive(Debug, Serialize)]
struct EngagementRequest {
    tweet_id: String,
}

/// Response from engagement operations
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct EngagementResponse {
    data: EngagementData,
}

/// Engagement data from API response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct EngagementData {
    #[serde(default)]
    liked: bool,
    #[serde(default)]
    retweeted: bool,
}

/// Response from creating a tweet
#[derive(Debug, Deserialize)]
struct CreateTweetResponse {
    data: TweetData,
}

/// Tweet data from API response
#[derive(Debug, Deserialize)]
struct TweetData {
    id: String,
    text: String,
}

/// Response from getting user information
#[derive(Debug, Deserialize)]
struct UserMeResponse {
    data: UserData,
}

/// User data from API response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct UserData {
    id: String,
    username: String,
}

/// HTTP client for X API operations
pub struct XApiClient {
    base_url: String,
}

impl XApiClient {
    /// Create a new X API client
    pub fn new() -> Self {
        Self {
            base_url: X_API_BASE.to_string(),
        }
    }

    /// Create a new X API client with custom base URL (for testing)
    pub fn with_base_url(base_url: String) -> Self {
        Self { base_url }
    }

    /// Get bearer token from auth store
    fn get_bearer_token(&self) -> Result<String> {
        std::env::var("XCOM_RS_BEARER_TOKEN")
            .context("XCOM_RS_BEARER_TOKEN not set")?
            .strip_prefix("Bearer ")
            .map(String::from)
            .or_else(|| std::env::var("XCOM_RS_BEARER_TOKEN").ok())
            .context("Failed to parse bearer token")
    }

    /// Get the current user's ID
    pub fn get_user_id(&self) -> Result<String> {
        let token = self.get_bearer_token()?;
        let url = format!("{}/2/users/me", self.base_url);

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", token))
            .call();

        match response {
            Ok(resp) => {
                let body = resp
                    .into_string()
                    .context("Failed to read user response body")?;
                let user_response: UserMeResponse =
                    serde_json::from_str(&body).context("Failed to parse user response")?;
                Ok(user_response.data.id)
            }
            Err(ureq::Error::Status(code, resp)) => {
                let error_body = resp
                    .into_string()
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(ClassifiedError::from_status_code(code, error_body).into())
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(ClassifiedError::timeout(format!("Network error: {}", transport)).into())
            }
        }
    }

    /// Create a tweet (optionally as a reply)
    pub fn create_tweet(&self, text: &str, reply_to: Option<&str>) -> Result<Tweet> {
        let token = self.get_bearer_token()?;
        let url = format!("{}/2/tweets", self.base_url);

        let request_body = CreateTweetRequest {
            text: text.to_string(),
            reply: reply_to.map(|id| ReplySettings {
                in_reply_to_tweet_id: id.to_string(),
            }),
        };

        let request_json =
            serde_json::to_string(&request_body).context("Failed to serialize request body")?;

        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", token))
            .set("Content-Type", "application/json")
            .send_string(&request_json);

        match response {
            Ok(resp) => {
                let body = resp.into_string().context("Failed to read response body")?;
                let create_response: CreateTweetResponse =
                    serde_json::from_str(&body).context("Failed to parse create tweet response")?;

                let mut tweet = Tweet::new(create_response.data.id);
                tweet.text = Some(create_response.data.text);
                Ok(tweet)
            }
            Err(ureq::Error::Status(code, resp)) => {
                let error_body = resp
                    .into_string()
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(ClassifiedError::from_status_code(code, error_body).into())
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(ClassifiedError::timeout(format!("Network error: {}", transport)).into())
            }
        }
    }

    /// Like a tweet
    pub fn like_tweet(&self, user_id: &str, tweet_id: &str) -> Result<bool> {
        let token = self.get_bearer_token()?;
        let url = format!("{}/2/users/{}/likes", self.base_url, user_id);

        let request_body = EngagementRequest {
            tweet_id: tweet_id.to_string(),
        };

        let request_json =
            serde_json::to_string(&request_body).context("Failed to serialize like request")?;

        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", token))
            .set("Content-Type", "application/json")
            .send_string(&request_json);

        match response {
            Ok(resp) => {
                let body = resp
                    .into_string()
                    .context("Failed to read like response body")?;
                let _engagement_response: EngagementResponse =
                    serde_json::from_str(&body).context("Failed to parse like response")?;
                Ok(true)
            }
            Err(ureq::Error::Status(code, resp)) => {
                let error_body = resp
                    .into_string()
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(ClassifiedError::from_status_code(code, error_body).into())
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(ClassifiedError::timeout(format!("Network error: {}", transport)).into())
            }
        }
    }

    /// Unlike a tweet
    pub fn unlike_tweet(&self, user_id: &str, tweet_id: &str) -> Result<bool> {
        let token = self.get_bearer_token()?;
        let url = format!("{}/2/users/{}/likes/{}", self.base_url, user_id, tweet_id);

        let response = ureq::delete(&url)
            .set("Authorization", &format!("Bearer {}", token))
            .call();

        match response {
            Ok(_) => Ok(true),
            Err(ureq::Error::Status(code, resp)) => {
                let error_body = resp
                    .into_string()
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(ClassifiedError::from_status_code(code, error_body).into())
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(ClassifiedError::timeout(format!("Network error: {}", transport)).into())
            }
        }
    }

    /// Retweet a tweet
    pub fn retweet(&self, user_id: &str, tweet_id: &str) -> Result<bool> {
        let token = self.get_bearer_token()?;
        let url = format!("{}/2/users/{}/retweets", self.base_url, user_id);

        let request_body = EngagementRequest {
            tweet_id: tweet_id.to_string(),
        };

        let request_json =
            serde_json::to_string(&request_body).context("Failed to serialize retweet request")?;

        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", token))
            .set("Content-Type", "application/json")
            .send_string(&request_json);

        match response {
            Ok(resp) => {
                let body = resp
                    .into_string()
                    .context("Failed to read retweet response body")?;
                let _engagement_response: EngagementResponse =
                    serde_json::from_str(&body).context("Failed to parse retweet response")?;
                Ok(true)
            }
            Err(ureq::Error::Status(code, resp)) => {
                let error_body = resp
                    .into_string()
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(ClassifiedError::from_status_code(code, error_body).into())
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(ClassifiedError::timeout(format!("Network error: {}", transport)).into())
            }
        }
    }

    /// Unretweet a tweet
    pub fn unretweet(&self, user_id: &str, tweet_id: &str) -> Result<bool> {
        let token = self.get_bearer_token()?;
        let url = format!(
            "{}/2/users/{}/retweets/{}",
            self.base_url, user_id, tweet_id
        );

        let response = ureq::delete(&url)
            .set("Authorization", &format!("Bearer {}", token))
            .call();

        match response {
            Ok(_) => Ok(true),
            Err(ureq::Error::Status(code, resp)) => {
                let error_body = resp
                    .into_string()
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(ClassifiedError::from_status_code(code, error_body).into())
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(ClassifiedError::timeout(format!("Network error: {}", transport)).into())
            }
        }
    }

    /// Add a bookmark
    pub fn add_bookmark(&self, user_id: &str, tweet_id: &str) -> Result<bool> {
        let token = self.get_bearer_token()?;
        let url = format!("{}/2/users/{}/bookmarks", self.base_url, user_id);

        let request_body = EngagementRequest {
            tweet_id: tweet_id.to_string(),
        };

        let request_json =
            serde_json::to_string(&request_body).context("Failed to serialize bookmark request")?;

        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", token))
            .set("Content-Type", "application/json")
            .send_string(&request_json);

        match response {
            Ok(_) => Ok(true),
            Err(ureq::Error::Status(code, resp)) => {
                let error_body = resp
                    .into_string()
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(ClassifiedError::from_status_code(code, error_body).into())
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(ClassifiedError::timeout(format!("Network error: {}", transport)).into())
            }
        }
    }

    /// Remove a bookmark
    pub fn remove_bookmark(&self, user_id: &str, tweet_id: &str) -> Result<bool> {
        let token = self.get_bearer_token()?;
        let url = format!(
            "{}/2/users/{}/bookmarks/{}",
            self.base_url, user_id, tweet_id
        );

        let response = ureq::delete(&url)
            .set("Authorization", &format!("Bearer {}", token))
            .call();

        match response {
            Ok(_) => Ok(true),
            Err(ureq::Error::Status(code, resp)) => {
                let error_body = resp
                    .into_string()
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(ClassifiedError::from_status_code(code, error_body).into())
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(ClassifiedError::timeout(format!("Network error: {}", transport)).into())
            }
        }
    }
}

impl Default for XApiClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tweet_request_serialization() {
        let request = CreateTweetRequest {
            text: "Hello world".to_string(),
            reply: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Hello world"));
        assert!(!json.contains("reply"));
    }

    #[test]
    fn test_create_tweet_request_with_reply_serialization() {
        let request = CreateTweetRequest {
            text: "Hello reply".to_string(),
            reply: Some(ReplySettings {
                in_reply_to_tweet_id: "123".to_string(),
            }),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Hello reply"));
        assert!(json.contains("in_reply_to_tweet_id"));
        assert!(json.contains("123"));
    }

    #[test]
    fn test_create_tweet_response_deserialization() {
        let json = r#"{"data":{"id":"1234567890","text":"Hello world"}}"#;
        let response: CreateTweetResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.id, "1234567890");
        assert_eq!(response.data.text, "Hello world");
    }
}
