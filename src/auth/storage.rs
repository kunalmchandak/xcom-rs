use anyhow::Result;

use super::models::AuthStatus;

/// Environment-based auth store
/// Reads authentication state from environment variables only.
#[derive(Debug, Clone)]
pub struct AuthStore {
    // No persistent state; purely environment-driven
}

impl AuthStore {
    /// Create a new auth store (environment-driven)
    pub fn new() -> Self {
        Self {}
    }

    /// Create an auth store with default storage location
    /// For env-only mode, this is equivalent to new()
    pub fn with_default_storage() -> Result<Self> {
        Ok(Self::new())
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

    /// Get current authentication status from environment variables
    pub fn status(&self) -> AuthStatus {
        // Check if bearer token is set
        let _token = match Self::get_bearer_token() {
            Some(t) if !t.is_empty() => t,
            _ => {
                return AuthStatus::unauthenticated(vec![
                    "Set XCOM_RS_BEARER_TOKEN and re-run the command".to_string(),
                ]);
            }
        };

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

        AuthStatus::authenticated("bearer".to_string(), scopes)
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
}
