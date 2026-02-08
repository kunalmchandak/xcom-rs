use crate::skills::models::Skill;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Discovers skills from the embedded skills directory
pub fn discover_skills(base_path: &Path) -> Result<Vec<Skill>> {
    let skills_dir = base_path.join("skills");

    if !skills_dir.exists() {
        return Ok(vec![]);
    }

    let mut skills = Vec::new();

    for entry in fs::read_dir(&skills_dir).context("Failed to read skills directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let skill_md = path.join("SKILL.md");
        if skill_md.exists() {
            let skill_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Try to extract description from SKILL.md (first line after frontmatter)
            let description = extract_description(&skill_md).ok();

            skills.push(Skill {
                name: skill_name,
                source_path: skill_md,
                description,
            });
        }
    }

    Ok(skills)
}

/// Find a specific skill by name
pub fn find_skill(base_path: &Path, skill_name: &str) -> Result<Option<Skill>> {
    let skill_dir = base_path.join("skills").join(skill_name);
    let skill_md = skill_dir.join("SKILL.md");

    if skill_md.exists() {
        let description = extract_description(&skill_md).ok();
        Ok(Some(Skill {
            name: skill_name.to_string(),
            source_path: skill_md,
            description,
        }))
    } else {
        Ok(None)
    }
}

/// Extract description from SKILL.md
fn extract_description(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path).context("Failed to read SKILL.md")?;

    // Simple heuristic: skip YAML frontmatter and take first non-empty line
    let mut in_frontmatter = false;
    let mut frontmatter_ended = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect frontmatter
        if trimmed == "---" {
            if !in_frontmatter && !frontmatter_ended {
                in_frontmatter = true;
                continue;
            } else if in_frontmatter {
                in_frontmatter = false;
                frontmatter_ended = true;
                continue;
            }
        }

        if in_frontmatter {
            continue;
        }

        // Skip empty lines and headings
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            return Ok(trimmed.to_string());
        }
    }

    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_skills_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let skills = discover_skills(temp_dir.path()).unwrap();
        assert_eq!(skills.len(), 0);
    }

    #[test]
    fn test_discover_skills_with_skill() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join("skills");
        let skill_dir = skills_dir.join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();

        let skill_md = skill_dir.join("SKILL.md");
        fs::write(&skill_md, "# Test Skill\n\nA test skill description.").unwrap();

        let skills = discover_skills(temp_dir.path()).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-skill");
    }

    #[test]
    fn test_find_skill_existing() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join("skills");
        let skill_dir = skills_dir.join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();

        let skill_md = skill_dir.join("SKILL.md");
        fs::write(&skill_md, "Test skill").unwrap();

        let skill = find_skill(temp_dir.path(), "test-skill").unwrap();
        assert!(skill.is_some());
        assert_eq!(skill.unwrap().name, "test-skill");
    }

    #[test]
    fn test_find_skill_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let skill = find_skill(temp_dir.path(), "nonexistent").unwrap();
        assert!(skill.is_none());
    }
}
