/// Search operations for tweets and users
pub mod commands;
pub mod models;

pub use commands::{
    HttpSearchClient, SearchClient, SearchCommand, SearchRecentArgs, SearchUsersArgs,
};
pub use models::{SearchRecentResult, SearchTweet, SearchUser, SearchUsersResult};

// Mock client is only available in tests
#[cfg(test)]
pub use commands::MockSearchClient;
