//! Tweets command handlers organized by feature.
//!
//! Each sub-module contains argument types, result types, and the implementation
//! for a specific feature area. All public types are re-exported from this module
//! for backward compatibility.

pub mod create;
pub mod engagement;
pub mod list;
pub mod show;
pub mod thread;
pub mod types;

// Re-export all public types for backward compatibility
pub use types::{
    ClassifiedError, ConversationArgs, CreateArgs, CreateResult, EngagementArgs, EngagementResult,
    ErrorKind, IdempotencyConflictError, IfExistsPolicy, ListArgs, ListResult, ListResultMeta,
    PaginationMeta, ReplyArgs, ReplyResult, ShowArgs, ShowResult, ThreadArgs, ThreadMeta,
    ThreadPartialFailureError, ThreadResult,
};

use crate::tweets::{
    client::TweetApiClient, http_client::XApiClient, ledger::IdempotencyLedger,
    models::ConversationResult,
};
use anyhow::Result;

/// Main tweets command handler.
///
/// Delegates to feature-specific modules for each operation while providing
/// a unified entry point for the CLI.
pub struct TweetCommand {
    ledger: IdempotencyLedger,
    api_client: Box<dyn TweetApiClient>,
    http_client: XApiClient,
}

impl TweetCommand {
    /// Create a new tweet command handler with a custom API client
    pub fn with_client(ledger: IdempotencyLedger, client: Box<dyn TweetApiClient>) -> Self {
        Self {
            ledger,
            api_client: client,
            http_client: XApiClient::new(),
        }
    }

    /// Create a tweet with idempotency support
    pub fn create(&self, args: CreateArgs) -> Result<CreateResult> {
        create::create(&self.ledger, &self.http_client, args)
    }

    /// Like a tweet
    pub fn like(&self, args: EngagementArgs) -> Result<EngagementResult> {
        engagement::like(&self.http_client, args)
    }

    /// Unlike a tweet
    pub fn unlike(&self, args: EngagementArgs) -> Result<EngagementResult> {
        engagement::unlike(&self.http_client, args)
    }

    /// Retweet a tweet
    pub fn retweet(&self, args: EngagementArgs) -> Result<EngagementResult> {
        engagement::retweet(&self.http_client, args)
    }

    /// Unretweet a tweet
    pub fn unretweet(&self, args: EngagementArgs) -> Result<EngagementResult> {
        engagement::unretweet(&self.http_client, args)
    }

    /// List tweets with field projection and pagination
    pub fn list(&self, args: ListArgs) -> Result<ListResult> {
        list::list_with_client(self.api_client.as_ref(), args)
    }

    /// Reply to a tweet with idempotency support
    pub fn reply(&self, args: ReplyArgs) -> Result<ReplyResult> {
        thread::reply(&self.ledger, &self.http_client, args)
    }

    /// Post a thread of tweets (sequential replies)
    pub fn thread(&self, args: ThreadArgs) -> Result<ThreadResult> {
        thread::thread(&self.ledger, &self.http_client, args)
    }

    /// Show a single tweet by ID
    pub fn show(&self, args: ShowArgs) -> Result<ShowResult> {
        show::show(self.api_client.as_ref(), args)
    }

    /// Retrieve a conversation tree starting from a tweet
    pub fn conversation(&self, args: ConversationArgs) -> Result<ConversationResult> {
        show::conversation(self.api_client.as_ref(), args)
    }
}
