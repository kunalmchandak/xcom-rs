/// Integration tests for XDG paths support
/// These tests verify that XDG_CONFIG_HOME and XDG_DATA_HOME are respected
use std::process::Command;
use xcom_rs::test_utils::helpers::assert_success_json;

#[test]
#[cfg_attr(not(feature = "env-tests"), ignore)]
fn test_auth_storage_respects_xdg_data_home() {
    // Create a temporary test directory
    let test_dir =
        std::env::temp_dir().join(format!("xcom-rs-xdg-auth-test-{}", std::process::id()));
    let xdg_data = test_dir.join("data");
    std::fs::create_dir_all(&xdg_data).expect("Failed to create test directory");

    // Import some auth data with XDG_DATA_HOME set
    let test_token_data = "STUB_B64_{\"accessToken\":\"test_token_xdg\",\"tokenType\":\"Bearer\",\"expiresAt\":null,\"scopes\":[\"read\"]}";
    let import_output = Command::new("cargo")
        .env("HOME", &test_dir)
        .env("XDG_DATA_HOME", &xdg_data)
        .args([
            "run",
            "--",
            "auth",
            "import",
            test_token_data,
            "--output",
            "json",
        ])
        .output()
        .expect("Failed to execute auth import");

    assert_success_json(&import_output);

    // Verify the auth file was created in XDG_DATA_HOME
    let expected_path = xdg_data.join("xcom-rs").join("auth.json");
    assert!(
        expected_path.exists(),
        "Auth file should be created in XDG_DATA_HOME/xcom-rs/auth.json"
    );

    // Verify we can read it back with XDG_DATA_HOME set
    let status_output = Command::new("cargo")
        .env("HOME", &test_dir)
        .env("XDG_DATA_HOME", &xdg_data)
        .args(["run", "--", "auth", "status", "--output", "json"])
        .output()
        .expect("Failed to execute auth status");

    let status_json = assert_success_json(&status_output);

    assert_eq!(status_json["ok"], true);
    assert_eq!(
        status_json["data"]["authenticated"], true,
        "Should be authenticated after import"
    );

    // Cleanup
    std::fs::remove_dir_all(&test_dir).ok();
}

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

#[test]
#[cfg_attr(not(feature = "env-tests"), ignore)]
fn test_fallback_to_default_path_without_xdg() {
    // Create a temporary test directory
    let test_dir =
        std::env::temp_dir().join(format!("xcom-rs-fallback-test-{}", std::process::id()));
    std::fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    // Run auth import WITHOUT XDG environment variables
    let test_token_data = "STUB_B64_{\"accessToken\":\"test_fallback\",\"tokenType\":\"Bearer\",\"expiresAt\":null,\"scopes\":[\"read\"]}";
    let import_output = Command::new("cargo")
        .env("HOME", &test_dir)
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .args([
            "run",
            "--",
            "auth",
            "import",
            test_token_data,
            "--output",
            "json",
        ])
        .output()
        .expect("Failed to execute auth import");

    assert_success_json(&import_output);

    // Verify the auth file was created in HOME/.local/share/xcom-rs/
    let expected_path = test_dir
        .join(".local")
        .join("share")
        .join("xcom-rs")
        .join("auth.json");
    assert!(
        expected_path.exists(),
        "Auth file should be created in HOME/.local/share/xcom-rs/auth.json when XDG_DATA_HOME is not set"
    );

    // Cleanup
    std::fs::remove_dir_all(&test_dir).ok();
}
