// src/skills/registry.rs
use crate::error::Result;

/// A skill entry from the remote registry.
#[derive(Debug, Clone)]
pub struct RegistrySkill {
    pub name: String,
    pub description: String,
    pub repo_url: String,
}

/// Search the remote skills.sh registry.
pub fn search_remote(query: &str) -> Result<Vec<RegistrySkill>> {
    let _url = format!(
        "https://skills.sh/api/skills?q={}",
        urlencoding::encode(query)
    );

    // For now, return empty — full registry integration in Phase 2
    // The `npx skills find` CLI can be used as a fallback
    tracing::info!("Registry search not yet implemented for query: {query}");
    Ok(vec![])
}

/// Get download URL for a skill by name.
pub fn get_download_url(name: &str) -> Result<String> {
    // Default to GitHub convention: owner/repo
    Ok(format!("https://github.com/{name}.git"))
}
