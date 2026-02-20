use serde::{Deserialize, Serialize};

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
