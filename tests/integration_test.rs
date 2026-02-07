use std::process::Command;
use xcom_rs::context::ExecutionContext;
use xcom_rs::protocol::ErrorCode;

#[test]
fn test_non_interactive_context() {
    // Test that ExecutionContext properly handles non-interactive mode
    let ctx = ExecutionContext::new(true, Some("trace-test".to_string()));

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
    assert_eq!(err.code, ErrorCode::InteractionRequired);
    assert_eq!(err.message, "Authentication credentials needed");
    assert!(!err.is_retryable);

    // Check that nextSteps are in details
    let details = err.details.expect("Should have details");
    assert!(details.contains_key("nextSteps"));
}

#[test]
fn test_interactive_context() {
    // Test that ExecutionContext allows interaction in interactive mode
    let ctx = ExecutionContext::new(false, None);

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
    // Build the binary first
    let build_status = Command::new("cargo")
        .args(["build", "--release"])
        .status()
        .expect("Failed to build binary");
    assert!(build_status.success(), "Build failed");

    // Test demo-interactive command with --non-interactive flag
    let output = Command::new("./target/release/xcom-rs")
        .args(["demo-interactive", "--non-interactive", "--output", "json"])
        .output()
        .expect("Failed to execute command");

    // Should exit with code 4 (OperationFailed)
    assert_eq!(
        output.status.code(),
        Some(4),
        "Should exit with code 4 for INTERACTION_REQUIRED"
    );

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Should return valid JSON");

    // Verify error structure
    assert_eq!(json["ok"], false, "ok should be false");
    assert_eq!(
        json["error"]["code"], "INTERACTION_REQUIRED",
        "Should return INTERACTION_REQUIRED error"
    );
    assert_eq!(
        json["error"]["isRetryable"], false,
        "INTERACTION_REQUIRED should not be retryable"
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
    // Build the binary first
    let build_status = Command::new("cargo")
        .args(["build", "--release"])
        .status()
        .expect("Failed to build binary");
    assert!(build_status.success(), "Build failed");

    // Test demo-interactive command without --non-interactive flag (should succeed)
    let output = Command::new("./target/release/xcom-rs")
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
