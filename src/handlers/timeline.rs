use crate::{
    cli::TimelineCommands,
    output::{print_envelope, print_ndjson, OutputFormat},
    protocol::{Envelope, ErrorCode, ErrorDetails, ExitCode},
    timeline::{
        commands::TimelineError,
        models::{TimelineArgs, TimelineKind},
        HttpTimelineClient, TimelineCommand,
    },
};
use anyhow::Result;
use std::collections::HashMap;

/// Handle timeline subcommands.
pub fn handle_timeline(
    command: TimelineCommands,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    let timeline_cmd = match TimelineCommand::<HttpTimelineClient>::new() {
        Ok(cmd) => cmd,
        Err(e) => {
            let error = build_error_details(&e);
            let envelope = if let Some(meta) = create_meta() {
                Envelope::<()>::error_with_meta("error", error, meta)
            } else {
                Envelope::<()>::error("error", error)
            };
            let _ = print_envelope(&envelope, output_format);
            std::process::exit(ExitCode::AuthenticationError.into());
        }
    };

    let (args, response_type) = match command {
        TimelineCommands::Home { limit, cursor } => (
            TimelineArgs {
                kind: TimelineKind::Home,
                limit,
                cursor,
            },
            "timeline.home",
        ),
        TimelineCommands::Mentions { limit, cursor } => (
            TimelineArgs {
                kind: TimelineKind::Mentions,
                limit,
                cursor,
            },
            "timeline.mentions",
        ),
        TimelineCommands::User {
            handle,
            limit,
            cursor,
        } => (
            TimelineArgs {
                kind: TimelineKind::User { handle },
                limit,
                cursor,
            },
            "timeline.user",
        ),
    };

    tracing::info!(response_type = %response_type, "Fetching timeline");

    match timeline_cmd.get(args) {
        Ok(result) => {
            if output_format == OutputFormat::Ndjson {
                print_ndjson(&result.tweets)
            } else {
                let envelope = if let Some(meta) = create_meta() {
                    Envelope::success_with_meta(response_type, result, meta)
                } else {
                    Envelope::success(response_type, result)
                };
                print_envelope(&envelope, output_format)
            }
        }
        Err(e) => {
            let error = build_error_details(&e);

            let envelope = if let Some(meta) = create_meta() {
                Envelope::<()>::error_with_meta("error", error, meta)
            } else {
                Envelope::<()>::error("error", error)
            };
            let _ = print_envelope(&envelope, output_format);

            let exit_code: i32 = match &e {
                TimelineError::AuthRequired => ExitCode::AuthenticationError.into(),
                TimelineError::ApiError(_) => ExitCode::OperationFailed.into(),
            };
            std::process::exit(exit_code);
        }
    }
}

fn build_error_details(e: &TimelineError) -> ErrorDetails {
    match e {
        TimelineError::AuthRequired => ErrorDetails::auth_required(
            e.to_string(),
            vec!["Set XCOM_RS_BEARER_TOKEN and re-run the command".to_string()],
        ),
        TimelineError::ApiError(classified) => {
            if let Some(retry_after_ms) = classified.retry_after_ms {
                ErrorDetails::with_retry_after(e.to_error_code(), e.to_string(), retry_after_ms)
            } else {
                ErrorDetails::new(e.to_error_code(), e.to_string())
            }
        }
    }
}

/// Convenience: build ErrorCode from TimelineError (for use in other modules)
impl From<&TimelineError> for ErrorCode {
    fn from(e: &TimelineError) -> Self {
        e.to_error_code()
    }
}
