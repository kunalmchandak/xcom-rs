use serde::{Deserialize, Serialize};

/// OAuth2 credentials stored in auth.json
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OAuth2Credentials {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(rename = "expiresAt", skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    #[serde(rename = "authMode")]
    pub auth_mode: String,
}

impl OAuth2Credentials {
    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            if let Ok(duration) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            {
                let now = duration.as_secs() as i64;
                return now >= expires_at;
            }
        }
        false
    }

    /// Check if the token is refreshable
    pub fn is_refreshable(&self) -> bool {
        self.refresh_token.is_some()
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
    #[serde(rename = "expiresAt", skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refreshable: Option<bool>,
}

impl AuthStatus {
    /// Create an unauthenticated status with next steps
    pub fn unauthenticated(next_steps: Vec<String>) -> Self {
        Self {
            authenticated: false,
            auth_mode: None,
            scopes: None,
            next_steps: Some(next_steps),
            expires_at: None,
            refreshable: None,
        }
    }

    /// Create an authenticated status
    pub fn authenticated(auth_mode: String, scopes: Vec<String>) -> Self {
        Self {
            authenticated: true,
            auth_mode: Some(auth_mode),
            scopes: Some(scopes),
            next_steps: None,
            expires_at: None,
            refreshable: None,
        }
    }

    /// Create an authenticated status with full details
    pub fn authenticated_with_details(
        auth_mode: String,
        scopes: Vec<String>,
        expires_at: Option<i64>,
        refreshable: bool,
    ) -> Self {
        Self {
            authenticated: true,
            auth_mode: Some(auth_mode),
            scopes: Some(scopes),
            next_steps: None,
            expires_at,
            refreshable: Some(refreshable),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth2_credentials_serialization() {
        let creds = OAuth2Credentials {
            access_token: "test_access_token".to_string(),
            refresh_token: Some("test_refresh_token".to_string()),
            expires_at: Some(1700000000),
            scopes: Some(vec!["tweet.read".to_string(), "tweet.write".to_string()]),
            auth_mode: "oauth2".to_string(),
        };

        let json = serde_json::to_string(&creds).unwrap();
        let deserialized: OAuth2Credentials = serde_json::from_str(&json).unwrap();

        assert_eq!(creds, deserialized);
    }

    #[test]
    fn test_oauth2_credentials_is_expired() {
        // Expired token
        let expired_creds = OAuth2Credentials {
            access_token: "test_token".to_string(),
            refresh_token: None,
            expires_at: Some(1), // Very old timestamp
            scopes: None,
            auth_mode: "oauth2".to_string(),
        };
        assert!(expired_creds.is_expired());

        // Valid token
        let future_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 3600; // 1 hour from now

        let valid_creds = OAuth2Credentials {
            access_token: "test_token".to_string(),
            refresh_token: None,
            expires_at: Some(future_time),
            scopes: None,
            auth_mode: "oauth2".to_string(),
        };
        assert!(!valid_creds.is_expired());

        // No expiration set
        let no_expiry_creds = OAuth2Credentials {
            access_token: "test_token".to_string(),
            refresh_token: None,
            expires_at: None,
            scopes: None,
            auth_mode: "oauth2".to_string(),
        };
        assert!(!no_expiry_creds.is_expired());
    }

    #[test]
    fn test_oauth2_credentials_is_refreshable() {
        let refreshable = OAuth2Credentials {
            access_token: "test_token".to_string(),
            refresh_token: Some("refresh_token".to_string()),
            expires_at: None,
            scopes: None,
            auth_mode: "oauth2".to_string(),
        };
        assert!(refreshable.is_refreshable());

        let not_refreshable = OAuth2Credentials {
            access_token: "test_token".to_string(),
            refresh_token: None,
            expires_at: None,
            scopes: None,
            auth_mode: "oauth2".to_string(),
        };
        assert!(!not_refreshable.is_refreshable());
    }

    #[test]
    fn test_auth_status_with_details() {
        let status = AuthStatus::authenticated_with_details(
            "oauth2".to_string(),
            vec!["tweet.read".to_string()],
            Some(1700000000),
            true,
        );

        assert!(status.authenticated);
        assert_eq!(status.auth_mode, Some("oauth2".to_string()));
        assert_eq!(status.expires_at, Some(1700000000));
        assert_eq!(status.refreshable, Some(true));
    }
}
