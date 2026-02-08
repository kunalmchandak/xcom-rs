use std::process::Command;
use xcom_rs::context::ExecutionContext;
use xcom_rs::protocol::ErrorCode;

#[test]
fn test_non_interactive_context() {
    // Test that ExecutionContext properly handles non-interactive mode
    let ctx = ExecutionContext::new(true, Some("trace-test".to_string()), None, None, false);

    // Simulate a command that needs interaction
    let error = ctx.check_interaction_required(
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

    // Simulate a command that needs interaction
    let error = ctx.check_interaction_required(
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
