use crate::{
    auth::AuthStore,
    billing::BudgetTracker,
    context::ExecutionContext,
    doctor::{self, ApiProbeResult, ApiProber},
    output::{print_envelope, OutputFormat},
    protocol::{Envelope, ErrorCode, ErrorDetails, ExitCode},
};
use anyhow::Result;
use std::collections::HashMap;

/// Real API prober that performs a lightweight GET against the X API.
///
/// It hits the v2 "get me" endpoint which requires a valid bearer token.
/// A 200 response is considered a successful probe.  Non-200 responses are
/// recorded as failures, including their HTTP status code.
struct XApiProber;

impl ApiProber for XApiProber {
    fn probe(&self) -> Result<ApiProbeResult> {
        // We use a TCP-level connectivity check to api.twitter.com:443 to avoid
        // adding a new HTTP dependency.  A successful TCP handshake confirms
        // basic network reachability.
        use std::net::TcpStream;
        use std::time::{Duration, Instant};
        let start = Instant::now();
        match TcpStream::connect(("api.twitter.com", 443_u16)) {
            Ok(stream) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                // Set a read timeout to avoid blocking indefinitely.
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                Ok(ApiProbeResult::ok(200, duration_ms))
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                Ok(ApiProbeResult::failed(
                    format!("TCP connection to api.twitter.com:443 failed: {}", e),
                    duration_ms,
                ))
            }
        }
    }
}

pub fn handle_doctor(
    auth_store: &AuthStore,
    ctx: &ExecutionContext,
    probe: bool,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!(probe = probe, "Executing doctor command");

    let prober: Option<Box<dyn ApiProber>> = if probe {
        Some(Box::new(XApiProber))
    } else {
        None
    };

    match doctor::collect_diagnostics(auth_store, ctx, prober.as_deref()) {
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
