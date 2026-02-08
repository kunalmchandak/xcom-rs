use crate::{
    context::ExecutionContext,
    output::{print_envelope, OutputFormat},
    protocol::{Envelope, ErrorCode, ErrorDetails},
    skills::{discover_skills, find_skill, install_skill, InstallOptions, SkillInstallResult},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallSkillsResponse {
    pub installed_skills: Vec<SkillInstallResult>,
}

pub fn handle_install_skills(
    skill_filter: Option<&str>,
    agent: Option<&str>,
    global: bool,
    yes: bool,
    ctx: &ExecutionContext,
    create_meta: &dyn Fn() -> Option<HashMap<String, serde_json::Value>>,
    output_format: OutputFormat,
) -> Result<()> {
    tracing::info!(
        skill = ?skill_filter,
        agent = ?agent,
        global = global,
        "Executing install-skills command"
    );

    // Get repository root (current directory)
    let repo_root = env::current_dir()?;

    // Discover or find specific skill
    let skills_to_install = if let Some(skill_name) = skill_filter {
        // Find specific skill
        match find_skill(&repo_root, skill_name)? {
            Some(skill) => vec![skill],
            None => {
                let error = ErrorDetails::new(
                    ErrorCode::InvalidArgument,
                    format!("Skill '{}' not found in skills/ directory", skill_name),
                );
                let envelope = if let Some(meta) = create_meta() {
                    Envelope::<()>::error_with_meta("install-skills", error, meta)
                } else {
                    Envelope::<()>::error("install-skills", error)
                };
                print_envelope(&envelope, output_format)?;
                return Err(anyhow::anyhow!("Skill not found"));
            }
        }
    } else {
        // Discover all skills
        discover_skills(&repo_root)?
    };

    if skills_to_install.is_empty() {
        let error = ErrorDetails::new(
            ErrorCode::InvalidArgument,
            "No skills found in skills/ directory".to_string(),
        );
        let envelope = if let Some(meta) = create_meta() {
            Envelope::<()>::error_with_meta("install-skills", error, meta)
        } else {
            Envelope::<()>::error("install-skills", error)
        };
        print_envelope(&envelope, output_format)?;
        return Err(anyhow::anyhow!("No skills found"));
    }

    // Check for interactive requirement
    // In non-interactive mode, auto-confirm (treat as --yes)
    // Otherwise, if --yes is not specified, would require interactive confirmation
    let auto_confirm = yes || ctx.non_interactive;

    if !auto_confirm {
        tracing::info!("Interactive confirmation would be required (--yes not specified and not in non-interactive mode)");
        // In a real interactive scenario, we would prompt here
        // For now, we treat missing --yes in interactive mode as implicit confirmation
    }

    // Install skills
    let options = InstallOptions {
        global,
        agent: agent.map(|s| s.to_string()),
    };

    let mut results = Vec::new();
    for skill in skills_to_install {
        match install_skill(&skill, &options) {
            Ok(result) => {
                tracing::info!(skill = %skill.name, "Successfully installed skill");
                results.push(result);
            }
            Err(e) => {
                tracing::error!(skill = %skill.name, error = %e, "Failed to install skill");
                // Compute canonical path even for failures for consistency
                let canonical_path = if global {
                    dirs::home_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("~"))
                        .join(".agents")
                        .join("skills")
                        .join(&skill.name)
                        .join("SKILL.md")
                } else {
                    std::path::PathBuf::from(".agents")
                        .join("skills")
                        .join(&skill.name)
                        .join("SKILL.md")
                };
                results.push(SkillInstallResult::failure(
                    skill.name.clone(),
                    canonical_path,
                    e.to_string(),
                ));
            }
        }
    }

    let response = InstallSkillsResponse {
        installed_skills: results,
    };

    let envelope = if let Some(meta) = create_meta() {
        Envelope::success_with_meta("install-skills", response, meta)
    } else {
        Envelope::success("install-skills", response)
    };

    print_envelope(&envelope, output_format)
}
