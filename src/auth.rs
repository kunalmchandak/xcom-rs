use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Authentication status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub authenticated: bool,
    #[serde(rename = "authMode", skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    #[serde(rename = "nextSteps", skip_serializing_if = "Option::is_none")]
    pub next_steps: Option<Vec<String>>,
}

impl AuthStatus {
    /// Create an unauthenticated status with next steps
    pub fn unauthenticated(next_steps: Vec<String>) -> Self {
        Self {
            authenticated: false,
            auth_mode: None,
            scopes: None,
            next_steps: Some(next_steps),
        }
    }

    /// Create an authenticated status
    pub fn authenticated(auth_mode: String, scopes: Vec<String>) -> Self {
        Self {
            authenticated: true,
            auth_mode: Some(auth_mode),
            scopes: Some(scopes),
            next_steps: None,
        }
    }
}

/// Authentication token data for export/import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "tokenType")]
    pub token_type: String,
    #[serde(rename = "expiresAt", skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    pub scopes: Vec<String>,
}

/// Compare two JSON strings for equality by parsing and re-serializing
/// This handles key ordering differences
fn json_content_equal(json1: &str, json2: &str) -> bool {
    match (
        serde_json::from_str::<serde_json::Value>(json1),
        serde_json::from_str::<serde_json::Value>(json2),
    ) {
        (Ok(v1), Ok(v2)) => v1 == v2,
        _ => false, // If parsing fails, treat as different
    }
}

/// In-memory auth store for testing/stub implementation
#[derive(Debug, Clone)]
pub struct AuthStore {
    token: Option<AuthToken>,
    storage_path: Option<PathBuf>,
}

impl AuthStore {
    /// Create a new empty auth store
    pub fn new() -> Self {
        Self {
            token: None,
            storage_path: None,
        }
    }

    /// Create an auth store with persistent storage at the given path
    pub fn with_storage(path: PathBuf) -> Result<Self> {
        let mut store = Self {
            token: None,
            storage_path: Some(path.clone()),
        };

        // Try to load existing token from storage
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(token) = serde_json::from_str::<AuthToken>(&content) {
                    store.token = Some(token);
                }
            }
        }

        Ok(store)
    }

    /// Get default storage path: ~/.config/xcom-rs/auth.json
    pub fn default_storage_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| anyhow::anyhow!("Could not determine home directory"))?;
        let config_dir = PathBuf::from(home).join(".config").join("xcom-rs");
        std::fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join("auth.json"))
    }

    /// Create an auth store with default storage location
    pub fn with_default_storage() -> Result<Self> {
        Self::with_storage(Self::default_storage_path()?)
    }

    /// Save the current token to persistent storage
    /// Only writes if the content has changed (prevents unnecessary file modifications)
    fn save_to_storage(&self) -> Result<()> {
        if let Some(path) = &self.storage_path {
            if let Some(token) = &self.token {
                let new_json = serde_json::to_string_pretty(token)?;

                // Check if file exists and compare content
                let should_write = if path.exists() {
                    match std::fs::read_to_string(path) {
                        Ok(existing_json) => {
                            // Compare normalized JSON to handle key ordering differences
                            !json_content_equal(&existing_json, &new_json)
                        }
                        Err(_) => true, // If we can't read, write anyway
                    }
                } else {
                    true // File doesn't exist, write it
                };

                if should_write {
                    std::fs::write(path, new_json)?;
                }
            } else {
                // If no token, delete the storage file
                if path.exists() {
                    std::fs::remove_file(path)?;
                }
            }
        }
        Ok(())
    }

    /// Get current authentication status
    pub fn status(&self) -> AuthStatus {
        match &self.token {
            Some(token) => {
                // Check if token is expired
                if let Some(expires_at) = token.expires_at {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    if now >= expires_at {
                        return AuthStatus::unauthenticated(vec![
                            "Token expired. Run 'xcom-rs auth login' to re-authenticate"
                                .to_string(),
                        ]);
                    }
                }
                AuthStatus::authenticated("bearer".to_string(), token.scopes.clone())
            }
            None => AuthStatus::unauthenticated(vec![
                "Not authenticated. Run 'xcom-rs auth login' to authenticate".to_string(),
            ]),
        }
    }

    /// Export authentication data (returns encrypted in real implementation)
    pub fn export(&self) -> Result<String> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No authentication data to export"))?;

        // In real implementation, this would encrypt the data
        // For now, we use base64 encoding as a placeholder
        let json = serde_json::to_string(token)?;
        Ok(base64::encode(json))
    }

    /// Import authentication data (expects encrypted data in real implementation)
    pub fn import(&mut self, data: &str) -> Result<()> {
        // In real implementation, this would decrypt the data
        // For now, we use base64 decoding as a placeholder
        let json =
            base64::decode(data).map_err(|e| anyhow::anyhow!("Invalid auth data format: {}", e))?;
        let json_str = String::from_utf8(json)
            .map_err(|e| anyhow::anyhow!("Invalid auth data encoding: {}", e))?;
        let token: AuthToken = serde_json::from_str(&json_str)
            .map_err(|e| anyhow::anyhow!("Invalid auth data structure: {}", e))?;

        self.token = Some(token);
        self.save_to_storage()?;
        Ok(())
    }

    /// Set a token (for testing)
    pub fn set_token(&mut self, token: AuthToken) {
        self.token = Some(token);
        let _ = self.save_to_storage(); // Ignore errors in test helper
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.status().authenticated
    }
}

impl Default for AuthStore {
    fn default() -> Self {
        Self::new()
    }
}

// Note: base64 crate is needed - add to Cargo.toml
// For now, we'll implement a simple base64 encode/decode
mod base64 {
    use anyhow::Result;

    pub fn encode(data: String) -> String {
        // Simple base64 encoding using standard library would require adding dependency
        // For stub implementation, we'll use a simple reversible encoding
        format!("STUB_B64_{}", data)
    }

    pub fn decode(data: &str) -> Result<Vec<u8>> {
        if let Some(stripped) = data.strip_prefix("STUB_B64_") {
            Ok(stripped.as_bytes().to_vec())
        } else {
            Err(anyhow::anyhow!("Invalid base64 format"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_status_unauthenticated() {
        let status = AuthStatus::unauthenticated(vec!["Login first".to_string()]);
        assert!(!status.authenticated);
        assert!(status.auth_mode.is_none());
        assert!(status.scopes.is_none());
        assert!(status.next_steps.is_some());
    }

    #[test]
    fn test_auth_status_authenticated() {
        let status = AuthStatus::authenticated(
            "bearer".to_string(),
            vec!["read".to_string(), "write".to_string()],
        );
        assert!(status.authenticated);
        assert_eq!(status.auth_mode, Some("bearer".to_string()));
        assert_eq!(
            status.scopes,
            Some(vec!["read".to_string(), "write".to_string()])
        );
        assert!(status.next_steps.is_none());
    }

    #[test]
    fn test_auth_store_default_unauthenticated() {
        let store = AuthStore::new();
        let status = store.status();
        assert!(!status.authenticated);
        assert!(status.next_steps.is_some());
    }

    #[test]
    fn test_auth_store_with_token() {
        let mut store = AuthStore::new();
        let token = AuthToken {
            access_token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: None,
            scopes: vec!["read".to_string()],
        };
        store.set_token(token);

        let status = store.status();
        assert!(status.authenticated);
        assert_eq!(status.auth_mode, Some("bearer".to_string()));
    }

    #[test]
    fn test_auth_export_import() {
        let mut store = AuthStore::new();
        let token = AuthToken {
            access_token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: None,
            scopes: vec!["read".to_string(), "write".to_string()],
        };
        store.set_token(token);

        // Export
        let exported = store.export().unwrap();
        assert!(!exported.is_empty());

        // Import into new store
        let mut new_store = AuthStore::new();
        new_store.import(&exported).unwrap();

        // Verify it matches
        let status = new_store.status();
        assert!(status.authenticated);
        assert_eq!(
            status.scopes,
            Some(vec!["read".to_string(), "write".to_string()])
        );
    }

    #[test]
    fn test_export_without_token() {
        let store = AuthStore::new();
        let result = store.export();
        assert!(result.is_err());
    }

    #[test]
    fn test_import_invalid_data() {
        let mut store = AuthStore::new();
        let result = store.import("invalid_data");
        assert!(result.is_err());
    }

    #[test]
    fn test_stable_writes_same_content() {
        // Test that writing the same token twice doesn't modify the file
        let test_dir =
            std::env::temp_dir().join(format!("auth-stable-test-{}", std::process::id()));
        std::fs::create_dir_all(&test_dir).unwrap();
        let test_path = test_dir.join("auth.json");

        let token = AuthToken {
            access_token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: None,
            scopes: vec!["read".to_string()],
        };

        // Create store and save token
        let mut store = AuthStore::with_storage(test_path.clone()).unwrap();
        store.set_token(token.clone());

        // Get first modification time
        let metadata1 = std::fs::metadata(&test_path).unwrap();
        let mtime1 = metadata1.modified().unwrap();

        // Wait to ensure timestamp would change if file was rewritten
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Save the same token again
        store.set_token(token);

        // Get second modification time
        let metadata2 = std::fs::metadata(&test_path).unwrap();
        let mtime2 = metadata2.modified().unwrap();

        // Timestamps should be identical
        assert_eq!(
            mtime1, mtime2,
            "File should not be rewritten when content is identical"
        );

        // Cleanup
        std::fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_stable_writes_different_content() {
        // Test that writing different tokens does modify the file
        let test_dir = std::env::temp_dir().join(format!("auth-diff-test-{}", std::process::id()));
        std::fs::create_dir_all(&test_dir).unwrap();
        let test_path = test_dir.join("auth.json");

        let token1 = AuthToken {
            access_token: "test_token_1".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: None,
            scopes: vec!["read".to_string()],
        };

        let token2 = AuthToken {
            access_token: "test_token_2".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: None,
            scopes: vec!["read".to_string()],
        };

        // Create store and save first token
        let mut store = AuthStore::with_storage(test_path.clone()).unwrap();
        store.set_token(token1);

        // Get first modification time
        let metadata1 = std::fs::metadata(&test_path).unwrap();
        let mtime1 = metadata1.modified().unwrap();

        // Wait to ensure timestamp would change
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Save a different token
        store.set_token(token2);

        // Get second modification time
        let metadata2 = std::fs::metadata(&test_path).unwrap();
        let mtime2 = metadata2.modified().unwrap();

        // Timestamps should be different
        assert_ne!(
            mtime1, mtime2,
            "File should be rewritten when content changes"
        );

        // Cleanup
        std::fs::remove_dir_all(&test_dir).ok();
    }
}
