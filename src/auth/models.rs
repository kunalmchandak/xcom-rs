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

/// OAuth1.0a credentials stored in auth.json
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OAuth1aCredentials {
    #[serde(rename = "authMode")]
    pub auth_mode: String,
    #[serde(rename = "consumerKey")]
    pub consumer_key: String,
    #[serde(rename = "consumerSecret")]
    pub consumer_secret: String,
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "accessTokenSecret")]
    pub access_token_secret: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

/// Unified authentication credentials (OAuth2 or OAuth1.0a)
/// Supports backward compatibility with existing OAuth2-only auth.json files
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AuthCredentials {
    OAuth1a(OAuth1aCredentials),
    OAuth2(OAuth2Credentials),
}

impl AuthCredentials {
    /// Get the auth mode
    pub fn auth_mode(&self) -> &str {
        match self {
            AuthCredentials::OAuth1a(creds) => &creds.auth_mode,
            AuthCredentials::OAuth2(creds) => &creds.auth_mode,
        }
    }

    /// Get scopes
    pub fn scopes(&self) -> Option<Vec<String>> {
        match self {
            AuthCredentials::OAuth1a(creds) => creds.scopes.clone(),
            AuthCredentials::OAuth2(creds) => creds.scopes.clone(),
        }
    }

    /// Check if the credentials are expired (OAuth2 only, OAuth1.0a never expires)
    pub fn is_expired(&self) -> bool {
        match self {
            AuthCredentials::OAuth1a(_) => false,
            AuthCredentials::OAuth2(creds) => creds.is_expired(),
        }
    }

    /// Check if the credentials are refreshable (OAuth2 only)
    pub fn is_refreshable(&self) -> bool {
        match self {
            AuthCredentials::OAuth1a(_) => false,
            AuthCredentials::OAuth2(creds) => creds.is_refreshable(),
        }
    }

    /// Get expires_at (OAuth2 only)
    pub fn expires_at(&self) -> Option<i64> {
        match self {
            AuthCredentials::OAuth1a(_) => None,
            AuthCredentials::OAuth2(creds) => creds.expires_at,
        }
    }
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

    #[test]
    fn test_oauth1a_credentials_serialization() {
        let creds = OAuth1aCredentials {
            auth_mode: "oauth1a".to_string(),
            consumer_key: "test_consumer_key".to_string(),
            consumer_secret: "test_consumer_secret".to_string(),
            access_token: "test_access_token".to_string(),
            access_token_secret: "test_access_token_secret".to_string(),
            scopes: None,
        };

        let json = serde_json::to_string(&creds).unwrap();
        let deserialized: OAuth1aCredentials = serde_json::from_str(&json).unwrap();

        assert_eq!(creds, deserialized);
    }

    #[test]
    fn test_auth_credentials_oauth2_backward_compat() {
        // Test that existing OAuth2 JSON can be deserialized
        let oauth2_json = r#"{
            "access_token": "test_token",
            "refresh_token": "refresh_token",
            "expiresAt": 1700000000,
            "scopes": ["tweet.read"],
            "authMode": "oauth2"
        }"#;

        let creds: AuthCredentials = serde_json::from_str(oauth2_json).unwrap();
        assert_eq!(creds.auth_mode(), "oauth2");
        assert_eq!(creds.scopes(), Some(vec!["tweet.read".to_string()]));
        assert!(creds.is_refreshable());
    }

    #[test]
    fn test_auth_credentials_oauth1a() {
        let oauth1a_json = r#"{
            "authMode": "oauth1a",
            "consumerKey": "consumer_key",
            "consumerSecret": "consumer_secret",
            "accessToken": "access_token",
            "accessTokenSecret": "access_token_secret",
            "scopes": null
        }"#;

        let creds: AuthCredentials = serde_json::from_str(oauth1a_json).unwrap();
        assert_eq!(creds.auth_mode(), "oauth1a");
        assert_eq!(creds.scopes(), None);
        assert!(!creds.is_expired());
        assert!(!creds.is_refreshable());
    }

    #[test]
    fn test_auth_credentials_oauth2_without_authmode() {
        // Test backward compat: if authMode is missing, should still deserialize as OAuth2
        let oauth2_json_no_mode = r#"{
            "access_token": "test_token",
            "authMode": "oauth2"
        }"#;

        let creds: AuthCredentials = serde_json::from_str(oauth2_json_no_mode).unwrap();
        assert_eq!(creds.auth_mode(), "oauth2");
    }
}
