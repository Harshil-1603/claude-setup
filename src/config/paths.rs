// src/config/paths.rs
use std::path::PathBuf;

/// Returns the Claude config directory (~/.claude).
pub fn claude_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude"))
}

/// Returns ~/.claude/CLAUDE.md
pub fn claude_md_path() -> Option<PathBuf> {
    claude_dir().map(|d| d.join("CLAUDE.md"))
}

/// Returns ~/.claude/CLAUDE.local.md
pub fn claude_local_md_path() -> Option<PathBuf> {
    claude_dir().map(|d| d.join("CLAUDE.local.md"))
}

/// Returns ~/.claude/settings.json
pub fn settings_json_path() -> Option<PathBuf> {
    claude_dir().map(|d| d.join("settings.json"))
}

/// Returns ~/.claude/skills/
pub fn skills_dir() -> Option<PathBuf> {
    claude_dir().map(|d| d.join("skills"))
}

/// Returns ~/.claude/backups/
pub fn backups_dir() -> Option<PathBuf> {
    claude_dir().map(|d| d.join("backups"))
}
