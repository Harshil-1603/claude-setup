// src/config/claude_md.rs
use anyhow::Result;

/// Generate the base CLAUDE.md content from the template.
pub fn generate_base() -> Result<String> {
    Ok(crate::config::templates::claude_md_template().to_string())
}

/// Merge the base CLAUDE.md with the user's CLAUDE.local.md if it exists.
pub fn merge_with_local(base: &str, local: Option<&str>) -> String {
    match local {
        Some(local_content) => format!("{base}\n\n---\n\n# User Overrides\n\n{local_content}"),
        None => base.to_string(),
    }
}

/// Write CLAUDE.md to ~/.claude/CLAUDE.md (atomic write with backup).
pub fn write(content: &str) -> Result<()> {
    let path = crate::config::paths::claude_md_path()
        .ok_or_else(|| anyhow::anyhow!("Cannot resolve ~/.claude/CLAUDE.md path"))?;

    // Backup existing file if it exists
    if path.exists() {
        let backup_dir = crate::config::paths::backups_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot resolve backups directory"))?;
        std::fs::create_dir_all(&backup_dir)?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let backup_path = backup_dir.join(format!("CLAUDE.md.{timestamp}.bak"));
        std::fs::copy(&path, &backup_path)?;
    }

    // Atomic write: write to temp, then rename
    let parent = path.parent().ok_or_else(|| anyhow::anyhow!("No parent dir"))?;
    let temp = tempfile::NamedTempFile::new_in(parent)?;
    std::io::Write::write_all(&mut temp.as_file(), content.as_bytes())?;
    temp.persist(&path)?;

    Ok(())
}
