use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::auth::{AuthStatus, AuthStore};
use crate::billing::BudgetTracker;
use crate::context::ExecutionContext;

/// Diagnostic information about the system configuration and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorDiagnostics {
    /// Authentication status
    #[serde(rename = "authStatus")]
    pub auth_status: AuthStatus,

    /// Authentication storage path resolution (if file-based)
    #[serde(rename = "authStoragePath", skip_serializing_if = "Option::is_none")]
    pub auth_storage_path: Option<PathInfo>,

    /// Budget tracker storage path resolution
    #[serde(rename = "budgetStoragePath", skip_serializing_if = "Option::is_none")]
    pub budget_storage_path: Option<PathInfo>,

    /// Execution mode settings
    #[serde(rename = "executionMode")]
    pub execution_mode: ExecutionMode,

    /// Any warnings encountered during diagnostics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

/// Information about a resolved path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathInfo {
    /// The resolved absolute path
    pub path: String,

    /// Whether the path exists
    pub exists: bool,

    /// Whether the path is readable (if it exists)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readable: Option<bool>,

    /// Whether the path is writable (if it exists)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writable: Option<bool>,
}

impl PathInfo {
    /// Create PathInfo from a PathBuf
    pub fn from_path(path: PathBuf) -> Self {
        let exists = path.exists();
        let readable = if exists {
            Some(
                path.metadata()
                    .map(|m| !m.permissions().readonly())
                    .unwrap_or(false),
            )
        } else {
            None
        };
        let writable = readable; // Simplified: same as readable for now

        Self {
            path: path.to_string_lossy().to_string(),
            exists,
            readable,
            writable,
        }
    }
}

/// Execution mode settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMode {
    /// Whether running in non-interactive mode
    #[serde(rename = "nonInteractive")]
    pub non_interactive: bool,

    /// Whether running in dry-run mode
    #[serde(rename = "dryRun")]
    pub dry_run: bool,

    /// Maximum cost limit in credits (if set)
    #[serde(rename = "maxCostCredits", skip_serializing_if = "Option::is_none")]
    pub max_cost_credits: Option<u32>,

    /// Daily budget limit in credits (if set)
    #[serde(rename = "dailyBudgetCredits", skip_serializing_if = "Option::is_none")]
    pub daily_budget_credits: Option<u32>,
}

impl ExecutionMode {
    /// Create from ExecutionContext
    pub fn from_context(ctx: &ExecutionContext) -> Self {
        Self {
            non_interactive: ctx.non_interactive,
            dry_run: ctx.dry_run,
            max_cost_credits: ctx.max_cost_credits,
            daily_budget_credits: ctx.budget_daily_credits,
        }
    }
}

/// Collect diagnostic information
pub fn collect_diagnostics(
    auth_store: &AuthStore,
    ctx: &ExecutionContext,
) -> Result<DoctorDiagnostics> {
    let mut warnings = Vec::new();

    // Get auth status
    let auth_status = auth_store.status();

    // Try to get auth storage path
    let auth_storage_path = match AuthStore::default_storage_path() {
        Ok(path) => Some(PathInfo::from_path(path)),
        Err(e) => {
            warnings.push(format!("Failed to resolve auth storage path: {}", e));
            None
        }
    };

    // Try to get budget tracker storage path
    let budget_storage_path = match BudgetTracker::default_storage_path() {
        Ok(path) => Some(PathInfo::from_path(path)),
        Err(e) => {
            warnings.push(format!("Failed to resolve budget storage path: {}", e));
            None
        }
    };

    // Get execution mode settings
    let execution_mode = ExecutionMode::from_context(ctx);

    Ok(DoctorDiagnostics {
        auth_status,
        auth_storage_path,
        budget_storage_path,
        execution_mode,
        warnings: if warnings.is_empty() {
            None
        } else {
            Some(warnings)
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthToken;

    #[test]
    fn test_path_info_nonexistent() {
        let path = PathBuf::from("/nonexistent/path/to/file");
        let info = PathInfo::from_path(path);
        assert!(!info.exists);
        assert!(info.readable.is_none());
        assert!(info.writable.is_none());
    }

    #[test]
    fn test_execution_mode_from_context() {
        let ctx = ExecutionContext::new(true, None, Some(100), Some(500), true);
        let mode = ExecutionMode::from_context(&ctx);
        assert!(mode.non_interactive);
        assert!(mode.dry_run);
        assert_eq!(mode.max_cost_credits, Some(100));
        assert_eq!(mode.daily_budget_credits, Some(500));
    }

    #[test]
    fn test_collect_diagnostics_unauthenticated() {
        let auth_store = AuthStore::new();
        let ctx = ExecutionContext::new(false, None, None, None, false);
        let result = collect_diagnostics(&auth_store, &ctx);
        assert!(result.is_ok());
        let diagnostics = result.unwrap();
        assert!(!diagnostics.auth_status.authenticated);
        assert!(!diagnostics.execution_mode.non_interactive);
        assert!(!diagnostics.execution_mode.dry_run);
    }

    #[test]
    fn test_collect_diagnostics_authenticated() {
        let mut auth_store = AuthStore::new();
        let token = AuthToken {
            access_token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: None,
            scopes: vec!["read".to_string(), "write".to_string()],
        };
        auth_store.set_token(token);

        let ctx = ExecutionContext::new(true, Some("trace-123".to_string()), Some(50), None, true);
        let result = collect_diagnostics(&auth_store, &ctx);
        assert!(result.is_ok());
        let diagnostics = result.unwrap();
        assert!(diagnostics.auth_status.authenticated);
        assert!(diagnostics.execution_mode.non_interactive);
        assert!(diagnostics.execution_mode.dry_run);
        assert_eq!(diagnostics.execution_mode.max_cost_credits, Some(50));
    }
}
