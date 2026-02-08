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
    if !yes && !ctx.non_interactive {
        tracing::info!("Interactive confirmation would be required (--yes not specified)");
        // In non-interactive mode, this would fail, but for now we proceed
    }

    if ctx.non_interactive && !yes {
        let error = ErrorDetails::new(
            ErrorCode::InteractionRequired,
            "Interactive confirmation required but --non-interactive mode is enabled. Use --yes to skip confirmation.".to_string(),
        );
        let envelope = if let Some(meta) = create_meta() {
            Envelope::<()>::error_with_meta("install-skills", error, meta)
        } else {
            Envelope::<()>::error("install-skills", error)
        };
        print_envelope(&envelope, output_format)?;
        return Err(anyhow::anyhow!("Interaction required"));
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
                results.push(SkillInstallResult::failure(
                    skill.name.clone(),
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
