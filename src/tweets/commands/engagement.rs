//! Tweet engagement operations: like, unlike, retweet, unretweet.

use anyhow::Result;

use super::types::{EngagementArgs, EngagementResult};
use crate::tweets::http_client::XApiClient;

/// Like a tweet.
pub fn like(http_client: &XApiClient, args: EngagementArgs) -> Result<EngagementResult> {
    // Get user ID
    let user_id = http_client.get_user_id()?;

    // Like the tweet
    let success = http_client.like_tweet(&user_id, &args.tweet_id)?;

    Ok(EngagementResult {
        tweet_id: args.tweet_id,
        success,
    })
}

/// Unlike a tweet.
pub fn unlike(http_client: &XApiClient, args: EngagementArgs) -> Result<EngagementResult> {
    // Get user ID
    let user_id = http_client.get_user_id()?;

    // Unlike the tweet
    let success = http_client.unlike_tweet(&user_id, &args.tweet_id)?;

    Ok(EngagementResult {
        tweet_id: args.tweet_id,
        success,
    })
}

/// Retweet a tweet.
pub fn retweet(http_client: &XApiClient, args: EngagementArgs) -> Result<EngagementResult> {
    // Get user ID
    let user_id = http_client.get_user_id()?;

    // Retweet the tweet
    let success = http_client.retweet(&user_id, &args.tweet_id)?;

    Ok(EngagementResult {
        tweet_id: args.tweet_id,
        success,
    })
}

/// Unretweet a tweet.
pub fn unretweet(http_client: &XApiClient, args: EngagementArgs) -> Result<EngagementResult> {
    // Get user ID
    let user_id = http_client.get_user_id()?;

    // Unretweet the tweet
    let success = http_client.unretweet(&user_id, &args.tweet_id)?;

    Ok(EngagementResult {
        tweet_id: args.tweet_id,
        success,
    })
}
