use anyhow::{Context, Result};
use std::path::PathBuf;

use super::models::{AuthCredentials, AuthStatus, OAuth1aCredentials, OAuth2Credentials};

/// Environment-based auth store
/// Reads authentication state from environment variables and persisted OAuth2 credentials.
#[derive(Debug, Clone)]
pub struct AuthStore {
    auth_file_path: Option<PathBuf>,
}

impl AuthStore {
    /// Create a new auth store (environment-driven, no persistent storage)
    pub fn new() -> Self {
        Self {
            auth_file_path: None,
        }
    }

    /// Create an auth store with default storage location
    pub fn with_default_storage() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("xcom-rs");
        let auth_file_path = config_dir.join("auth.json");
        Ok(Self {
            auth_file_path: Some(auth_file_path),
        })
    }

    /// Get the path to the auth file
    fn auth_file_path(&self) -> Option<&PathBuf> {
        self.auth_file_path.as_ref()
    }

    /// Load authentication credentials from disk (OAuth2 or OAuth1.0a)
    pub fn load_credentials(&self) -> Result<Option<AuthCredentials>> {
        let Some(path) = self.auth_file_path() else {
            return Ok(None);
        };

        if !path.exists() {
            return Ok(None);
        };

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read auth file: {}", path.display()))?;

        let creds: AuthCredentials = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse auth file: {}", path.display()))?;

        Ok(Some(creds))
    }

    /// Load OAuth2 credentials from disk (legacy method for backward compatibility)
    pub fn load_oauth2_credentials(&self) -> Result<Option<OAuth2Credentials>> {
        match self.load_credentials()? {
            Some(AuthCredentials::OAuth2(creds)) => Ok(Some(creds)),
            Some(AuthCredentials::OAuth1a(_)) => Ok(None),
            None => Ok(None),
        }
    }

    /// Load OAuth1.0a credentials from disk
    pub fn load_oauth1a_credentials(&self) -> Result<Option<OAuth1aCredentials>> {
        match self.load_credentials()? {
            Some(AuthCredentials::OAuth1a(creds)) => Ok(Some(creds)),
            Some(AuthCredentials::OAuth2(_)) => Ok(None),
            None => Ok(None),
        }
    }

    /// Save authentication credentials to disk (OAuth2 or OAuth1.0a)
    pub fn save_credentials(&self, creds: &AuthCredentials) -> Result<()> {
        let Some(path) = self.auth_file_path() else {
            anyhow::bail!("No storage path configured");
        };

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let content =
            serde_json::to_string_pretty(creds).context("Failed to serialize credentials")?;

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write auth file: {}", path.display()))?;

        Ok(())
    }

    /// Save OAuth2 credentials to disk (legacy method for backward compatibility)
    pub fn save_oauth2_credentials(&self, creds: &OAuth2Credentials) -> Result<()> {
        self.save_credentials(&AuthCredentials::OAuth2(creds.clone()))
    }

    /// Save OAuth1.0a credentials to disk
    pub fn save_oauth1a_credentials(&self, creds: &OAuth1aCredentials) -> Result<()> {
        self.save_credentials(&AuthCredentials::OAuth1a(creds.clone()))
    }

    /// Delete saved OAuth2 credentials
    pub fn delete_oauth2_credentials(&self) -> Result<()> {
        let Some(path) = self.auth_file_path() else {
            return Ok(()); // Nothing to delete
        };

        if path.exists() {
            std::fs::remove_file(path)
                .with_context(|| format!("Failed to delete auth file: {}", path.display()))?;
        }

        Ok(())
    }

    /// Get OAuth1.0a consumer key from environment
    fn get_oauth1a_consumer_key() -> Option<String> {
        std::env::var("XCOM_RS_OAUTH1A_CONSUMER_KEY").ok()
    }

    /// Get OAuth1.0a consumer secret from environment
    fn get_oauth1a_consumer_secret() -> Option<String> {
        std::env::var("XCOM_RS_OAUTH1A_CONSUMER_SECRET").ok()
    }

    /// Get OAuth1.0a access token from environment
    fn get_oauth1a_access_token() -> Option<String> {
        std::env::var("XCOM_RS_OAUTH1A_ACCESS_TOKEN").ok()
    }

    /// Get OAuth1.0a access token secret from environment
    fn get_oauth1a_access_token_secret() -> Option<String> {
        std::env::var("XCOM_RS_OAUTH1A_ACCESS_TOKEN_SECRET").ok()
    }

    /// Resolve OAuth1.0a credentials from environment or disk
    pub fn resolve_oauth1a_credentials(&self) -> Result<Option<OAuth1aCredentials>> {
        // Priority 1: Check environment variables
        if let (
            Some(consumer_key),
            Some(consumer_secret),
            Some(access_token),
            Some(access_token_secret),
        ) = (
            Self::get_oauth1a_consumer_key(),
            Self::get_oauth1a_consumer_secret(),
            Self::get_oauth1a_access_token(),
            Self::get_oauth1a_access_token_secret(),
        ) {
            if !consumer_key.is_empty()
                && !consumer_secret.is_empty()
                && !access_token.is_empty()
                && !access_token_secret.is_empty()
            {
                return Ok(Some(OAuth1aCredentials {
                    auth_mode: "oauth1a".to_string(),
                    consumer_key,
                    consumer_secret,
                    access_token,
                    access_token_secret,
                    scopes: Self::get_scopes(),
                }));
            }
        }

        // Priority 2: Check saved credentials
        self.load_oauth1a_credentials()
    }

    /// Resolve bearer token with priority:
    /// 1. OAuth1.0a env vars (CI/non-interactive use)
    /// 2. OAuth2/Bearer env vars
    /// 3. Saved OAuth1.0a credentials
    /// 4. Saved OAuth2 credentials
    ///
    /// Note: For OAuth1.0a, this returns None since OAuth1.0a uses signature-based auth
    /// Use resolve_oauth1a_credentials() for OAuth1.0a auth
    /// Automatically refreshes expired tokens if refresh token is available (OAuth2 only)
    pub fn resolve_token(&self) -> Result<Option<String>> {
        // Priority 1: Check OAuth1.0a environment variables (returns None, OAuth1.0a doesn't use bearer tokens)
        if Self::get_oauth1a_consumer_key().is_some() {
            // OAuth1.0a is configured via env, but doesn't use bearer tokens
            return Ok(None);
        }

        // Priority 2: Check OAuth2/Bearer environment variable
        if let Some(token) = Self::get_bearer_token() {
            if !token.is_empty() {
                return Ok(Some(token));
            }
        }

        // Priority 3: Check saved credentials (OAuth1.0a or OAuth2)
        if let Some(creds) = self.load_credentials()? {
            match creds {
                AuthCredentials::OAuth1a(_) => {
                    // OAuth1.0a doesn't use bearer tokens
                    return Ok(None);
                }
                AuthCredentials::OAuth2(mut oauth2_creds) => {
                    // Handle OAuth2 token refresh logic
                    return self.handle_oauth2_token(&mut oauth2_creds);
                }
            }
        }

        // Priority 4: Legacy - Check saved OAuth2 credentials
        if let Some(mut creds) = self.load_oauth2_credentials()? {
            return self.handle_oauth2_token(&mut creds);
        }

        Ok(None)
    }

    /// Handle OAuth2 token resolution and refresh
    fn handle_oauth2_token(&self, creds: &mut OAuth2Credentials) -> Result<Option<String>> {
        // If expired and refreshable, try to refresh
        if creds.is_expired() && creds.is_refreshable() {
            // Get client_id from environment (required for refresh)
            let client_id = std::env::var("XCOM_RS_CLIENT_ID")
                .context("XCOM_RS_CLIENT_ID required for token refresh")?;
            let client_secret = std::env::var("XCOM_RS_CLIENT_SECRET").ok();

            let client = super::oauth2::OAuth2Client::new(client_id, client_secret);
            let refresh_token = creds
                .refresh_token
                .as_ref()
                .context("No refresh token available")?;

            tracing::info!("Refreshing expired OAuth2 token");
            let token_response = client.refresh_token(refresh_token)?;

            // Update credentials
            creds.access_token = token_response.access_token.clone();
            if let Some(new_refresh_token) = &token_response.refresh_token {
                creds.refresh_token = Some(new_refresh_token.clone());
            }
            if let Some(expires_in) = token_response.expires_in {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .context("System time error")?
                    .as_secs() as i64;
                creds.expires_at = Some(now + expires_in as i64);
            }

            // Save updated credentials
            self.save_oauth2_credentials(creds)?;

            Ok(Some(token_response.access_token))
        } else if !creds.is_expired() {
            Ok(Some(creds.access_token.clone()))
        } else {
            // Expired and not refreshable
            anyhow::bail!("Stored access token expired and cannot be refreshed. Run 'xcom-rs auth login' to re-authenticate");
        }
    }

    /// Parse bearer token from environment variable
    /// Accepts "Bearer <token>" or raw token format
    fn parse_bearer_token(value: &str) -> String {
        if let Some(stripped) = value.strip_prefix("Bearer ") {
            stripped.to_string()
        } else {
            value.to_string()
        }
    }

    /// Parse scopes from environment variable
    /// Supports space-separated and comma-separated formats
    fn parse_scopes(value: &str) -> Vec<String> {
        // Try comma-separated first
        if value.contains(',') {
            value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            // Fall back to space-separated
            value.split_whitespace().map(|s| s.to_string()).collect()
        }
    }

    /// Get bearer token from environment
    fn get_bearer_token() -> Option<String> {
        std::env::var("XCOM_RS_BEARER_TOKEN")
            .ok()
            .map(|v| Self::parse_bearer_token(&v))
    }

    /// Get scopes from environment
    fn get_scopes() -> Option<Vec<String>> {
        std::env::var("XCOM_RS_SCOPES")
            .ok()
            .map(|v| Self::parse_scopes(&v))
    }

    /// Get expires_at from environment
    /// Returns None if invalid or not set
    fn get_expires_at() -> Option<i64> {
        std::env::var("XCOM_RS_EXPIRES_AT")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
    }

    /// Get current authentication status from environment variables and stored credentials
    pub fn status(&self) -> AuthStatus {
        // Priority 1: Check OAuth1.0a environment variables
        if let Ok(Some(oauth1a_creds)) = self.resolve_oauth1a_credentials() {
            return AuthStatus::authenticated_with_details(
                "oauth1a".to_string(),
                oauth1a_creds.scopes.unwrap_or_default(),
                None,  // OAuth1.0a tokens don't expire
                false, // OAuth1.0a tokens are not refreshable
            );
        }

        // Priority 2: Check OAuth2/Bearer environment variable
        if let Some(token) = Self::get_bearer_token() {
            if !token.is_empty() {
                // Check expiration if set
                if let Some(expires_at) = Self::get_expires_at() {
                    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                        Ok(duration) => {
                            let now = duration.as_secs() as i64;
                            if now >= expires_at {
                                return AuthStatus::unauthenticated(vec![
                                    "Verify XCOM_RS_EXPIRES_AT is not in the past".to_string(),
                                ]);
                            }
                        }
                        Err(_) => {
                            return AuthStatus::unauthenticated(vec![
                                "System time error. Please check your system clock.".to_string(),
                            ]);
                        }
                    }
                }

                // Get scopes (optional)
                let scopes = Self::get_scopes().unwrap_or_default();
                return AuthStatus::authenticated("bearer".to_string(), scopes);
            }
        }

        // Priority 3: Check saved credentials (OAuth1.0a or OAuth2)
        if let Ok(Some(creds)) = self.load_credentials() {
            match creds {
                AuthCredentials::OAuth1a(oauth1a_creds) => {
                    return AuthStatus::authenticated_with_details(
                        "oauth1a".to_string(),
                        oauth1a_creds.scopes.unwrap_or_default(),
                        None,  // OAuth1.0a tokens don't expire
                        false, // OAuth1.0a tokens are not refreshable
                    );
                }
                AuthCredentials::OAuth2(oauth2_creds) => {
                    if oauth2_creds.is_expired() {
                        // If expired but refreshable, still show as authenticated but mark as needs refresh
                        if oauth2_creds.is_refreshable() {
                            return AuthStatus::authenticated_with_details(
                                oauth2_creds.auth_mode.clone(),
                                oauth2_creds.scopes.clone().unwrap_or_default(),
                                oauth2_creds.expires_at,
                                true,
                            );
                        } else {
                            return AuthStatus::unauthenticated(vec![
                                "Stored access token expired. Run 'xcom-rs auth login' to re-authenticate"
                                    .to_string(),
                            ]);
                        }
                    }

                    return AuthStatus::authenticated_with_details(
                        oauth2_creds.auth_mode.clone(),
                        oauth2_creds.scopes.clone().unwrap_or_default(),
                        oauth2_creds.expires_at,
                        oauth2_creds.is_refreshable(),
                    );
                }
            }
        }

        // No credentials found
        AuthStatus::unauthenticated(vec![
            "Set XCOM_RS_BEARER_TOKEN or run 'xcom-rs auth login'".to_string()
        ])
    }

    /// Check if authenticated (based on environment variables)
    pub fn is_authenticated(&self) -> bool {
        Self::get_bearer_token().filter(|t| !t.is_empty()).is_some() && {
            // Check expiration if set
            if let Some(expires_at) = Self::get_expires_at() {
                if let Ok(duration) =
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                {
                    let now = duration.as_secs() as i64;
                    now < expires_at
                } else {
                    false
                }
            } else {
                true
            }
        }
    }
}

impl Default for AuthStore {
    fn default() -> Self {
        Self::new()
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
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // Save original value
        let original = std::env::var("XCOM_RS_BEARER_TOKEN").ok();
        std::env::remove_var("XCOM_RS_BEARER_TOKEN");

        let store = AuthStore::new();
        let status = store.status();
        assert!(!status.authenticated);
        assert!(status.next_steps.is_some());

        // Restore original value
        if let Some(val) = original {
            std::env::set_var("XCOM_RS_BEARER_TOKEN", val);
        }
    }

    #[test]
    fn test_auth_store_with_token() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // Save original values
        let original_token = std::env::var("XCOM_RS_BEARER_TOKEN").ok();
        let original_scopes = std::env::var("XCOM_RS_SCOPES").ok();

        std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");
        std::env::set_var("XCOM_RS_SCOPES", "read");

        let store = AuthStore::new();
        let status = store.status();
        assert!(status.authenticated);
        assert_eq!(status.auth_mode, Some("bearer".to_string()));
        assert_eq!(status.scopes, Some(vec!["read".to_string()]));

        // Restore original values
        match original_token {
            Some(val) => std::env::set_var("XCOM_RS_BEARER_TOKEN", val),
            None => std::env::remove_var("XCOM_RS_BEARER_TOKEN"),
        }
        match original_scopes {
            Some(val) => std::env::set_var("XCOM_RS_SCOPES", val),
            None => std::env::remove_var("XCOM_RS_SCOPES"),
        }
    }

    #[test]
    fn test_parse_bearer_token_with_prefix() {
        let token = AuthStore::parse_bearer_token("Bearer test_token_123");
        assert_eq!(token, "test_token_123");
    }

    #[test]
    fn test_parse_bearer_token_without_prefix() {
        let token = AuthStore::parse_bearer_token("test_token_456");
        assert_eq!(token, "test_token_456");
    }

    #[test]
    fn test_parse_scopes_space_separated() {
        let scopes = AuthStore::parse_scopes("read write delete");
        assert_eq!(
            scopes,
            vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string()
            ]
        );
    }

    #[test]
    fn test_parse_scopes_comma_separated() {
        let scopes = AuthStore::parse_scopes("read, write, delete");
        assert_eq!(
            scopes,
            vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string()
            ]
        );
    }

    #[test]
    fn test_get_bearer_token_from_env() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // Save original value
        let original = std::env::var("XCOM_RS_BEARER_TOKEN").ok();
        std::env::set_var("XCOM_RS_BEARER_TOKEN", "Bearer my_token");

        let token = AuthStore::get_bearer_token();
        assert_eq!(token, Some("my_token".to_string()));

        // Restore original value
        match original {
            Some(val) => std::env::set_var("XCOM_RS_BEARER_TOKEN", val),
            None => std::env::remove_var("XCOM_RS_BEARER_TOKEN"),
        }
    }

    #[test]
    fn test_get_scopes_from_env() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // Save original value
        let original = std::env::var("XCOM_RS_SCOPES").ok();
        std::env::set_var("XCOM_RS_SCOPES", "read write");

        let scopes = AuthStore::get_scopes();
        assert_eq!(scopes, Some(vec!["read".to_string(), "write".to_string()]));

        // Restore original value
        match original {
            Some(val) => std::env::set_var("XCOM_RS_SCOPES", val),
            None => std::env::remove_var("XCOM_RS_SCOPES"),
        }
    }

    #[test]
    fn test_get_expires_at_from_env() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // Save original value
        let original = std::env::var("XCOM_RS_EXPIRES_AT").ok();
        std::env::set_var("XCOM_RS_EXPIRES_AT", "1700000000");

        let expires = AuthStore::get_expires_at();
        assert_eq!(expires, Some(1700000000));

        // Restore original value
        match original {
            Some(val) => std::env::set_var("XCOM_RS_EXPIRES_AT", val),
            None => std::env::remove_var("XCOM_RS_EXPIRES_AT"),
        }
    }

    #[test]
    fn test_status_expired_token() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // Save original values
        let original_token = std::env::var("XCOM_RS_BEARER_TOKEN").ok();
        let original_expires = std::env::var("XCOM_RS_EXPIRES_AT").ok();

        std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");
        std::env::set_var("XCOM_RS_EXPIRES_AT", "1"); // Very old timestamp

        let store = AuthStore::new();
        let status = store.status();
        assert!(!status.authenticated);
        assert!(status.next_steps.is_some());

        // Restore original values
        match original_token {
            Some(val) => std::env::set_var("XCOM_RS_BEARER_TOKEN", val),
            None => std::env::remove_var("XCOM_RS_BEARER_TOKEN"),
        }
        match original_expires {
            Some(val) => std::env::set_var("XCOM_RS_EXPIRES_AT", val),
            None => std::env::remove_var("XCOM_RS_EXPIRES_AT"),
        }
    }

    #[test]
    fn test_is_authenticated() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // Save original value and set test value
        let original = std::env::var("XCOM_RS_BEARER_TOKEN").ok();
        std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");

        let store = AuthStore::new();
        assert!(store.is_authenticated());

        // Restore original value
        match original {
            Some(val) => std::env::set_var("XCOM_RS_BEARER_TOKEN", val),
            None => std::env::remove_var("XCOM_RS_BEARER_TOKEN"),
        }
    }

    #[test]
    fn test_is_not_authenticated() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // Save original value and remove for test
        let original = std::env::var("XCOM_RS_BEARER_TOKEN").ok();
        std::env::remove_var("XCOM_RS_BEARER_TOKEN");

        let store = AuthStore::new();
        assert!(!store.is_authenticated());

        // Restore original value
        if let Some(val) = original {
            std::env::set_var("XCOM_RS_BEARER_TOKEN", val);
        }
    }

    #[test]
    fn test_save_and_load_oauth2_credentials() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let auth_file = temp_dir.path().join("auth.json");

        let store = AuthStore {
            auth_file_path: Some(auth_file.clone()),
        };

        let creds = OAuth2Credentials {
            access_token: "test_access_token".to_string(),
            refresh_token: Some("test_refresh_token".to_string()),
            expires_at: Some(1700000000),
            scopes: Some(vec!["tweet.read".to_string()]),
            auth_mode: "oauth2".to_string(),
        };

        // Save credentials
        store.save_oauth2_credentials(&creds).unwrap();

        // Verify file was created
        assert!(auth_file.exists());

        // Load credentials
        let loaded = store.load_oauth2_credentials().unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), creds);
    }

    #[test]
    fn test_delete_oauth2_credentials() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let auth_file = temp_dir.path().join("auth.json");

        let store = AuthStore {
            auth_file_path: Some(auth_file.clone()),
        };

        let creds = OAuth2Credentials {
            access_token: "test_token".to_string(),
            refresh_token: None,
            expires_at: None,
            scopes: None,
            auth_mode: "oauth2".to_string(),
        };

        // Save and verify
        store.save_oauth2_credentials(&creds).unwrap();
        assert!(auth_file.exists());

        // Delete
        store.delete_oauth2_credentials().unwrap();
        assert!(!auth_file.exists());
    }

    #[test]
    fn test_status_with_oauth2_credentials() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        use tempfile::TempDir;

        // Clear environment variable
        let original = std::env::var("XCOM_RS_BEARER_TOKEN").ok();
        std::env::remove_var("XCOM_RS_BEARER_TOKEN");

        let temp_dir = TempDir::new().unwrap();
        let auth_file = temp_dir.path().join("auth.json");

        let store = AuthStore {
            auth_file_path: Some(auth_file),
        };

        // Create valid credentials
        let future_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 3600;

        let creds = OAuth2Credentials {
            access_token: "test_token".to_string(),
            refresh_token: Some("refresh_token".to_string()),
            expires_at: Some(future_time),
            scopes: Some(vec!["tweet.read".to_string()]),
            auth_mode: "oauth2".to_string(),
        };

        store.save_oauth2_credentials(&creds).unwrap();

        let status = store.status();
        assert!(status.authenticated);
        assert_eq!(status.auth_mode, Some("oauth2".to_string()));
        assert_eq!(status.refreshable, Some(true));

        // Restore environment
        if let Some(val) = original {
            std::env::set_var("XCOM_RS_BEARER_TOKEN", val);
        }
    }

    #[test]
    fn test_resolve_token_from_env() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        use tempfile::TempDir;

        let original = std::env::var("XCOM_RS_BEARER_TOKEN").ok();
        std::env::set_var("XCOM_RS_BEARER_TOKEN", "env_token");

        let temp_dir = TempDir::new().unwrap();
        let auth_file = temp_dir.path().join("auth.json");

        let store = AuthStore {
            auth_file_path: Some(auth_file),
        };

        let token = store.resolve_token().unwrap();
        assert_eq!(token, Some("env_token".to_string()));

        // Restore environment
        match original {
            Some(val) => std::env::set_var("XCOM_RS_BEARER_TOKEN", val),
            None => std::env::remove_var("XCOM_RS_BEARER_TOKEN"),
        }
    }

    #[test]
    fn test_resolve_token_from_oauth2() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        use tempfile::TempDir;

        let original = std::env::var("XCOM_RS_BEARER_TOKEN").ok();
        std::env::remove_var("XCOM_RS_BEARER_TOKEN");

        let temp_dir = TempDir::new().unwrap();
        let auth_file = temp_dir.path().join("auth.json");

        let store = AuthStore {
            auth_file_path: Some(auth_file),
        };

        // Create valid credentials
        let future_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 3600;

        let creds = OAuth2Credentials {
            access_token: "oauth2_token".to_string(),
            refresh_token: None,
            expires_at: Some(future_time),
            scopes: None,
            auth_mode: "oauth2".to_string(),
        };

        store.save_oauth2_credentials(&creds).unwrap();

        let token = store.resolve_token().unwrap();
        assert_eq!(token, Some("oauth2_token".to_string()));

        // Restore environment
        if let Some(val) = original {
            std::env::set_var("XCOM_RS_BEARER_TOKEN", val);
        }
    }

    #[test]
    fn test_resolve_token_none() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        use tempfile::TempDir;

        let original = std::env::var("XCOM_RS_BEARER_TOKEN").ok();
        std::env::remove_var("XCOM_RS_BEARER_TOKEN");

        let temp_dir = TempDir::new().unwrap();
        let auth_file = temp_dir.path().join("auth.json");

        let store = AuthStore {
            auth_file_path: Some(auth_file),
        };

        let token = store.resolve_token().unwrap();
        assert_eq!(token, None);

        // Restore environment
        if let Some(val) = original {
            std::env::set_var("XCOM_RS_BEARER_TOKEN", val);
        }
    }

    #[test]
    fn test_save_and_load_oauth1a_credentials() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let auth_file = temp_dir.path().join("auth.json");

        let store = AuthStore {
            auth_file_path: Some(auth_file.clone()),
        };

        let creds = OAuth1aCredentials {
            auth_mode: "oauth1a".to_string(),
            consumer_key: "test_consumer_key".to_string(),
            consumer_secret: "test_consumer_secret".to_string(),
            access_token: "test_access_token".to_string(),
            access_token_secret: "test_access_token_secret".to_string(),
            scopes: None,
        };

        // Save credentials
        store.save_oauth1a_credentials(&creds).unwrap();

        // Verify file was created
        assert!(auth_file.exists());

        // Load credentials
        let loaded = store.load_oauth1a_credentials().unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), creds);
    }

    #[test]
    fn test_resolve_oauth1a_from_env() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        use tempfile::TempDir;

        // Save original values
        let original_consumer_key = std::env::var("XCOM_RS_OAUTH1A_CONSUMER_KEY").ok();
        let original_consumer_secret = std::env::var("XCOM_RS_OAUTH1A_CONSUMER_SECRET").ok();
        let original_access_token = std::env::var("XCOM_RS_OAUTH1A_ACCESS_TOKEN").ok();
        let original_access_token_secret =
            std::env::var("XCOM_RS_OAUTH1A_ACCESS_TOKEN_SECRET").ok();

        // Set OAuth1.0a environment variables
        std::env::set_var("XCOM_RS_OAUTH1A_CONSUMER_KEY", "env_consumer_key");
        std::env::set_var("XCOM_RS_OAUTH1A_CONSUMER_SECRET", "env_consumer_secret");
        std::env::set_var("XCOM_RS_OAUTH1A_ACCESS_TOKEN", "env_access_token");
        std::env::set_var(
            "XCOM_RS_OAUTH1A_ACCESS_TOKEN_SECRET",
            "env_access_token_secret",
        );

        let temp_dir = TempDir::new().unwrap();
        let auth_file = temp_dir.path().join("auth.json");

        let store = AuthStore {
            auth_file_path: Some(auth_file),
        };

        let creds = store.resolve_oauth1a_credentials().unwrap();
        assert!(creds.is_some());
        let creds = creds.unwrap();
        assert_eq!(creds.consumer_key, "env_consumer_key");
        assert_eq!(creds.access_token, "env_access_token");

        // Restore environment
        match original_consumer_key {
            Some(val) => std::env::set_var("XCOM_RS_OAUTH1A_CONSUMER_KEY", val),
            None => std::env::remove_var("XCOM_RS_OAUTH1A_CONSUMER_KEY"),
        }
        match original_consumer_secret {
            Some(val) => std::env::set_var("XCOM_RS_OAUTH1A_CONSUMER_SECRET", val),
            None => std::env::remove_var("XCOM_RS_OAUTH1A_CONSUMER_SECRET"),
        }
        match original_access_token {
            Some(val) => std::env::set_var("XCOM_RS_OAUTH1A_ACCESS_TOKEN", val),
            None => std::env::remove_var("XCOM_RS_OAUTH1A_ACCESS_TOKEN"),
        }
        match original_access_token_secret {
            Some(val) => std::env::set_var("XCOM_RS_OAUTH1A_ACCESS_TOKEN_SECRET", val),
            None => std::env::remove_var("XCOM_RS_OAUTH1A_ACCESS_TOKEN_SECRET"),
        }
    }

    #[test]
    fn test_status_with_oauth1a_env() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        use tempfile::TempDir;

        // Save original values
        let original_consumer_key = std::env::var("XCOM_RS_OAUTH1A_CONSUMER_KEY").ok();
        let original_consumer_secret = std::env::var("XCOM_RS_OAUTH1A_CONSUMER_SECRET").ok();
        let original_access_token = std::env::var("XCOM_RS_OAUTH1A_ACCESS_TOKEN").ok();
        let original_access_token_secret =
            std::env::var("XCOM_RS_OAUTH1A_ACCESS_TOKEN_SECRET").ok();

        // Set OAuth1.0a environment variables
        std::env::set_var("XCOM_RS_OAUTH1A_CONSUMER_KEY", "env_consumer_key");
        std::env::set_var("XCOM_RS_OAUTH1A_CONSUMER_SECRET", "env_consumer_secret");
        std::env::set_var("XCOM_RS_OAUTH1A_ACCESS_TOKEN", "env_access_token");
        std::env::set_var(
            "XCOM_RS_OAUTH1A_ACCESS_TOKEN_SECRET",
            "env_access_token_secret",
        );

        let temp_dir = TempDir::new().unwrap();
        let auth_file = temp_dir.path().join("auth.json");

        let store = AuthStore {
            auth_file_path: Some(auth_file),
        };

        let status = store.status();
        assert!(status.authenticated);
        assert_eq!(status.auth_mode, Some("oauth1a".to_string()));
        assert_eq!(status.refreshable, Some(false)); // OAuth1.0a is not refreshable
        assert_eq!(status.expires_at, None); // OAuth1.0a doesn't expire

        // Restore environment
        match original_consumer_key {
            Some(val) => std::env::set_var("XCOM_RS_OAUTH1A_CONSUMER_KEY", val),
            None => std::env::remove_var("XCOM_RS_OAUTH1A_CONSUMER_KEY"),
        }
        match original_consumer_secret {
            Some(val) => std::env::set_var("XCOM_RS_OAUTH1A_CONSUMER_SECRET", val),
            None => std::env::remove_var("XCOM_RS_OAUTH1A_CONSUMER_SECRET"),
        }
        match original_access_token {
            Some(val) => std::env::set_var("XCOM_RS_OAUTH1A_ACCESS_TOKEN", val),
            None => std::env::remove_var("XCOM_RS_OAUTH1A_ACCESS_TOKEN"),
        }
        match original_access_token_secret {
            Some(val) => std::env::set_var("XCOM_RS_OAUTH1A_ACCESS_TOKEN_SECRET", val),
            None => std::env::remove_var("XCOM_RS_OAUTH1A_ACCESS_TOKEN_SECRET"),
        }
    }

    #[test]
    fn test_status_with_oauth1a_saved() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        use tempfile::TempDir;

        // Clear environment variables
        let original_bearer = std::env::var("XCOM_RS_BEARER_TOKEN").ok();
        std::env::remove_var("XCOM_RS_BEARER_TOKEN");

        let temp_dir = TempDir::new().unwrap();
        let auth_file = temp_dir.path().join("auth.json");

        let store = AuthStore {
            auth_file_path: Some(auth_file),
        };

        let creds = OAuth1aCredentials {
            auth_mode: "oauth1a".to_string(),
            consumer_key: "saved_consumer_key".to_string(),
            consumer_secret: "saved_consumer_secret".to_string(),
            access_token: "saved_access_token".to_string(),
            access_token_secret: "saved_access_token_secret".to_string(),
            scopes: None,
        };

        store.save_oauth1a_credentials(&creds).unwrap();

        let status = store.status();
        assert!(status.authenticated);
        assert_eq!(status.auth_mode, Some("oauth1a".to_string()));
        assert_eq!(status.refreshable, Some(false));
        assert_eq!(status.expires_at, None);

        // Restore environment
        if let Some(val) = original_bearer {
            std::env::set_var("XCOM_RS_BEARER_TOKEN", val);
        }
    }
}
