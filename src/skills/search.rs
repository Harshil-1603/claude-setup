// src/skills/search.rs
use anyhow::Result;
use crate::config::paths;

/// Search installed skills by query (matches name, description, triggers).
pub fn search_local(query: &str) -> Result<()> {
    let skills_dir = paths::skills_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot resolve skills directory"))?;

    if !skills_dir.exists() {
        println!("No skills installed.");
        return Ok(());
    }

    let query_lower = query.to_lowercase();
    let mut found = false;

    for entry in std::fs::read_dir(&skills_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_md = path.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }

        let content = std::fs::read_to_string(&skill_md)?;
        if let Ok((manifest, _)) = crate::skills::manifest::parse(&content) {
            let matches_name = manifest.name.to_lowercase().contains(&query_lower);
            let matches_desc = manifest.description.to_lowercase().contains(&query_lower);
            let matches_triggers = manifest
                .triggers
                .iter()
                .any(|t| t.to_lowercase().contains(&query_lower));

            if matches_name || matches_desc || matches_triggers {
                println!(
                    "  {} — {}",
                    manifest.name, manifest.description
                );
                found = true;
            }
        }
    }

    if !found {
        println!("No skills found matching '{query}'.");
    }

    Ok(())
}
