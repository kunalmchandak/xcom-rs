//! Tweet show and conversation retrieval operations.

use anyhow::{Context, Result};

use crate::tweets::client::{fetch_conversation, TweetApiClient};
use crate::tweets::models::ConversationResult;

use super::types::{ConversationArgs, ShowArgs, ShowResult};

/// Show a single tweet by ID.
pub fn show(api_client: &dyn TweetApiClient, args: ShowArgs) -> Result<ShowResult> {
    let tweet = api_client
        .get_tweet(&args.tweet_id)
        .context("Failed to fetch tweet")?;
    Ok(ShowResult { tweet })
}

/// Retrieve a conversation tree starting from a tweet.
pub fn conversation(
    api_client: &dyn TweetApiClient,
    args: ConversationArgs,
) -> Result<ConversationResult> {
    fetch_conversation(api_client, &args.tweet_id).context("Failed to fetch conversation")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweets::client::MockTweetApiClient;

    fn create_fixture_client() -> MockTweetApiClient {
        MockTweetApiClient::with_conversation_fixture()
    }

    /// Characterization test: show returns the tweet with correct id and conversation_id
    #[test]
    fn test_show_returns_tweet() {
        let client = create_fixture_client();

        let args = ShowArgs {
            tweet_id: "tweet_root".to_string(),
        };

        let result = show(&client, args).unwrap();
        assert_eq!(result.tweet.id, "tweet_root");
        assert_eq!(
            result.tweet.conversation_id,
            Some("conv_root_001".to_string())
        );
    }

    /// Characterization test: show with nonexistent tweet returns error
    #[test]
    fn test_show_not_found() {
        let client = create_fixture_client();

        let args = ShowArgs {
            tweet_id: "nonexistent_tweet".to_string(),
        };

        let result = show(&client, args);
        assert!(result.is_err());
    }

    /// Characterization test: conversation returns a non-empty tree with correct posts and edges
    #[test]
    fn test_conversation_returns_tree() {
        let client = create_fixture_client();

        let args = ConversationArgs {
            tweet_id: "tweet_root".to_string(),
        };

        let result = conversation(&client, args).unwrap();
        assert!(!result.posts.is_empty());
        assert!(result.posts.iter().any(|t| t.id == "tweet_root"));
        assert!(!result.edges.is_empty());
        assert!(
            !result.conversation_id.is_empty(),
            "conversation_id should be present"
        );
        assert_eq!(result.conversation_id, "conv_root_001");
    }

    /// Characterization test: conversation edges connect parent/child tweets correctly
    #[test]
    fn test_conversation_edges_structure() {
        let client = create_fixture_client();

        let args = ConversationArgs {
            tweet_id: "tweet_root".to_string(),
        };

        let result = conversation(&client, args).unwrap();
        let root_edge = result
            .edges
            .iter()
            .find(|e| e.parent_id == "tweet_root" && e.child_id == "tweet_reply1");
        assert!(root_edge.is_some(), "Expected edge from root to reply1");

        let reply_edge = result
            .edges
            .iter()
            .find(|e| e.parent_id == "tweet_reply1" && e.child_id == "tweet_reply2");
        assert!(reply_edge.is_some(), "Expected edge from reply1 to reply2");
    }
}
