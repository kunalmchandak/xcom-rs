//! X API client interface and mock implementation for tweet operations.

use anyhow::{Context, Result};
use serde::Deserialize;

use super::commands::types::{ListArgs, ListResult};
use super::models::{ConversationEdge, ConversationResult, ReferencedTweet, Tweet};

/// X API v2 response structures
#[derive(Debug, Deserialize)]
struct ApiTweetResponse {
    data: Option<ApiTweetData>,
}

#[derive(Debug, Deserialize)]
struct ApiTweetsResponse {
    data: Option<Vec<ApiTweetData>>,
    meta: Option<ApiMetaWithPagination>,
}

#[derive(Debug, Deserialize)]
struct ApiTweetData {
    id: String,
    text: Option<String>,
    author_id: Option<String>,
    created_at: Option<String>,
    conversation_id: Option<String>,
    in_reply_to_user_id: Option<String>,
    referenced_tweets: Option<Vec<ApiReferencedTweet>>,
}

#[derive(Debug, Deserialize)]
struct ApiReferencedTweet {
    #[serde(rename = "type")]
    ref_type: String,
    id: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ApiMetaWithPagination {
    result_count: Option<usize>,
    next_token: Option<String>,
}

/// Trait representing the X API client interface for tweet operations.
/// This allows mocking in tests without real API calls.
pub trait TweetApiClient: Send + Sync {
    /// Create a tweet, optionally as a reply to another tweet.
    /// Returns the created tweet.
    fn post_tweet(&self, text: &str, reply_to: Option<&str>) -> Result<Tweet>;

    /// Fetch a single tweet by ID.
    fn get_tweet(&self, tweet_id: &str) -> Result<Tweet>;

    /// Search recent tweets matching a query.
    /// Returns a list of matching tweets.
    fn search_recent(&self, query: &str, limit: usize) -> Result<Vec<Tweet>>;

    /// List tweets for the authenticated user with field projection and pagination
    fn list_tweets(&self, args: &ListArgs) -> Result<ListResult>;
}

/// HTTP-based implementation of TweetApiClient using X API
pub struct HttpTweetApiClient {
    bearer_token: String,
}

impl HttpTweetApiClient {
    /// Create a new HTTP tweet API client with the given bearer token
    pub fn new(bearer_token: String) -> Self {
        Self { bearer_token }
    }

    /// Create from environment variable (XCOM_RS_BEARER_TOKEN)
    pub fn from_env() -> Result<Self> {
        let auth_store = crate::auth::storage::AuthStore::new();
        let status = auth_store.status();
        if !status.authenticated {
            anyhow::bail!(
                "Authentication required. Set XCOM_RS_BEARER_TOKEN environment variable."
            );
        }
        let bearer_token =
            std::env::var("XCOM_RS_BEARER_TOKEN").context("XCOM_RS_BEARER_TOKEN not set")?;
        Ok(Self::new(bearer_token))
    }

    /// Resolve authenticated user ID
    fn resolve_me(&self) -> Result<String> {
        // Allow test override
        if let Ok(user_id) = std::env::var("XCOM_TEST_USER_ID") {
            return Ok(user_id);
        }

        // Call X API to get authenticated user
        let url = "https://api.twitter.com/2/users/me";

        let response = ureq::get(url)
            .set("Authorization", &format!("Bearer {}", self.bearer_token))
            .call();

        let api_response: serde_json::Value = match response {
            Ok(resp) => resp
                .into_json()
                .context("Failed to parse user/me response")?,
            Err(ureq::Error::Status(code, _)) if code == 401 || code == 403 => {
                anyhow::bail!("Authentication required");
            }
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                anyhow::bail!("X API error {}: {}", code, body);
            }
            Err(e) => {
                anyhow::bail!("Failed to get user/me: {}", e);
            }
        };

        let user_id = api_response["data"]["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to extract user ID from response"))?
            .to_string();

        Ok(user_id)
    }

    /// Convert API tweet data to Tweet model
    fn api_tweet_to_tweet(api_tweet: ApiTweetData) -> Tweet {
        let mut tweet = Tweet::new(api_tweet.id);
        tweet.text = api_tweet.text;
        tweet.author_id = api_tweet.author_id;
        tweet.created_at = api_tweet.created_at;
        tweet.conversation_id = api_tweet.conversation_id;
        tweet.in_reply_to_user_id = api_tweet.in_reply_to_user_id;
        tweet.referenced_tweets = api_tweet.referenced_tweets.map(|refs| {
            refs.into_iter()
                .map(|r| ReferencedTweet {
                    ref_type: r.ref_type,
                    id: r.id,
                })
                .collect()
        });
        tweet
    }
}

impl TweetApiClient for HttpTweetApiClient {
    fn post_tweet(&self, text: &str, reply_to: Option<&str>) -> Result<Tweet> {
        let url = "https://api.twitter.com/2/tweets";
        let mut body = serde_json::json!({
            "text": text,
        });

        if let Some(reply_id) = reply_to {
            body["reply"] = serde_json::json!({
                "in_reply_to_tweet_id": reply_id,
            });
        }

        let response = ureq::post(url)
            .set("Authorization", &format!("Bearer {}", self.bearer_token))
            .set("Content-Type", "application/json")
            .send_json(&body);

        let api_response: ApiTweetResponse = match response {
            Ok(resp) => resp.into_json().context("Failed to parse tweet response")?,
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                anyhow::bail!("X API error {}: {}", code, body);
            }
            Err(e) => anyhow::bail!("Failed to post tweet: {}", e),
        };

        let api_tweet = api_response
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in tweet response"))?;

        Ok(Self::api_tweet_to_tweet(api_tweet))
    }

    fn get_tweet(&self, tweet_id: &str) -> Result<Tweet> {
        let url = format!(
            "https://api.twitter.com/2/tweets/{}?tweet.fields=id,text,author_id,created_at,conversation_id,in_reply_to_user_id,referenced_tweets",
            tweet_id
        );

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.bearer_token))
            .call();

        let api_response: ApiTweetResponse = match response {
            Ok(resp) => resp.into_json().context("Failed to parse tweet response")?,
            Err(ureq::Error::Status(404, _)) => {
                anyhow::bail!("Tweet not found: {}", tweet_id);
            }
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                anyhow::bail!("X API error {}: {}", code, body);
            }
            Err(e) => anyhow::bail!("Failed to get tweet: {}", e),
        };

        let api_tweet = api_response
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in tweet response"))?;

        Ok(Self::api_tweet_to_tweet(api_tweet))
    }

    fn search_recent(&self, query: &str, limit: usize) -> Result<Vec<Tweet>> {
        let url = format!(
            "https://api.twitter.com/2/tweets/search/recent?query={}&max_results={}&tweet.fields=id,text,author_id,created_at,conversation_id,in_reply_to_user_id,referenced_tweets",
            urlencoding::encode(query),
            limit
        );

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.bearer_token))
            .call();

        let api_response: ApiTweetsResponse = match response {
            Ok(resp) => resp
                .into_json()
                .context("Failed to parse search response")?,
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                anyhow::bail!("X API error {}: {}", code, body);
            }
            Err(e) => anyhow::bail!("Failed to search tweets: {}", e),
        };

        let tweets = api_response
            .data
            .unwrap_or_default()
            .into_iter()
            .map(Self::api_tweet_to_tweet)
            .collect();

        Ok(tweets)
    }

    fn list_tweets(&self, args: &ListArgs) -> Result<ListResult> {
        use super::commands::types::{ListResultMeta, PaginationMeta};

        let user_id = self.resolve_me()?;

        // Build field list from requested fields
        let field_strings: Vec<String> =
            args.fields.iter().map(|f| f.as_str().to_string()).collect();
        let fields_param = field_strings.join(",");

        let mut url = format!(
            "https://api.twitter.com/2/users/{}/tweets?max_results={}",
            user_id,
            args.limit.unwrap_or(10)
        );
        url.push_str(&format!(
            "&tweet.fields={}",
            urlencoding::encode(&fields_param)
        ));

        if let Some(cursor) = &args.cursor {
            url.push_str(&format!(
                "&pagination_token={}",
                urlencoding::encode(cursor)
            ));
        }

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.bearer_token))
            .call();

        let api_response: ApiTweetsResponse = match response {
            Ok(resp) => resp.into_json().context("Failed to parse list response")?,
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                anyhow::bail!("X API error {}: {}", code, body);
            }
            Err(e) => anyhow::bail!("Failed to list tweets: {}", e),
        };

        let tweets: Vec<Tweet> = api_response
            .data
            .unwrap_or_default()
            .into_iter()
            .map(Self::api_tweet_to_tweet)
            .collect();

        // Apply field projection
        let projected_tweets = tweets
            .into_iter()
            .map(|t| t.project(&args.fields))
            .collect();

        let meta = api_response.meta.map(|api_meta| ListResultMeta {
            pagination: PaginationMeta {
                next_cursor: api_meta.next_token,
                prev_cursor: None,
            },
        });

        Ok(ListResult {
            tweets: projected_tweets,
            meta,
        })
    }
}

/// Mock implementation of TweetApiClient for testing.
pub struct MockTweetApiClient {
    /// Pre-configured tweets to return from get_tweet
    pub tweets: std::collections::HashMap<String, Tweet>,
    /// Pre-configured search results
    pub search_results: Vec<Tweet>,
    /// Whether to simulate an error
    pub simulate_error: bool,
}

impl MockTweetApiClient {
    /// Create a new empty mock client
    pub fn new() -> Self {
        Self {
            tweets: std::collections::HashMap::new(),
            search_results: Vec::new(),
            simulate_error: false,
        }
    }

    /// Add a tweet to the mock store
    pub fn add_tweet(&mut self, tweet: Tweet) {
        self.tweets.insert(tweet.id.clone(), tweet);
    }

    /// Build a mock client with a conversation tree fixture.
    /// Returns a mock client containing a root tweet and replies, all sharing
    /// the same conversation_id.
    pub fn with_conversation_fixture() -> Self {
        let mut client = Self::new();

        let conversation_id = "conv_root_001".to_string();

        // Root tweet
        let mut root = Tweet::new("tweet_root".to_string());
        root.text = Some("Root tweet".to_string());
        root.author_id = Some("user_1".to_string());
        root.created_at = Some("2024-01-01T00:00:00Z".to_string());
        root.conversation_id = Some(conversation_id.clone());
        client.add_tweet(root);

        // First-level reply
        let mut reply1 = Tweet::new("tweet_reply1".to_string());
        reply1.text = Some("First reply".to_string());
        reply1.author_id = Some("user_2".to_string());
        reply1.created_at = Some("2024-01-01T00:01:00Z".to_string());
        reply1.conversation_id = Some(conversation_id.clone());
        reply1.referenced_tweets = Some(vec![ReferencedTweet {
            ref_type: "replied_to".to_string(),
            id: "tweet_root".to_string(),
        }]);
        client.add_tweet(reply1.clone());

        // Second-level reply
        let mut reply2 = Tweet::new("tweet_reply2".to_string());
        reply2.text = Some("Second reply (to reply1)".to_string());
        reply2.author_id = Some("user_3".to_string());
        reply2.created_at = Some("2024-01-01T00:02:00Z".to_string());
        reply2.conversation_id = Some(conversation_id.clone());
        reply2.referenced_tweets = Some(vec![ReferencedTweet {
            ref_type: "replied_to".to_string(),
            id: "tweet_reply1".to_string(),
        }]);
        client.add_tweet(reply2);

        // Populate search results (all tweets in the conversation)
        client.search_results = client.tweets.values().cloned().collect();

        client
    }
}

impl Default for MockTweetApiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TweetApiClient for MockTweetApiClient {
    fn post_tweet(&self, text: &str, reply_to: Option<&str>) -> Result<Tweet> {
        if self.simulate_error {
            return Err(anyhow::anyhow!("Simulated API error"));
        }
        let mut tweet = Tweet::new(format!("mock_tweet_{}", uuid::Uuid::new_v4()));
        tweet.text = Some(text.to_string());
        tweet.created_at = Some("2024-01-01T00:00:00Z".to_string());
        if let Some(reply_id) = reply_to {
            tweet.referenced_tweets = Some(vec![ReferencedTweet {
                ref_type: "replied_to".to_string(),
                id: reply_id.to_string(),
            }]);
        }
        Ok(tweet)
    }

    fn get_tweet(&self, tweet_id: &str) -> Result<Tweet> {
        if self.simulate_error {
            return Err(anyhow::anyhow!("Simulated API error"));
        }
        self.tweets
            .get(tweet_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Tweet not found: {}", tweet_id))
    }

    fn search_recent(&self, _query: &str, limit: usize) -> Result<Vec<Tweet>> {
        if self.simulate_error {
            return Err(anyhow::anyhow!("Simulated API error"));
        }
        Ok(self.search_results.iter().take(limit).cloned().collect())
    }

    fn list_tweets(&self, args: &ListArgs) -> Result<ListResult> {
        use super::commands::types::{ListResultMeta, PaginationMeta};

        if self.simulate_error {
            return Err(anyhow::anyhow!("Simulated API error"));
        }

        let limit = args.limit.unwrap_or(10);
        let offset = if let Some(cursor) = &args.cursor {
            cursor
                .strip_prefix("cursor_")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0)
        } else {
            0
        };

        let mut tweets = Vec::new();
        for i in offset..(offset + limit) {
            let mut tweet = Tweet::new(format!("tweet_{}", i));
            tweet.text = Some(format!("Tweet text {}", i));
            tweet.author_id = Some(format!("user_{}", i));
            tweet.created_at = Some("2024-01-01T00:00:00Z".to_string());

            // Apply field projection
            let projected = tweet.project(&args.fields);
            tweets.push(projected);
        }

        let next_cursor = if tweets.len() == limit {
            Some(format!("cursor_{}", offset + limit))
        } else {
            None
        };

        let prev_cursor = if offset > 0 {
            Some(format!("cursor_{}", offset.saturating_sub(limit)))
        } else {
            None
        };

        let meta = Some(ListResultMeta {
            pagination: PaginationMeta {
                next_cursor,
                prev_cursor,
            },
        });

        Ok(ListResult { tweets, meta })
    }
}

/// Build conversation edges from a list of tweets.
/// Edges connect parent tweets to their replies using referenced_tweets.
pub fn build_conversation_edges(tweets: &[Tweet]) -> Vec<ConversationEdge> {
    let mut edges = Vec::new();
    for tweet in tweets {
        if let Some(refs) = &tweet.referenced_tweets {
            for r in refs {
                if r.ref_type == "replied_to" {
                    edges.push(ConversationEdge {
                        parent_id: r.id.clone(),
                        child_id: tweet.id.clone(),
                    });
                }
            }
        }
    }
    edges
}

/// Fetch a conversation tree from the API client.
/// First fetches the root tweet to get its conversation_id, then searches for
/// all tweets in the conversation and builds the tree.
pub fn fetch_conversation(
    client: &dyn TweetApiClient,
    tweet_id: &str,
) -> Result<ConversationResult> {
    // Step 1: Get root tweet to obtain conversation_id
    let root_tweet = client.get_tweet(tweet_id)?;
    let conversation_id = root_tweet
        .conversation_id
        .clone()
        .unwrap_or_else(|| tweet_id.to_string());

    // Step 2: Search for all tweets in the conversation
    let query = format!("conversation_id:{}", conversation_id);
    let mut posts = client.search_recent(&query, 100)?;

    // Ensure the root tweet is included
    if !posts.iter().any(|t| t.id == root_tweet.id) {
        posts.insert(0, root_tweet);
    }

    // Sort by created_at for stable ordering
    posts.sort_by(|a, b| {
        let a_time = a.created_at.as_deref().unwrap_or("");
        let b_time = b.created_at.as_deref().unwrap_or("");
        a_time.cmp(b_time)
    });

    // Step 3: Build edges
    let edges = build_conversation_edges(&posts);

    Ok(ConversationResult {
        conversation_id,
        posts,
        edges,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_post_tweet_no_reply() {
        let client = MockTweetApiClient::new();
        let tweet = client.post_tweet("Hello world", None).unwrap();
        assert_eq!(tweet.text, Some("Hello world".to_string()));
        assert!(tweet.referenced_tweets.is_none());
    }

    #[test]
    fn test_mock_post_tweet_with_reply() {
        let client = MockTweetApiClient::new();
        let tweet = client
            .post_tweet("Hello reply", Some("parent_tweet_id"))
            .unwrap();
        assert!(tweet.referenced_tweets.is_some());
        let refs = tweet.referenced_tweets.unwrap();
        assert_eq!(refs[0].ref_type, "replied_to");
        assert_eq!(refs[0].id, "parent_tweet_id");
    }

    #[test]
    fn test_mock_get_tweet() {
        let client = MockTweetApiClient::with_conversation_fixture();
        let tweet = client.get_tweet("tweet_root").unwrap();
        assert_eq!(tweet.id, "tweet_root");
        assert_eq!(tweet.conversation_id, Some("conv_root_001".to_string()));
    }

    #[test]
    fn test_mock_get_tweet_not_found() {
        let client = MockTweetApiClient::new();
        let result = client.get_tweet("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_build_conversation_edges() {
        let client = MockTweetApiClient::with_conversation_fixture();
        let tweets: Vec<Tweet> = client.tweets.values().cloned().collect();
        let edges = build_conversation_edges(&tweets);
        // reply1 -> root and reply2 -> reply1
        assert!(edges.len() >= 2);
    }

    #[test]
    fn test_fetch_conversation() {
        let client = MockTweetApiClient::with_conversation_fixture();
        let result = fetch_conversation(&client, "tweet_root").unwrap();
        assert!(!result.posts.is_empty());
        assert!(result.posts.iter().any(|t| t.id == "tweet_root"));
        // Edges should exist for replies
        assert!(!result.edges.is_empty());
    }
}
