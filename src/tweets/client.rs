//! X API client interface and mock implementation for tweet operations.

use anyhow::Result;

use super::models::{ConversationEdge, ConversationResult, ReferencedTweet, Tweet};

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

    Ok(ConversationResult { posts, edges })
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
