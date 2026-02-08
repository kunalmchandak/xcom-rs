use std::process::Command;
use xcom_rs::context::{ExecutionContext, ExecutionPolicy};
use xcom_rs::protocol::ErrorCode;

#[test]
fn test_non_interactive_context() {
    // Test that ExecutionContext properly handles non-interactive mode
    let ctx = ExecutionContext::new(true, Some("trace-test".to_string()), None, None, false);
    let policy = ExecutionPolicy::new();

    // Simulate a command that needs interaction
    let error = policy.check_interaction_required(
        &ctx,
        "Authentication credentials needed",
        vec![
            "Run 'xcom-rs auth login' to authenticate".to_string(),
            "Or set XCOM_TOKEN environment variable".to_string(),
        ],
    );

    assert!(
        error.is_some(),
        "Should return error in non-interactive mode"
    );
    let err = error.unwrap();
    assert_eq!(err.code, ErrorCode::AuthRequired);
    assert_eq!(err.message, "Authentication credentials needed");
    assert!(!err.is_retryable);

    // Check that nextSteps are in details
    let details = err.details.expect("Should have details");
    assert!(details.contains_key("nextSteps"));
}

#[test]
fn test_interactive_context() {
    // Test that ExecutionContext allows interaction in interactive mode
    let ctx = ExecutionContext::new(false, None, None, None, false);
    let policy = ExecutionPolicy::new();

    // Simulate a command that needs interaction
    let error = policy.check_interaction_required(
        &ctx,
        "Authentication credentials needed",
        vec!["Run 'xcom-rs auth login' to authenticate".to_string()],
    );

    assert!(
        error.is_none(),
        "Should not return error in interactive mode"
    );
}

#[test]
fn test_demo_interactive_non_interactive_mode() {
    // Get the binary path from cargo-provided environment variable
    let bin_path = env!("CARGO_BIN_EXE_xcom-rs");

    // Test demo-interactive command with --non-interactive flag
    let output = Command::new(bin_path)
        .args(["demo-interactive", "--non-interactive", "--output", "json"])
        .output()
        .expect("Failed to execute command");

    // Should exit with code 3 (AuthenticationError)
    assert_eq!(
        output.status.code(),
        Some(3),
        "Should exit with code 3 for AUTH_REQUIRED"
    );

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Should return valid JSON");

    // Verify error structure
    assert_eq!(json["ok"], false, "ok should be false");
    assert_eq!(
        json["error"]["code"], "auth_required",
        "Should return auth_required error"
    );
    assert_eq!(
        json["error"]["isRetryable"], false,
        "auth_required should not be retryable"
    );

    // Verify nextSteps in details
    assert!(
        json["error"]["details"]["nextSteps"].is_array(),
        "Should have nextSteps in error details"
    );
    let next_steps = json["error"]["details"]["nextSteps"]
        .as_array()
        .expect("nextSteps should be array");
    assert!(!next_steps.is_empty(), "nextSteps should not be empty");
}

#[test]
fn test_demo_interactive_interactive_mode() {
    // Get the binary path from cargo-provided environment variable
    let bin_path = env!("CARGO_BIN_EXE_xcom-rs");

    // Test demo-interactive command without --non-interactive flag (should succeed)
    let output = Command::new(bin_path)
        .args(["demo-interactive", "--output", "json"])
        .output()
        .expect("Failed to execute command");

    // Should exit with code 0 (Success)
    assert_eq!(
        output.status.code(),
        Some(0),
        "Should exit with code 0 in interactive mode"
    );

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Should return valid JSON");

    // Verify success structure
    assert_eq!(json["ok"], true, "ok should be true");
    assert_eq!(
        json["data"]["confirmed"], true,
        "Should have confirmed field"
    );
}

#[test]
fn test_invalid_log_format() {
    // Get the binary path from cargo-provided environment variable
    let bin_path = env!("CARGO_BIN_EXE_xcom-rs");

    // Test with invalid --log-format value
    let output = Command::new(bin_path)
        .args(["--log-format", "invalid", "commands"])
        .output()
        .expect("Failed to execute command");

    // Should exit with code 2 (InvalidArgument)
    assert_eq!(
        output.status.code(),
        Some(2),
        "Should exit with code 2 for invalid argument"
    );

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Should return valid JSON");

    // Verify error structure
    assert_eq!(json["ok"], false, "ok should be false");
    assert_eq!(
        json["error"]["code"], "invalid_argument",
        "Should return invalid_argument error"
    );

    // Verify error message contains the invalid value
    let message = json["error"]["message"]
        .as_str()
        .expect("message should be a string");
    assert!(
        message.contains("invalid") || message.contains("Invalid"),
        "Error message should mention the invalid value"
    );
}

#[test]
fn test_valid_log_format_json() {
    // Get the binary path from cargo-provided environment variable
    let bin_path = env!("CARGO_BIN_EXE_xcom-rs");

    // Test with valid --log-format=json (need --output=json to get JSON on stdout)
    let output = Command::new(bin_path)
        .args(["--log-format", "json", "--output", "json", "commands"])
        .output()
        .expect("Failed to execute command");

    // Should exit with code 0 (Success)
    assert_eq!(
        output.status.code(),
        Some(0),
        "Should exit with code 0 for valid log format"
    );

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Should return valid JSON");

    // Verify success structure
    assert_eq!(json["ok"], true, "ok should be true");

    // Verify logs are in JSON format on stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    // JSON logs should have structured format like {"timestamp":"...","level":"..."}
    // If stderr is not empty, it should be valid JSON lines
    if !stderr.trim().is_empty() {
        for line in stderr.lines() {
            if !line.trim().is_empty() {
                assert!(
                    serde_json::from_str::<serde_json::Value>(line).is_ok(),
                    "Stderr should contain valid JSON when --log-format=json"
                );
            }
        }
    }
}

#[test]
fn test_valid_log_format_text() {
    // Get the binary path from cargo-provided environment variable
    let bin_path = env!("CARGO_BIN_EXE_xcom-rs");

    // Test with valid --log-format=text (need --output=json to get JSON on stdout)
    let output = Command::new(bin_path)
        .args(["--log-format", "text", "--output", "json", "commands"])
        .output()
        .expect("Failed to execute command");

    // Should exit with code 0 (Success)
    assert_eq!(
        output.status.code(),
        Some(0),
        "Should exit with code 0 for valid log format"
    );

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Should return valid JSON");

    // Verify success structure
    assert_eq!(json["ok"], true, "ok should be true");

    // With text format, stderr should be plain text (not JSON)
    // We just verify it doesn't crash
}
