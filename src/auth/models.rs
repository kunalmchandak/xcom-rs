use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthToken {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "tokenType")]
    pub token_type: String,
    #[serde(rename = "expiresAt", skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    pub scopes: Vec<String>,
}
