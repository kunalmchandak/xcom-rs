//! Media upload module for the X API.
//!
//! Provides `media upload <path>` which posts a file to `POST /2/media/upload`
//! and returns the `media_id` needed to attach the media to a tweet.

pub mod commands;
pub mod models;

pub use commands::{MediaCommand, StubMediaClient, UploadArgs, XMediaClient};
pub use models::UploadResult;
