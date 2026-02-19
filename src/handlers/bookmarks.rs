use crate::{
    bookmarks::{BookmarkArgs, BookmarkCommand, BookmarkListArgs},
    cli::BookmarksCommands,
    output::{print_envelope, print_ndjson, OutputFormat},
    protocol::{Envelope, ErrorCode, ErrorDetails, ExitCode},
};
use anyhow::Result;
use std::collections::HashMap;

pub fn handle_bookmarks(
    command: BookmarksCommands,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    let bookmark_cmd = BookmarkCommand::new();

    match command {
        BookmarksCommands::Add { tweet_id } => {
            handle_bookmark_add(bookmark_cmd, tweet_id, create_meta, output_format)
        }
        BookmarksCommands::Remove { tweet_id } => {
            handle_bookmark_remove(bookmark_cmd, tweet_id, create_meta, output_format)
        }
        BookmarksCommands::List { limit, cursor } => {
            handle_bookmark_list(bookmark_cmd, limit, cursor, create_meta, output_format)
        }
    }
}

fn handle_bookmark_add(
    bookmark_cmd: BookmarkCommand,
    tweet_id: String,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!(tweet_id = %tweet_id, "Adding tweet to bookmarks");

    let args = BookmarkArgs {
        tweet_id: tweet_id.clone(),
    };

    match bookmark_cmd.add(args) {
        Ok(result) => {
            let envelope = if let Some(meta) = create_meta() {
                Envelope::success_with_meta("bookmarks.add", result, meta)
            } else {
                Envelope::success("bookmarks.add", result)
            };
            print_envelope(&envelope, output_format)
        }
        Err(e) => {
            let error = ErrorDetails::new(ErrorCode::InternalError, e.to_string());
            let envelope = if let Some(meta) = create_meta() {
                Envelope::<()>::error_with_meta("error", error, meta)
            } else {
                Envelope::<()>::error("error", error)
            };
            let _ = print_envelope(&envelope, output_format);
            std::process::exit(ExitCode::OperationFailed.into());
        }
    }
}

fn handle_bookmark_remove(
    bookmark_cmd: BookmarkCommand,
    tweet_id: String,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!(tweet_id = %tweet_id, "Removing tweet from bookmarks");

    let args = BookmarkArgs {
        tweet_id: tweet_id.clone(),
    };

    match bookmark_cmd.remove(args) {
        Ok(result) => {
            let envelope = if let Some(meta) = create_meta() {
                Envelope::success_with_meta("bookmarks.remove", result, meta)
            } else {
                Envelope::success("bookmarks.remove", result)
            };
            print_envelope(&envelope, output_format)
        }
        Err(e) => {
            let error = ErrorDetails::new(ErrorCode::InternalError, e.to_string());
            let envelope = if let Some(meta) = create_meta() {
                Envelope::<()>::error_with_meta("error", error, meta)
            } else {
                Envelope::<()>::error("error", error)
            };
            let _ = print_envelope(&envelope, output_format);
            std::process::exit(ExitCode::OperationFailed.into());
        }
    }
}

fn handle_bookmark_list(
    bookmark_cmd: BookmarkCommand,
    limit: Option<usize>,
    cursor: Option<String>,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!("Listing bookmarks");

    let args = BookmarkListArgs { limit, cursor };

    match bookmark_cmd.list(args) {
        Ok(result) => {
            if output_format == OutputFormat::Ndjson {
                print_ndjson(&result.tweets)
            } else {
                let envelope = if let Some(meta) = create_meta() {
                    Envelope::success_with_meta("bookmarks.list", result, meta)
                } else {
                    Envelope::success("bookmarks.list", result)
                };
                print_envelope(&envelope, output_format)
            }
        }
        Err(e) => {
            let error = ErrorDetails::new(ErrorCode::InternalError, e.to_string());
            let envelope = if let Some(meta) = create_meta() {
                Envelope::<()>::error_with_meta("error", error, meta)
            } else {
                Envelope::<()>::error("error", error)
            };
            let _ = print_envelope(&envelope, output_format);
            std::process::exit(ExitCode::OperationFailed.into());
        }
    }
}
