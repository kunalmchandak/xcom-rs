use crate::{
    cli::MediaCommands,
    media::{MediaCommand, StubMediaClient, UploadArgs},
    output::{print_envelope, OutputFormat},
    protocol::{Envelope, ErrorCode, ErrorDetails, ExitCode},
};
use anyhow::Result;
use std::collections::HashMap;

pub fn handle_media(
    command: MediaCommands,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    match command {
        MediaCommands::Upload { path } => handle_upload(path, create_meta, output_format),
    }
}

fn handle_upload(
    path: String,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!(path = %path, "Uploading media");

    let client = StubMediaClient;
    let cmd = MediaCommand::new(client);
    let args = UploadArgs { path: path.clone() };

    match cmd.upload(args) {
        Ok(result) => {
            let envelope = if let Some(meta) = create_meta() {
                Envelope::success_with_meta("media.upload", result, meta)
            } else {
                Envelope::success("media.upload", result)
            };
            print_envelope(&envelope, output_format)
        }
        Err(e) => {
            // Classify the error based on message prefix
            let error_code = if e.to_string().contains("InvalidInput") {
                ErrorCode::InvalidArgument
            } else if e.to_string().contains("AuthRequired") {
                ErrorCode::AuthorizationFailed
            } else {
                ErrorCode::InternalError
            };

            let error = ErrorDetails::new(error_code, e.to_string());
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
