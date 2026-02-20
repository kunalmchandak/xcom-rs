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
struct XApiProber {
    bearer_token: Option<String>,
}

impl XApiProber {
    fn new() -> Self {
        Self {
            bearer_token: std::env::var("XCOM_RS_BEARER_TOKEN").ok().map(|v| {
                if let Some(stripped) = v.strip_prefix("Bearer ") {
                    stripped.to_string()
                } else {
                    v
                }
            }),
        }
    }
}

impl ApiProber for XApiProber {
    fn probe(&self) -> Result<ApiProbeResult> {
        use std::time::Instant;

        // If no bearer token is set, return a skipped result with next steps
        let bearer_token = match &self.bearer_token {
            Some(token) if !token.is_empty() => token,
            _ => {
                return Ok(ApiProbeResult {
                    status: doctor::ProbeStatus::Skipped,
                    duration_ms: 0,
                    http_status: None,
                    message: Some(
                        "Authentication not configured; set XCOM_RS_BEARER_TOKEN to enable probe"
                            .to_string(),
                    ),
                });
            }
        };

        let start = Instant::now();

        // Perform HTTP GET to the X API v2 "users/me" endpoint
        let url = "https://api.twitter.com/2/users/me";
        let request = ureq::get(url)
            .set("Authorization", &format!("Bearer {}", bearer_token))
            .timeout(std::time::Duration::from_secs(10));

        match request.call() {
            Ok(response) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let status_code = response.status();
                if status_code == 200 {
                    Ok(ApiProbeResult::ok(status_code, duration_ms))
                } else {
                    Ok(ApiProbeResult::failed_with_status(
                        status_code,
                        format!("API returned non-200 status: {}", status_code),
                        duration_ms,
                    ))
                }
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                match e {
                    ureq::Error::Status(code, _) => {
                        // HTTP error with status code
                        Ok(ApiProbeResult::failed_with_status(
                            code,
                            format!("API returned error status: {}", code),
                            duration_ms,
                        ))
                    }
                    ureq::Error::Transport(transport) => {
                        // Network/connection error
                        Ok(ApiProbeResult::failed(
                            format!("Network error: {}", transport),
                            duration_ms,
                        ))
                    }
                }
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
        Some(Box::new(XApiProber::new()))
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
                "Verify file permissions for budget storage location".to_string(),
            ];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xapi_prober_no_bearer_token() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_RS_BEARER_TOKEN");

        let prober = XApiProber::new();
        let result = prober.probe();
        assert!(result.is_ok());
        let probe_result = result.unwrap();
        assert_eq!(probe_result.status, doctor::ProbeStatus::Skipped);
        assert_eq!(probe_result.duration_ms, 0);
        assert!(probe_result
            .message
            .as_ref()
            .unwrap()
            .contains("XCOM_RS_BEARER_TOKEN"));
    }

    #[test]
    fn test_xapi_prober_with_mock_server_success() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // Create a mock server
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/2/users/me")
            .match_header("authorization", "Bearer test_token_123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data":{"id":"12345","name":"test"}}"#)
            .create();

        std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token_123");

        // Create a custom prober that uses the mock server URL
        struct MockServerProber {
            url: String,
            bearer_token: Option<String>,
        }

        impl ApiProber for MockServerProber {
            fn probe(&self) -> Result<ApiProbeResult> {
                use std::time::Instant;
                let bearer_token = match &self.bearer_token {
                    Some(token) if !token.is_empty() => token,
                    _ => {
                        return Ok(ApiProbeResult {
                            status: doctor::ProbeStatus::Skipped,
                            duration_ms: 0,
                            http_status: None,
                            message: Some("Authentication not configured".to_string()),
                        });
                    }
                };

                let start = Instant::now();
                let request = ureq::get(&self.url)
                    .set("Authorization", &format!("Bearer {}", bearer_token))
                    .timeout(std::time::Duration::from_secs(10));

                match request.call() {
                    Ok(response) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        let status_code = response.status();
                        if status_code == 200 {
                            Ok(ApiProbeResult::ok(status_code, duration_ms))
                        } else {
                            Ok(ApiProbeResult::failed_with_status(
                                status_code,
                                format!("API returned non-200 status: {}", status_code),
                                duration_ms,
                            ))
                        }
                    }
                    Err(e) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        match e {
                            ureq::Error::Status(code, _) => Ok(ApiProbeResult::failed_with_status(
                                code,
                                format!("API returned error status: {}", code),
                                duration_ms,
                            )),
                            ureq::Error::Transport(transport) => Ok(ApiProbeResult::failed(
                                format!("Network error: {}", transport),
                                duration_ms,
                            )),
                        }
                    }
                }
            }
        }

        let prober = MockServerProber {
            url: format!("{}/2/users/me", server.url()),
            bearer_token: Some("test_token_123".to_string()),
        };

        let result = prober.probe();
        assert!(result.is_ok());
        let probe_result = result.unwrap();
        assert_eq!(probe_result.status, doctor::ProbeStatus::Ok);
        assert_eq!(probe_result.http_status, Some(200));

        mock.assert();
    }

    #[test]
    fn test_xapi_prober_with_mock_server_auth_error() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // Create a mock server that returns 401
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/2/users/me")
            .match_header("authorization", "Bearer invalid_token")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":[{"message":"Unauthorized"}]}"#)
            .create();

        std::env::set_var("XCOM_RS_BEARER_TOKEN", "invalid_token");

        // Create a custom prober that uses the mock server URL
        struct MockServerProber {
            url: String,
            bearer_token: Option<String>,
        }

        impl ApiProber for MockServerProber {
            fn probe(&self) -> Result<ApiProbeResult> {
                use std::time::Instant;
                let bearer_token = match &self.bearer_token {
                    Some(token) if !token.is_empty() => token,
                    _ => {
                        return Ok(ApiProbeResult {
                            status: doctor::ProbeStatus::Skipped,
                            duration_ms: 0,
                            http_status: None,
                            message: Some("Authentication not configured".to_string()),
                        });
                    }
                };

                let start = Instant::now();
                let request = ureq::get(&self.url)
                    .set("Authorization", &format!("Bearer {}", bearer_token))
                    .timeout(std::time::Duration::from_secs(10));

                match request.call() {
                    Ok(response) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        let status_code = response.status();
                        if status_code == 200 {
                            Ok(ApiProbeResult::ok(status_code, duration_ms))
                        } else {
                            Ok(ApiProbeResult::failed_with_status(
                                status_code,
                                format!("API returned non-200 status: {}", status_code),
                                duration_ms,
                            ))
                        }
                    }
                    Err(e) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        match e {
                            ureq::Error::Status(code, _) => Ok(ApiProbeResult::failed_with_status(
                                code,
                                format!("API returned error status: {}", code),
                                duration_ms,
                            )),
                            ureq::Error::Transport(transport) => Ok(ApiProbeResult::failed(
                                format!("Network error: {}", transport),
                                duration_ms,
                            )),
                        }
                    }
                }
            }
        }

        let prober = MockServerProber {
            url: format!("{}/2/users/me", server.url()),
            bearer_token: Some("invalid_token".to_string()),
        };

        let result = prober.probe();
        assert!(result.is_ok());
        let probe_result = result.unwrap();
        assert_eq!(probe_result.status, doctor::ProbeStatus::Failed);
        assert_eq!(probe_result.http_status, Some(401));
        assert!(probe_result
            .message
            .as_ref()
            .unwrap()
            .contains("error status"));

        mock.assert();
    }

    #[test]
    fn test_xapi_prober_with_mock_server_network_error() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");

        // Create a custom prober that uses an invalid URL to simulate network error
        struct MockServerProber {
            url: String,
            bearer_token: Option<String>,
        }

        impl ApiProber for MockServerProber {
            fn probe(&self) -> Result<ApiProbeResult> {
                use std::time::Instant;
                let bearer_token = match &self.bearer_token {
                    Some(token) if !token.is_empty() => token,
                    _ => {
                        return Ok(ApiProbeResult {
                            status: doctor::ProbeStatus::Skipped,
                            duration_ms: 0,
                            http_status: None,
                            message: Some("Authentication not configured".to_string()),
                        });
                    }
                };

                let start = Instant::now();
                let request = ureq::get(&self.url)
                    .set("Authorization", &format!("Bearer {}", bearer_token))
                    .timeout(std::time::Duration::from_millis(100));

                match request.call() {
                    Ok(response) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        let status_code = response.status();
                        if status_code == 200 {
                            Ok(ApiProbeResult::ok(status_code, duration_ms))
                        } else {
                            Ok(ApiProbeResult::failed_with_status(
                                status_code,
                                format!("API returned non-200 status: {}", status_code),
                                duration_ms,
                            ))
                        }
                    }
                    Err(e) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        match e {
                            ureq::Error::Status(code, _) => Ok(ApiProbeResult::failed_with_status(
                                code,
                                format!("API returned error status: {}", code),
                                duration_ms,
                            )),
                            ureq::Error::Transport(transport) => Ok(ApiProbeResult::failed(
                                format!("Network error: {}", transport),
                                duration_ms,
                            )),
                        }
                    }
                }
            }
        }

        let prober = MockServerProber {
            url: "http://invalid-host-that-does-not-exist-12345.com/api".to_string(),
            bearer_token: Some("test_token".to_string()),
        };

        let result = prober.probe();
        assert!(result.is_ok());
        let probe_result = result.unwrap();
        assert_eq!(probe_result.status, doctor::ProbeStatus::Failed);
        assert!(probe_result.http_status.is_none());
        assert!(probe_result
            .message
            .as_ref()
            .unwrap()
            .contains("Network error"));
    }
}
