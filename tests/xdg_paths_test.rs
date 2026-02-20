/// Integration tests for XDG paths support
/// These tests verify that XDG_DATA_HOME is respected for budget tracking
mod common;

use common::test_utils::helpers::assert_success_json;
use std::process::Command;

#[test]
#[cfg_attr(not(feature = "env-tests"), ignore)]
fn test_billing_storage_respects_xdg_data_home() {
    // Create a temporary test directory
    let test_dir =
        std::env::temp_dir().join(format!("xcom-rs-xdg-billing-test-{}", std::process::id()));
    let xdg_data = test_dir.join("data");
    std::fs::create_dir_all(&xdg_data).expect("Failed to create test directory");

    // Run billing estimate with XDG_DATA_HOME set and budget tracking
    let estimate_output = Command::new("cargo")
        .env("HOME", &test_dir)
        .env("XDG_DATA_HOME", &xdg_data)
        .args([
            "run",
            "--",
            "billing",
            "estimate",
            "tweets.create",
            "--text",
            "test",
            "--budget-daily-credits",
            "100",
            "--output",
            "json",
        ])
        .output()
        .expect("Failed to execute billing estimate");

    assert_success_json(&estimate_output);

    // Verify the budget file was created in XDG_DATA_HOME
    let expected_path = xdg_data.join("xcom-rs").join("budget.json");
    assert!(
        expected_path.exists(),
        "Budget file should be created in XDG_DATA_HOME/xcom-rs/budget.json"
    );

    // Verify the file content is valid JSON
    let content = std::fs::read_to_string(&expected_path).expect("Failed to read budget file");
    let _: serde_json::Value =
        serde_json::from_str(&content).expect("Budget file should contain valid JSON");

    // Cleanup
    std::fs::remove_dir_all(&test_dir).ok();
}
