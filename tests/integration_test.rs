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
