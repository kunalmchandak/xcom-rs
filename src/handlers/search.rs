use crate::{
    cli::SearchCommands,
    output::{print_envelope, print_ndjson, OutputFormat},
    protocol::{Envelope, ErrorCode, ErrorDetails, ExitCode},
    search::{SearchCommand, SearchRecentArgs, SearchUsersArgs},
};
use anyhow::Result;
use std::collections::HashMap;

pub fn handle_search(
    command: SearchCommands,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    let search_cmd = SearchCommand::new();

    match command {
        SearchCommands::Recent {
            query,
            limit,
            cursor,
        } => handle_search_recent(search_cmd, query, limit, cursor, create_meta, output_format),
        SearchCommands::Users {
            query,
            limit,
            cursor,
        } => handle_search_users(search_cmd, query, limit, cursor, create_meta, output_format),
    }
}

fn handle_search_recent(
    search_cmd: SearchCommand,
    query: String,
    limit: Option<usize>,
    cursor: Option<String>,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!(query = %query, "Searching recent tweets");

    let args = SearchRecentArgs {
        query,
        limit,
        cursor,
    };

    match search_cmd.search_recent(args) {
        Ok(result) => {
            if output_format == OutputFormat::Ndjson {
                print_ndjson(&result.tweets)
            } else {
                let envelope = if let Some(meta) = create_meta() {
                    Envelope::success_with_meta("search.recent", result, meta)
                } else {
                    Envelope::success("search.recent", result)
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

fn handle_search_users(
    search_cmd: SearchCommand,
    query: String,
    limit: Option<usize>,
    cursor: Option<String>,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!(query = %query, "Searching users");

    let args = SearchUsersArgs {
        query,
        limit,
        cursor,
    };

    match search_cmd.search_users(args) {
        Ok(result) => {
            if output_format == OutputFormat::Ndjson {
                print_ndjson(&result.users)
            } else {
                let envelope = if let Some(meta) = create_meta() {
                    Envelope::success_with_meta("search.users", result, meta)
                } else {
                    Envelope::success("search.users", result)
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
