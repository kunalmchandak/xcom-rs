/// Integration tests for XDG paths support
/// These tests verify that XDG_CONFIG_HOME and XDG_DATA_HOME are respected
use std::process::Command;

#[test]
fn test_auth_storage_respects_xdg_config_home() {
    // Create a temporary test directory
    let test_dir =
        std::env::temp_dir().join(format!("xcom-rs-xdg-config-test-{}", std::process::id()));
    let xdg_config = test_dir.join("config");
    std::fs::create_dir_all(&xdg_config).expect("Failed to create test directory");

    // Import some auth data with XDG_CONFIG_HOME set
    let test_token_data = "STUB_B64_{\"accessToken\":\"test_token_xdg\",\"tokenType\":\"Bearer\",\"expiresAt\":null,\"scopes\":[\"read\"]}";
    let import_output = Command::new("cargo")
        .env("HOME", &test_dir)
        .env("XDG_CONFIG_HOME", &xdg_config)
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

    assert!(import_output.status.success(), "Auth import should succeed");

    // Verify the auth file was created in XDG_CONFIG_HOME
    let expected_path = xdg_config.join("xcom-rs").join("auth.json");
    assert!(
        expected_path.exists(),
        "Auth file should be created in XDG_CONFIG_HOME/xcom-rs/auth.json"
    );

    // Verify we can read it back with XDG_CONFIG_HOME set
    let status_output = Command::new("cargo")
        .env("HOME", &test_dir)
        .env("XDG_CONFIG_HOME", &xdg_config)
        .args(["run", "--", "auth", "status", "--output", "json"])
        .output()
        .expect("Failed to execute auth status");

    assert!(status_output.status.success(), "Auth status should succeed");

    let status_stdout = String::from_utf8_lossy(&status_output.stdout);
    let status_json: serde_json::Value =
        serde_json::from_str(&status_stdout).expect("Auth status should return valid JSON");

    assert_eq!(status_json["ok"], true);
    assert_eq!(
        status_json["data"]["authenticated"], true,
        "Should be authenticated after import"
    );

    // Cleanup
    std::fs::remove_dir_all(&test_dir).ok();
}

#[test]
fn test_billing_storage_respects_xdg_data_home() {
    // Create a temporary test directory
    let test_dir =
        std::env::temp_dir().join(format!("xcom-rs-xdg-data-test-{}", std::process::id()));
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

    assert!(
        estimate_output.status.success(),
        "Billing estimate should succeed"
    );

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

    assert!(import_output.status.success(), "Auth import should succeed");

    // Verify the auth file was created in HOME/.config/xcom-rs/
    let expected_path = test_dir.join(".config").join("xcom-rs").join("auth.json");
    assert!(
        expected_path.exists(),
        "Auth file should be created in HOME/.config/xcom-rs/auth.json when XDG_CONFIG_HOME is not set"
    );

    // Cleanup
    std::fs::remove_dir_all(&test_dir).ok();
}
