// src/skills/installer.rs
use anyhow::Result;
use crate::config::paths;
use crate::error::ClaudeEngError;

/// List all installed skills.
pub fn list_installed() -> Result<()> {
    let skills_dir = paths::skills_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot resolve skills directory"))?;

    if !skills_dir.exists() {
        println!("No skills installed.");
        return Ok(());
    }

    let mut found = false;
    for entry in std::fs::read_dir(&skills_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let skill_md = path.join("SKILL.md");
            if skill_md.exists() {
                let content = std::fs::read_to_string(&skill_md)?;
                match crate::skills::manifest::parse(&content) {
                    Ok((manifest, _)) => {
                        println!(
                            "  {} — {} (v{})",
                            manifest.name,
                            manifest.description,
                            if manifest.version.is_empty() { "unversioned" } else { &manifest.version }
                        );
                        found = true;
                    }
                    Err(_) => {
                        println!("  <invalid> — {}", path.display());
                        found = true;
                    }
                }
            }
        }
    }

    if !found {
        println!("No skills installed.");
    }

    Ok(())
}

/// Install a skill from a git repository URL.
pub fn install_from_git(name: &str, url: &str) -> Result<()> {
    let skills_dir = paths::skills_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot resolve skills directory"))?;

    let skill_path = skills_dir.join(name);

    if skill_path.exists() {
        return Err(ClaudeEngError::SkillAlreadyInstalled {
            name: name.to_string(),
        }
        .into());
    }

    std::fs::create_dir_all(&skills_dir)?;

    // Clone the repository
    git2::Repository::clone(url, &skill_path)
        .map_err(|e| ClaudeEngError::GitError {
            operation: format!("clone {url}"),
            source: e,
        })?;

    // Verify it has a SKILL.md
    let skill_md = skill_path.join("SKILL.md");
    if !skill_md.exists() {
        // Cleanup
        std::fs::remove_dir_all(&skill_path)?;
        anyhow::bail!("Repository does not contain a SKILL.md file");
    }

    println!("Installed skill: {name}");
    Ok(())
}

/// Install a skill from the registry by name (e.g., "owner/repo").
pub fn install_from_registry(name: &str) -> Result<()> {
    let url = crate::skills::registry::get_download_url(name)?;
    install_from_git(name, &url)
}

/// Uninstall a skill by name.
pub fn uninstall(name: &str) -> Result<()> {
    let skills_dir = paths::skills_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot resolve skills directory"))?;

    let skill_path = skills_dir.join(name);

    if !skill_path.exists() {
        return Err(ClaudeEngError::SkillNotFound {
            name: name.to_string(),
        }
        .into());
    }

    std::fs::remove_dir_all(&skill_path)?;
    println!("Removed skill: {name}");
    Ok(())
}
