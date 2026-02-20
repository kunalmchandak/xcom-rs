//! Tweet listing with field projection and pagination.

use anyhow::Result;

use crate::tweets::client::TweetApiClient;

use super::types::{ListArgs, ListResult};

/// List tweets with field projection and pagination using the provided client.
pub fn list_with_client(client: &dyn TweetApiClient, args: ListArgs) -> Result<ListResult> {
    client.list_tweets(&args)
}
