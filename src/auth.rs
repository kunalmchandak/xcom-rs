use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Action to be performed during import
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ImportAction {
    /// Create new authentication entry
    Create,
    /// Update existing authentication entry
    Update,
    /// Skip - no changes needed
    Skip,
    /// Failed with error
    Fail,
}

/// Plan for a single import operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPlan {
    /// Action to be performed
    pub action: ImportAction,
    /// Optional reason (required for Fail action)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Whether this is a dry-run
    #[serde(rename = "dryRun")]
    pub dry_run: bool,
}

impl ImportPlan {
    /// Create a plan for creating new auth
    pub fn create(dry_run: bool) -> Self {
        Self {
            action: ImportAction::Create,
            reason: None,
            dry_run,
        }
    }

    /// Create a plan for updating existing auth
    pub fn update(dry_run: bool) -> Self {
        Self {
            action: ImportAction::Update,
            reason: None,
            dry_run,
        }
    }

    /// Create a plan for skipping (no changes)
    pub fn skip(reason: String, dry_run: bool) -> Self {
        Self {
            action: ImportAction::Skip,
            reason: Some(reason),
            dry_run,
        }
    }

    /// Create a plan for failed import
    pub fn fail(reason: String, dry_run: bool) -> Self {
        Self {
            action: ImportAction::Fail,
            reason: Some(reason),
            dry_run,
        }
    }
}

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
    fn save_to_storage(&self) -> Result<()> {
        if let Some(path) = &self.storage_path {
            if let Some(token) = &self.token {
                let json = serde_json::to_string_pretty(token)?;
                std::fs::write(path, json)?;
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

    /// Import authentication data with dry-run support
    /// Returns an ImportPlan describing what would happen
    pub fn import_with_plan(&mut self, data: &str, dry_run: bool) -> Result<ImportPlan> {
        // Validate and parse the data
        let token = match self.validate_import_data(data) {
            Ok(token) => token,
            Err(e) => {
                return Ok(ImportPlan::fail(e.to_string(), dry_run));
            }
        };

        // Determine the action
        let action = if self.token.is_none() {
            ImportAction::Create
        } else {
            ImportAction::Update
        };

        // If not dry-run, perform the actual import
        if !dry_run {
            self.token = Some(token);
            if let Err(e) = self.save_to_storage() {
                return Ok(ImportPlan::fail(format!("Failed to save: {}", e), dry_run));
            }
        }

        // Return the plan
        let plan = match action {
            ImportAction::Create => ImportPlan::create(dry_run),
            ImportAction::Update => ImportPlan::update(dry_run),
            _ => unreachable!(),
        };

        Ok(plan)
    }

    /// Validate import data and return parsed token
    fn validate_import_data(&self, data: &str) -> Result<AuthToken> {
        let json =
            base64::decode(data).map_err(|e| anyhow::anyhow!("Invalid auth data format: {}", e))?;
        let json_str = String::from_utf8(json)
            .map_err(|e| anyhow::anyhow!("Invalid auth data encoding: {}", e))?;
        let token: AuthToken = serde_json::from_str(&json_str)
            .map_err(|e| anyhow::anyhow!("Invalid auth data structure: {}", e))?;
        Ok(token)
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
    fn test_import_plan_create() {
        let mut store = AuthStore::new();
        let token = AuthToken {
            access_token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: None,
            scopes: vec!["read".to_string()],
        };
        let exported = base64::encode(serde_json::to_string(&token).unwrap());

        let plan = store.import_with_plan(&exported, true).unwrap();
        assert_eq!(plan.action, ImportAction::Create);
        assert!(plan.dry_run);
        assert!(plan.reason.is_none());

        // Verify no token was set in dry-run
        assert!(!store.is_authenticated());
    }

    #[test]
    fn test_import_plan_update() {
        let mut store = AuthStore::new();
        let token1 = AuthToken {
            access_token: "old_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: None,
            scopes: vec!["read".to_string()],
        };
        store.set_token(token1);

        let token2 = AuthToken {
            access_token: "new_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: None,
            scopes: vec!["write".to_string()],
        };
        let exported = base64::encode(serde_json::to_string(&token2).unwrap());

        let plan = store.import_with_plan(&exported, true).unwrap();
        assert_eq!(plan.action, ImportAction::Update);
        assert!(plan.dry_run);

        // Verify old token is still there in dry-run
        assert_eq!(
            store.token.as_ref().unwrap().access_token,
            "old_token".to_string()
        );
    }

    #[test]
    fn test_import_plan_fail() {
        let mut store = AuthStore::new();
        let plan = store.import_with_plan("invalid_data", true).unwrap();
        assert_eq!(plan.action, ImportAction::Fail);
        assert!(plan.dry_run);
        assert!(plan.reason.is_some());
        assert!(plan.reason.unwrap().contains("Invalid"));
    }

    #[test]
    fn test_import_plan_actual_import() {
        let mut store = AuthStore::new();
        let token = AuthToken {
            access_token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: None,
            scopes: vec!["read".to_string()],
        };
        let exported = base64::encode(serde_json::to_string(&token).unwrap());

        let plan = store.import_with_plan(&exported, false).unwrap();
        assert_eq!(plan.action, ImportAction::Create);
        assert!(!plan.dry_run);

        // Verify token was actually set
        assert!(store.is_authenticated());
        assert_eq!(
            store.token.as_ref().unwrap().access_token,
            "test_token".to_string()
        );
    }
}
