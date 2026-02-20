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
