use crate::skills::models::{Skill, SkillInstallResult};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub global: bool,
    pub agent: Option<String>,
}

/// Install a skill to the canonical location and optional agent-specific paths
pub fn install_skill(skill: &Skill, options: &InstallOptions) -> Result<SkillInstallResult> {
    let canonical_path = resolve_canonical_path(&skill.name, options.global)?;

    // Create canonical directory
    if let Some(parent) = canonical_path.parent() {
        fs::create_dir_all(parent).context("Failed to create canonical directory")?;
    }

    // Copy SKILL.md to canonical location
    fs::copy(&skill.source_path, &canonical_path)
        .context("Failed to copy SKILL.md to canonical location")?;

    tracing::info!(
        skill = %skill.name,
        path = %canonical_path.display(),
        "Installed skill to canonical location"
    );

    // Create agent-specific paths if requested
    let mut target_paths = vec![canonical_path.clone()];
    if let Some(agent_name) = &options.agent {
        let agent_specific_paths = resolve_agent_paths(&skill.name, agent_name, options.global)?;
        for agent_path in agent_specific_paths {
            if let Err(e) = create_agent_link(&canonical_path, &agent_path) {
                tracing::warn!(
                    error = %e,
                    agent_path = %agent_path.display(),
                    "Failed to create agent-specific link, using copy fallback"
                );
                // Fallback to copy
                if let Some(parent) = agent_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&canonical_path, &agent_path)
                    .context("Failed to copy to agent-specific location")?;
            }
            target_paths.push(agent_path);
        }
    }

    Ok(SkillInstallResult::success(
        skill.name.clone(),
        canonical_path,
        target_paths,
        false, // We always use copy for now
    ))
}

/// Resolve canonical installation path
fn resolve_canonical_path(skill_name: &str, global: bool) -> Result<PathBuf> {
    let base = if global {
        dirs::home_dir()
            .context("Failed to get home directory")?
            .join(".agents")
    } else {
        PathBuf::from(".agents")
    };

    Ok(base.join("skills").join(skill_name).join("SKILL.md"))
}

/// Resolve agent-specific paths
fn resolve_agent_paths(skill_name: &str, agent: &str, global: bool) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    match agent {
        "claude" => {
            let base = if global {
                dirs::home_dir()
                    .context("Failed to get home directory")?
                    .join(".claude")
            } else {
                PathBuf::from(".claude")
            };
            paths.push(base.join("skills").join(skill_name).join("SKILL.md"));
        }
        "opencode" => {
            if global {
                let home = dirs::home_dir().context("Failed to get home directory")?;
                paths.push(
                    home.join(".config")
                        .join("opencode")
                        .join("skills")
                        .join(skill_name)
                        .join("SKILL.md"),
                );
            }
            // For project scope, opencode uses canonical only
        }
        _ => {
            tracing::warn!(agent = %agent, "Unknown agent, skipping agent-specific paths");
        }
    }

    Ok(paths)
}

/// Create link or copy to agent-specific location
fn create_agent_link(source: &Path, target: &Path) -> Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).context("Failed to create agent directory")?;
    }

    // Try symlink first (not available on all platforms)
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(source, target).context("Failed to create symlink")
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_file;
        symlink_file(source, target).context("Failed to create symlink")
    }

    #[cfg(not(any(unix, windows)))]
    {
        anyhow::bail!("Symlink not supported on this platform");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_install_skill_project_scope() {
        let temp_dir = TempDir::new().unwrap();
        let skill_md = temp_dir.path().join("test.md");
        fs::write(&skill_md, "Test skill content").unwrap();

        let skill = Skill {
            name: "test-skill".to_string(),
            source_path: skill_md,
            description: None,
        };

        let options = InstallOptions {
            global: false,
            agent: None,
        };

        // Change to temp dir for test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = install_skill(&skill, &options).unwrap();

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.success);
        assert_eq!(result.name, "test-skill");
        assert_eq!(result.target_paths.len(), 1);
        assert!(result.target_paths[0]
            .to_string_lossy()
            .contains("test-skill"));
    }

    #[test]
    fn test_resolve_canonical_path_project() {
        let path = resolve_canonical_path("test-skill", false).unwrap();
        assert!(path.to_string_lossy().contains(".agents"));
        assert!(path.to_string_lossy().contains("test-skill"));
        assert!(path.to_string_lossy().ends_with("SKILL.md"));
    }

    #[test]
    fn test_resolve_agent_paths_claude() {
        let paths = resolve_agent_paths("test-skill", "claude", false).unwrap();
        assert_eq!(paths.len(), 1);
        assert!(paths[0].to_string_lossy().contains(".claude"));
        assert!(paths[0].to_string_lossy().contains("test-skill"));
    }

    #[test]
    fn test_resolve_agent_paths_opencode_global() {
        let paths = resolve_agent_paths("test-skill", "opencode", true).unwrap();
        assert_eq!(paths.len(), 1);
        assert!(paths[0].to_string_lossy().contains("opencode"));
    }

    #[test]
    fn test_resolve_agent_paths_opencode_project() {
        let paths = resolve_agent_paths("test-skill", "opencode", false).unwrap();
        assert_eq!(paths.len(), 0); // Project scope uses canonical only
    }
}
