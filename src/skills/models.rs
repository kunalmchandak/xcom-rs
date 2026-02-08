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
    pub skill: String,
    /// Whether installation succeeded
    pub success: bool,
    /// Canonical installation path
    pub canonical_path: PathBuf,
    /// Additional agent-specific paths (e.g., .claude/skills)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub agent_paths: Vec<PathBuf>,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Whether symlink was used (true) or copy fallback (false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_symlink: Option<bool>,
}

impl SkillInstallResult {
    pub fn success(
        skill: String,
        canonical_path: PathBuf,
        agent_paths: Vec<PathBuf>,
        used_symlink: bool,
    ) -> Self {
        Self {
            skill,
            success: true,
            canonical_path,
            agent_paths,
            error: None,
            used_symlink: Some(used_symlink),
        }
    }

    pub fn failure(skill: String, error: String) -> Self {
        Self {
            skill,
            success: false,
            canonical_path: PathBuf::new(),
            agent_paths: vec![],
            error: Some(error),
            used_symlink: None,
        }
    }
}
