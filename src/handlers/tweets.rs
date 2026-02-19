use crate::{
    cli::TweetsCommands,
    output::{print_envelope, print_ndjson, OutputFormat},
    protocol::{Envelope, ErrorCode, ErrorDetails, ExitCode},
    tweets::{
        ClassifiedError, CreateArgs, EngagementArgs, IdempotencyConflictError, IdempotencyLedger,
        IfExistsPolicy, ListArgs, TweetCommand, TweetFields,
    },
};
use anyhow::Result;
use std::{collections::HashMap, str::FromStr};

pub fn handle_tweets(
    command: TweetsCommands,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    let ledger = IdempotencyLedger::new(None)
        .map_err(|e| anyhow::anyhow!("Failed to initialize idempotency ledger: {}", e))?;
    let tweet_cmd = TweetCommand::new(ledger);

    match command {
        TweetsCommands::Create {
            text,
            client_request_id,
            if_exists,
        } => handle_create(
            tweet_cmd,
            text,
            client_request_id,
            if_exists,
            create_meta,
            output_format,
        ),
        TweetsCommands::List {
            fields,
            limit,
            cursor,
        } => handle_list(tweet_cmd, fields, limit, cursor, create_meta, output_format),
        TweetsCommands::Like { tweet_id } => {
            handle_engagement(tweet_cmd, "like", tweet_id, create_meta, output_format)
        }
        TweetsCommands::Unlike { tweet_id } => {
            handle_engagement(tweet_cmd, "unlike", tweet_id, create_meta, output_format)
        }
        TweetsCommands::Retweet { tweet_id } => {
            handle_engagement(tweet_cmd, "retweet", tweet_id, create_meta, output_format)
        }
        TweetsCommands::Unretweet { tweet_id } => {
            handle_engagement(tweet_cmd, "unretweet", tweet_id, create_meta, output_format)
        }
    }
}

fn handle_engagement(
    tweet_cmd: TweetCommand,
    action: &str,
    tweet_id: String,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!(action = %action, tweet_id = %tweet_id, "Executing engagement action");

    let args = EngagementArgs {
        tweet_id: tweet_id.clone(),
    };

    let result = match action {
        "like" => tweet_cmd.like(args),
        "unlike" => tweet_cmd.unlike(args),
        "retweet" => tweet_cmd.retweet(args),
        "unretweet" => tweet_cmd.unretweet(args),
        _ => unreachable!("Unknown engagement action: {}", action),
    };

    let op_type = format!("tweets.{}", action);

    match result {
        Ok(engagement_result) => {
            let envelope = if let Some(meta) = create_meta() {
                Envelope::success_with_meta(&op_type, engagement_result, meta)
            } else {
                Envelope::success(&op_type, engagement_result)
            };
            print_envelope(&envelope, output_format)
        }
        Err(e) => {
            let error = if let Some(classified) = e.downcast_ref::<ClassifiedError>() {
                if let Some(retry_after_ms) = classified.retry_after_ms {
                    ErrorDetails::with_retry_after(
                        classified.to_error_code(),
                        e.to_string(),
                        retry_after_ms,
                    )
                } else {
                    ErrorDetails::new(classified.to_error_code(), e.to_string())
                }
            } else {
                ErrorDetails::new(ErrorCode::InternalError, e.to_string())
            };

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

fn handle_create(
    tweet_cmd: TweetCommand,
    text: String,
    client_request_id: Option<String>,
    if_exists: String,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!(text = %text, "Creating tweet");

    let if_exists_policy = IfExistsPolicy::from_str(&if_exists).unwrap_or_else(|e| {
        eprintln!("Invalid if-exists policy: {}", e);
        std::process::exit(ExitCode::InvalidArgument.into());
    });

    let args = CreateArgs {
        text,
        client_request_id,
        if_exists: if_exists_policy,
    };

    match tweet_cmd.create(args) {
        Ok(result) => {
            let envelope = if let Some(meta) = create_meta() {
                Envelope::success_with_meta("tweets.create", result, meta)
            } else {
                Envelope::success("tweets.create", result)
            };
            print_envelope(&envelope, output_format)
        }
        Err(e) => {
            let error = if e.downcast_ref::<IdempotencyConflictError>().is_some() {
                ErrorDetails::new(ErrorCode::IdempotencyConflict, e.to_string())
            } else if let Some(classified) = e.downcast_ref::<ClassifiedError>() {
                if let Some(retry_after_ms) = classified.retry_after_ms {
                    ErrorDetails::with_retry_after(
                        classified.to_error_code(),
                        e.to_string(),
                        retry_after_ms,
                    )
                } else {
                    ErrorDetails::new(classified.to_error_code(), e.to_string())
                }
            } else {
                ErrorDetails::new(ErrorCode::InternalError, e.to_string())
            };

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

fn handle_list(
    tweet_cmd: TweetCommand,
    fields: Option<String>,
    limit: Option<usize>,
    cursor: Option<String>,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!("Listing tweets");

    let field_list = if let Some(fields_str) = fields {
        fields_str
            .split(',')
            .filter_map(|s| TweetFields::parse(s.trim()))
            .collect()
    } else {
        TweetFields::default_fields()
    };

    let args = ListArgs {
        fields: field_list,
        limit,
        cursor,
    };

    match tweet_cmd.list(args) {
        Ok(result) => {
            if output_format == OutputFormat::Ndjson {
                print_ndjson(&result.tweets)
            } else {
                let envelope = if let Some(meta) = create_meta() {
                    Envelope::success_with_meta("tweets.list", result, meta)
                } else {
                    Envelope::success("tweets.list", result)
                };
                print_envelope(&envelope, output_format)
            }
        }
        Err(e) => {
            let error = if let Some(classified) = e.downcast_ref::<ClassifiedError>() {
                if let Some(retry_after_ms) = classified.retry_after_ms {
                    ErrorDetails::with_retry_after(
                        classified.to_error_code(),
                        e.to_string(),
                        retry_after_ms,
                    )
                } else {
                    ErrorDetails::new(classified.to_error_code(), e.to_string())
                }
            } else {
                ErrorDetails::new(ErrorCode::InternalError, e.to_string())
            };

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
