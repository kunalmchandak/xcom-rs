use crate::{
    auth::AuthStore,
    billing::BudgetTracker,
    context::ExecutionContext,
    doctor,
    output::{print_envelope, OutputFormat},
    protocol::{Envelope, ErrorCode, ErrorDetails, ExitCode},
};
use anyhow::Result;
use std::collections::HashMap;

pub fn handle_doctor(
    auth_store: &AuthStore,
    ctx: &ExecutionContext,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!("Executing doctor command");
    match doctor::collect_diagnostics(auth_store, ctx) {
        Ok(diagnostics) => {
            let envelope = if let Some(meta) = create_meta() {
                Envelope::success_with_meta("doctor", diagnostics, meta)
            } else {
                Envelope::success("doctor", diagnostics)
            };
            print_envelope(&envelope, output_format)
        }
        Err(e) => {
            // If diagnostics collection fails completely, return error with next steps
            let mut next_steps = vec![
                "Check that configuration directories are accessible".to_string(),
                "Verify file permissions for auth and budget storage locations".to_string(),
            ];

            // Try to provide specific paths even if collection failed
            if let Ok(auth_path) = AuthStore::default_storage_path() {
                next_steps.push(format!("Auth storage: {}", auth_path.display()));
            }
            if let Ok(budget_path) = BudgetTracker::default_storage_path() {
                next_steps.push(format!("Budget storage: {}", budget_path.display()));
            }

            let mut details = HashMap::new();
            details.insert("nextSteps".to_string(), serde_json::json!(next_steps));

            let error = ErrorDetails::with_details(
                ErrorCode::InternalError,
                format!("Failed to collect diagnostics: {}", e),
                details,
            );
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
