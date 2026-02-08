use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a discovered skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Skill name
    pub name: String,
    /// Path to SKILL.md file
    pub source_path: PathBuf,
    /// Description extracted from SKILL.md (optional)
    pub description: Option<String>,
}

/// Result of skill installation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInstallResult {
    /// Skill name
    pub name: String,
    /// Whether installation succeeded
    pub success: bool,
    /// All installation paths (canonical + agent-specific)
    pub target_paths: Vec<PathBuf>,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Whether symlink was used (true) or copy fallback (false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_symlink: Option<bool>,
}

impl SkillInstallResult {
    pub fn success(name: String, target_paths: Vec<PathBuf>, used_symlink: bool) -> Self {
        Self {
            name,
            success: true,
            target_paths,
            error: None,
            used_symlink: Some(used_symlink),
        }
    }

    pub fn failure(name: String, error: String) -> Self {
        Self {
            name,
            success: false,
            target_paths: vec![],
            error: Some(error),
            used_symlink: None,
        }
    }
}
