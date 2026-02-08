use crate::{
    auth::AuthStore,
    cli::AuthCommands,
    output::{print_envelope, OutputFormat},
    protocol::{Envelope, ErrorCode, ErrorDetails, ExitCode},
};
use anyhow::Result;
use std::collections::HashMap;

pub fn handle_auth(
    command: AuthCommands,
    auth_store: &mut AuthStore,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!("Executing auth command");
    match command {
        AuthCommands::Status => handle_status(auth_store, create_meta, output_format),
        AuthCommands::Export => handle_export(auth_store, create_meta, output_format),
        AuthCommands::Import { data, dry_run } => {
            handle_import(auth_store, data, dry_run, create_meta, output_format)
        }
    }
}

fn handle_status(
    auth_store: &AuthStore,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!("Executing auth status command");
    let status = auth_store.status();
    let envelope = if let Some(meta) = create_meta() {
        Envelope::success_with_meta("auth.status", status, meta)
    } else {
        Envelope::success("auth.status", status)
    };
    print_envelope(&envelope, output_format)
}

fn handle_export(
    auth_store: &AuthStore,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!("Executing auth export command");
    match auth_store.export() {
        Ok(data) => {
            #[derive(serde::Serialize)]
            struct ExportResult {
                data: String,
            }
            let result = ExportResult { data };
            let envelope = if let Some(meta) = create_meta() {
                Envelope::success_with_meta("auth.export", result, meta)
            } else {
                Envelope::success("auth.export", result)
            };
            print_envelope(&envelope, output_format)
        }
        Err(e) => {
            let error = ErrorDetails::auth_required(
                e.to_string(),
                vec![
                    "Authenticate first by running 'xcom-rs auth login'".to_string(),
                    "Or import existing credentials with 'xcom-rs auth import'".to_string(),
                ],
            );
            let envelope = if let Some(meta) = create_meta() {
                Envelope::<()>::error_with_meta("error", error, meta)
            } else {
                Envelope::<()>::error("error", error)
            };
            let _ = print_envelope(&envelope, output_format);
            std::process::exit(ExitCode::AuthenticationError.into());
        }
    }
}

fn handle_import(
    auth_store: &mut AuthStore,
    data: String,
    dry_run: bool,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!(dry_run = dry_run, "Executing auth import command");
    match auth_store.import_with_plan(&data, dry_run) {
        Ok(plan) => {
            // Check if the plan indicates failure
            if plan.action == crate::auth::ImportAction::Fail {
                let error = ErrorDetails::new(
                    ErrorCode::InvalidArgument,
                    plan.reason.unwrap_or_else(|| "Import failed".to_string()),
                );
                let envelope = if let Some(meta) = create_meta() {
                    Envelope::<()>::error_with_meta("error", error, meta)
                } else {
                    Envelope::<()>::error("error", error)
                };
                let _ = print_envelope(&envelope, output_format);
                std::process::exit(ExitCode::InvalidArgument.into());
            }

            // For dry-run, return the plan
            if dry_run {
                let envelope = if let Some(meta) = create_meta() {
                    Envelope::success_with_meta("auth.import", plan, meta)
                } else {
                    Envelope::success("auth.import", plan)
                };
                print_envelope(&envelope, output_format)
            } else {
                // For actual import, return the status
                let status = auth_store.status();
                let envelope = if let Some(meta) = create_meta() {
                    Envelope::success_with_meta("auth.import", status, meta)
                } else {
                    Envelope::success("auth.import", status)
                };
                print_envelope(&envelope, output_format)
            }
        }
        Err(e) => {
            let error = ErrorDetails::new(ErrorCode::InvalidArgument, e.to_string());
            let envelope = if let Some(meta) = create_meta() {
                Envelope::<()>::error_with_meta("error", error, meta)
            } else {
                Envelope::<()>::error("error", error)
            };
            let _ = print_envelope(&envelope, output_format);
            std::process::exit(ExitCode::InvalidArgument.into());
        }
    }
}
