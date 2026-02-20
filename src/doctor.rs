use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::auth::{AuthStatus, AuthStore};
use crate::billing::BudgetTracker;
use crate::context::ExecutionContext;

/// Required OAuth scopes for full functionality
const REQUIRED_SCOPES: &[&str] = &[
    "tweet.read",
    "tweet.write",
    "users.read",
    "bookmark.read",
    "bookmark.write",
    "like.read",
    "like.write",
    "offline.access",
];

/// Diagnostic information about the system configuration and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorDiagnostics {
    /// Authentication status
    #[serde(rename = "authStatus")]
    pub auth_status: AuthStatus,

    /// Budget tracker storage path resolution
    #[serde(rename = "budgetStoragePath", skip_serializing_if = "Option::is_none")]
    pub budget_storage_path: Option<PathInfo>,

    /// Execution mode settings
    #[serde(rename = "executionMode")]
    pub execution_mode: ExecutionMode,

    /// Scope compatibility check result
    #[serde(rename = "scopeCheck")]
    pub scope_check: ScopeCheck,

    /// API probe result (always present; status is "skipped" when --probe was not specified)
    #[serde(rename = "apiProbe")]
    pub api_probe: ApiProbeResult,

    /// Any warnings encountered during diagnostics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,

    /// Next steps to resolve issues (populated on failures)
    #[serde(rename = "nextSteps", skip_serializing_if = "Option::is_none")]
    pub next_steps: Option<Vec<String>>,
}

/// Scope compatibility check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeCheck {
    /// Whether all required scopes are present
    pub ok: bool,

    /// Scopes that are present in the token
    #[serde(rename = "grantedScopes")]
    pub granted_scopes: Vec<String>,

    /// Required scopes that are missing
    #[serde(rename = "missingScopes")]
    pub missing_scopes: Vec<String>,
}

impl ScopeCheck {
    /// Evaluate scope compatibility from the granted scopes list
    pub fn evaluate(granted_scopes: &[String]) -> Self {
        let granted: std::collections::HashSet<&str> =
            granted_scopes.iter().map(|s| s.as_str()).collect();
        let missing: Vec<String> = REQUIRED_SCOPES
            .iter()
            .filter(|&&s| !granted.contains(s))
            .map(|&s| s.to_string())
            .collect();
        Self {
            ok: missing.is_empty(),
            granted_scopes: granted_scopes.to_vec(),
            missing_scopes: missing,
        }
    }

    /// Return a scope check for an unauthenticated user (all required scopes missing)
    pub fn unauthenticated() -> Self {
        Self {
            ok: false,
            granted_scopes: vec![],
            missing_scopes: REQUIRED_SCOPES.iter().map(|&s| s.to_string()).collect(),
        }
    }
}

/// Status of an API probe
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProbeStatus {
    /// Probe was not requested
    Skipped,
    /// Probe succeeded
    Ok,
    /// Probe failed
    Failed,
}

/// Result of an API connectivity probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProbeResult {
    /// Probe status
    pub status: ProbeStatus,

    /// Duration of the probe in milliseconds (0 when skipped)
    #[serde(rename = "durationMs")]
    pub duration_ms: u64,

    /// HTTP status code returned (if probe was executed)
    #[serde(rename = "httpStatus", skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,

    /// Human-readable message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl ApiProbeResult {
    /// Create a skipped probe result
    pub fn skipped() -> Self {
        Self {
            status: ProbeStatus::Skipped,
            duration_ms: 0,
            http_status: None,
            message: Some("Probe not requested; pass --probe to enable".to_string()),
        }
    }

    /// Create a successful probe result
    pub fn ok(http_status: u16, duration_ms: u64) -> Self {
        Self {
            status: ProbeStatus::Ok,
            duration_ms,
            http_status: Some(http_status),
            message: Some("API is reachable".to_string()),
        }
    }

    /// Create a failed probe result
    pub fn failed(message: String, duration_ms: u64) -> Self {
        Self {
            status: ProbeStatus::Failed,
            duration_ms,
            http_status: None,
            message: Some(message),
        }
    }

    /// Create a failed probe result with an HTTP status code
    pub fn failed_with_status(http_status: u16, message: String, duration_ms: u64) -> Self {
        Self {
            status: ProbeStatus::Failed,
            duration_ms,
            http_status: Some(http_status),
            message: Some(message),
        }
    }
}

/// Trait for performing an API connectivity probe.
/// This abstraction allows test code to inject a mock without touching the network.
pub trait ApiProber: Send + Sync {
    /// Execute a lightweight probe against the X API.
    /// Returns `Ok(ApiProbeResult)` for both successful and failed probes;
    /// `Err` only for unexpected internal errors.
    fn probe(&self) -> Result<ApiProbeResult>;
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

/// Collect diagnostic information.
///
/// When `prober` is `Some`, the API connectivity probe will be executed.
/// When `prober` is `None`, the probe result will be `skipped`.
pub fn collect_diagnostics(
    auth_store: &AuthStore,
    ctx: &ExecutionContext,
    prober: Option<&dyn ApiProber>,
) -> Result<DoctorDiagnostics> {
    let mut warnings = Vec::new();
    let mut next_steps: Vec<String> = Vec::new();

    // Get auth status
    let auth_status = auth_store.status();

    // Scope check
    let scope_check = if auth_status.authenticated {
        // Check if scopes are provided; if not, skip scope diagnostics
        if let Some(granted_scopes) = auth_status.scopes.clone() {
            let check = ScopeCheck::evaluate(&granted_scopes);
            if !check.ok {
                warnings.push(format!(
                    "Missing required OAuth scopes: {}",
                    check.missing_scopes.join(", ")
                ));
                next_steps.push("Re-authenticate with the required scopes".to_string());
                next_steps.push(format!(
                    "Missing scopes: {}",
                    check.missing_scopes.join(", ")
                ));
            }
            check
        } else {
            // XCOM_RS_SCOPES not set; skip scope diagnostics
            warnings.push("XCOM_RS_SCOPES not set; scope diagnostics skipped".to_string());
            next_steps.push("Set XCOM_RS_SCOPES to enable scope diagnostics".to_string());
            ScopeCheck {
                ok: false,
                granted_scopes: vec![],
                missing_scopes: vec![],
            }
        }
    } else {
        next_steps.push("Set XCOM_RS_BEARER_TOKEN and re-run the command".to_string());
        ScopeCheck::unauthenticated()
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

    // Run API probe if requested; always include apiProbe in the output
    let api_probe = match prober {
        Some(p) => {
            let result = p.probe()?;
            if result.status == ProbeStatus::Failed {
                if let Some(ref msg) = result.message {
                    warnings.push(format!("API probe failed: {}", msg));
                }
                next_steps.push("Check network connectivity to api.twitter.com".to_string());
                next_steps
                    .push("Verify that your access token is valid and not expired".to_string());
            }
            result
        }
        None => {
            // Probe was not requested; advise the user how to enable it
            next_steps.push(
                "To verify API connectivity, re-run with --probe: xcom-rs doctor --probe"
                    .to_string(),
            );
            ApiProbeResult::skipped()
        }
    };

    Ok(DoctorDiagnostics {
        auth_status,
        budget_storage_path,
        execution_mode,
        scope_check,
        api_probe,
        warnings: if warnings.is_empty() {
            None
        } else {
            Some(warnings)
        },
        next_steps: if next_steps.is_empty() {
            None
        } else {
            Some(next_steps)
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------------------
    // Mock prober for testing
    // ---------------------------------------------------------------------------

    struct MockProber {
        result: ApiProbeResult,
    }

    impl MockProber {
        fn ok() -> Self {
            Self {
                result: ApiProbeResult::ok(200, 42),
            }
        }

        fn failed(msg: &str) -> Self {
            Self {
                result: ApiProbeResult::failed(msg.to_string(), 100),
            }
        }
    }

    impl ApiProber for MockProber {
        fn probe(&self) -> Result<ApiProbeResult> {
            Ok(self.result.clone())
        }
    }

    // ---------------------------------------------------------------------------
    // PathInfo tests
    // ---------------------------------------------------------------------------

    #[test]
    fn test_path_info_nonexistent() {
        let path = PathBuf::from("/nonexistent/path/to/file");
        let info = PathInfo::from_path(path);
        assert!(!info.exists);
        assert!(info.readable.is_none());
        assert!(info.writable.is_none());
    }

    // ---------------------------------------------------------------------------
    // ExecutionMode tests
    // ---------------------------------------------------------------------------

    #[test]
    fn test_execution_mode_from_context() {
        let ctx = ExecutionContext::new(true, None, Some(100), Some(500), true);
        let mode = ExecutionMode::from_context(&ctx);
        assert!(mode.non_interactive);
        assert!(mode.dry_run);
        assert_eq!(mode.max_cost_credits, Some(100));
        assert_eq!(mode.daily_budget_credits, Some(500));
    }

    // ---------------------------------------------------------------------------
    // ScopeCheck tests
    // ---------------------------------------------------------------------------

    #[test]
    fn test_scope_check_all_present() {
        let granted: Vec<String> = REQUIRED_SCOPES.iter().map(|&s| s.to_string()).collect();
        let check = ScopeCheck::evaluate(&granted);
        assert!(check.ok);
        assert!(check.missing_scopes.is_empty());
    }

    #[test]
    fn test_scope_check_missing_scopes() {
        let granted = vec!["tweet.read".to_string(), "users.read".to_string()];
        let check = ScopeCheck::evaluate(&granted);
        assert!(!check.ok);
        assert!(check.missing_scopes.contains(&"tweet.write".to_string()));
        assert!(!check.missing_scopes.contains(&"tweet.read".to_string()));
    }

    #[test]
    fn test_scope_check_unauthenticated() {
        let check = ScopeCheck::unauthenticated();
        assert!(!check.ok);
        assert!(check.granted_scopes.is_empty());
        assert_eq!(check.missing_scopes.len(), REQUIRED_SCOPES.len());
    }

    // ---------------------------------------------------------------------------
    // collect_diagnostics tests
    // ---------------------------------------------------------------------------

    #[test]
    fn test_collect_diagnostics_unauthenticated_no_probe() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_RS_BEARER_TOKEN");

        let auth_store = AuthStore::new();
        let ctx = ExecutionContext::new(false, None, None, None, false);
        let result = collect_diagnostics(&auth_store, &ctx, None);
        assert!(result.is_ok());
        let diagnostics = result.unwrap();
        assert!(!diagnostics.auth_status.authenticated);
        assert!(!diagnostics.execution_mode.non_interactive);
        assert!(!diagnostics.execution_mode.dry_run);
        // Probe not requested → status is "skipped"
        assert_eq!(diagnostics.api_probe.status, ProbeStatus::Skipped);
        assert_eq!(diagnostics.api_probe.duration_ms, 0);
        // next_steps should include probe hint and auth setup
        let next_steps = diagnostics.next_steps.unwrap_or_default();
        assert!(next_steps.iter().any(|s| s.contains("--probe")));
        assert!(next_steps
            .iter()
            .any(|s| s.contains("XCOM_RS_BEARER_TOKEN")));
        // Scope check shows all missing
        assert!(!diagnostics.scope_check.ok);
        assert_eq!(
            diagnostics.scope_check.missing_scopes.len(),
            REQUIRED_SCOPES.len()
        );
    }

    #[test]
    fn test_collect_diagnostics_authenticated_full_scopes_no_probe() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");
        let scopes: Vec<String> = REQUIRED_SCOPES.iter().map(|&s| s.to_string()).collect();
        std::env::set_var("XCOM_RS_SCOPES", scopes.join(" "));

        let auth_store = AuthStore::new();
        let ctx = ExecutionContext::new(true, Some("trace-123".to_string()), Some(50), None, true);
        let result = collect_diagnostics(&auth_store, &ctx, None);
        assert!(result.is_ok());
        let diagnostics = result.unwrap();
        assert!(diagnostics.auth_status.authenticated);
        assert!(diagnostics.execution_mode.non_interactive);
        assert!(diagnostics.execution_mode.dry_run);
        assert_eq!(diagnostics.execution_mode.max_cost_credits, Some(50));
        // All scopes present → ok
        assert!(diagnostics.scope_check.ok);
        assert!(diagnostics.scope_check.missing_scopes.is_empty());
        // No probe requested → status is "skipped"
        assert_eq!(diagnostics.api_probe.status, ProbeStatus::Skipped);
    }

    #[test]
    fn test_collect_diagnostics_authenticated_missing_scopes() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("XCOM_RS_BEARER_TOKEN", "test_token");
        std::env::set_var("XCOM_RS_SCOPES", "tweet.read");

        let auth_store = AuthStore::new();
        let ctx = ExecutionContext::new(false, None, None, None, false);
        let result = collect_diagnostics(&auth_store, &ctx, None);
        assert!(result.is_ok());
        let diagnostics = result.unwrap();
        assert!(diagnostics.auth_status.authenticated);
        assert!(!diagnostics.scope_check.ok);
        assert!(!diagnostics.scope_check.missing_scopes.is_empty());
        // Warning should mention missing scopes
        let warnings = diagnostics.warnings.unwrap_or_default();
        assert!(warnings
            .iter()
            .any(|w| w.contains("Missing required OAuth scopes")));
    }

    #[test]
    fn test_collect_diagnostics_with_probe_success() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_RS_BEARER_TOKEN");

        let auth_store = AuthStore::new();
        let ctx = ExecutionContext::new(false, None, None, None, false);
        let prober = MockProber::ok();
        let result = collect_diagnostics(&auth_store, &ctx, Some(&prober));
        assert!(result.is_ok());
        let diagnostics = result.unwrap();
        let probe = &diagnostics.api_probe;
        assert_eq!(probe.status, ProbeStatus::Ok);
        assert_eq!(probe.http_status, Some(200));
        // durationMs should be set (42 from mock)
        assert_eq!(probe.duration_ms, 42);
    }

    #[test]
    fn test_collect_diagnostics_with_probe_failure() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_RS_BEARER_TOKEN");

        let auth_store = AuthStore::new();
        let ctx = ExecutionContext::new(false, None, None, None, false);
        let prober = MockProber::failed("connection refused");
        let result = collect_diagnostics(&auth_store, &ctx, Some(&prober));
        assert!(result.is_ok());
        let diagnostics = result.unwrap();
        let probe = &diagnostics.api_probe;
        assert_eq!(probe.status, ProbeStatus::Failed);
        assert_eq!(probe.duration_ms, 100);
        // Warnings and next_steps should be populated
        let warnings = diagnostics.warnings.unwrap_or_default();
        assert!(warnings.iter().any(|w| w.contains("API probe failed")));
        let next_steps = diagnostics.next_steps.unwrap_or_default();
        assert!(next_steps.iter().any(|s| s.contains("network")));
    }

    #[test]
    fn test_collect_diagnostics_skipped_probe_returns_skipped_status() {
        let _guard = crate::test_utils::env_lock::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("XCOM_RS_BEARER_TOKEN");

        // Passing None means probe is skipped; api_probe is always present with status=skipped
        let auth_store = AuthStore::new();
        let ctx = ExecutionContext::new(false, None, None, None, false);
        let diagnostics = collect_diagnostics(&auth_store, &ctx, None).unwrap();
        assert_eq!(diagnostics.api_probe.status, ProbeStatus::Skipped);
        assert_eq!(diagnostics.api_probe.duration_ms, 0);
        // next_steps must include --probe hint
        let next_steps = diagnostics.next_steps.unwrap_or_default();
        assert!(next_steps.iter().any(|s| s.contains("--probe")));
    }
}
