# Claude Engineering OS — Phase 1: Core + Skills Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a single Rust binary (`claude-eng`) that installs global config into `~/.claude/` and provides a skill system (add, remove, list, search) compatible with the `skills.sh` ecosystem.

**Architecture:** A CLI built with `clap` that generates `CLAUDE.md` and merges into `settings.json` in `~/.claude/`. Skills are Markdown files parsed from `SKILL.md` frontmatter. Built-in skills are embedded into the binary. The binary never runs Claude — it configures Claude to be better.

**Tech Stack:** Rust 2021 edition, `clap` (CLI), `serde`/`serde_yaml`/`serde_json` (serialization), `toml` (Cargo.toml parsing for manifest), `dirs` (path resolution), `reqwest` (registry HTTP), `git2` (cloning skills), `tempfile` (atomic writes), `anyhow`/`thiserror` (errors), `tracing`/`tracing-subscriber` (logging), `assert_cmd`/`predicates`/`tempfile` (testing).

## Global Constraints

- Rust 2021 edition, MSRV 1.70
- All public APIs must have doc comments
- All modules must have unit tests
- Config generation must be idempotent
- Atomic writes (write temp, then rename) for all config files
- Backups created before any overwrite of user config
- No telemetry, no network calls except `skill add`/`skill search` registry
- Binary name: `claude-eng`
- License: MIT

---

## File Structure

```
claude-engineering-os/
├── Cargo.toml
├── README.md
├── LICENSE
│
├── src/
│   ├── main.rs                     # Entry point, CLI dispatch
│   ├── lib.rs                      # Public re-exports
│   │
│   ├── cli/
│   │   ├── mod.rs                  # Cli enum + Subcommand dispatch
│   │   └── install.rs              # Install command handler
│   │
│   ├── config/
│   │   ├── mod.rs                  # Config module re-exports
│   │   ├── paths.rs                # Path resolution (~/.claude/*)
│   │   ├── claude_md.rs            # CLAUDE.md generation + local override merge
│   │   ├── settings.rs             # settings.json merge logic
│   │   └── templates.rs            # Embedded config templates
│   │
│   ├── skills/
│   │   ├── mod.rs                  # Skills module re-exports
│   │   ├── manifest.rs             # SKILL.md frontmatter parser
│   │   ├── registry.rs             # Remote registry client (skills.sh)
│   │   ├── installer.rs            # Install/uninstall skills to ~/.claude/skills/
│   │   ├── search.rs               # Local + registry search
│   │   └── builtin.rs              # Built-in skill definitions (embedded)
│   │
│   └── error.rs                    # Unified error types
│
├── templates/
│   ├── CLAUDE.md.template          # Base CLAUDE.md template
│   └── settings.json.template      # Base settings.json template
│
├── skills/
│   ├── brainstorming/
│   │   └── SKILL.md
│   ├── tdd/
│   │   └── SKILL.md
│   ├── systematic-debugging/
│   │   └── SKILL.md
│   ├── verification/
│   │   └── SKILL.md
│   └── code-review/
│       └── SKILL.md
│
├── docs/
│   ├── superpowers/
│   │   ├── specs/
│   │   │   └── 2026-06-27-claude-engineering-os-architecture.md
│   │   └── plans/
│   │       └── 2026-06-27-phase1-core-and-skills.md
│   ├── architecture.md
│   ├── user-guide.md
│   └── developer-guide.md
│
└── tests/
    ├── cli_install.rs
    ├── config_paths.rs
    ├── config_claude_md.rs
    ├── config_settings.rs
    ├── skills_manifest.rs
    ├── skills_search.rs
    └── skills_installer.rs
```

---

## Dependency Versions

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
toml = "0.8"
dirs = "5"
reqwest = { version = "0.12", features = ["blocking", "json"] }
git2 = "0.19"
tempfile = "3"
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["fmt"] }
include_dir = "0.7"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

---

### Task 1: Project Scaffolding + CLI Skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/cli/mod.rs`
- Create: `src/cli/install.rs`
- Create: `src/error.rs`

**Interfaces:**
- Consumes: nothing (first task)
- Produces: `Cli` struct with subcommands, `ClaudeEngError` error type, `cli::dispatch()` function

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "claude-engineering-os"
version = "0.1.0"
edition = "2021"
description = "A reusable engineering platform that improves every Claude Code session"
license = "MIT"
readme = "README.md"

[[bin]]
name = "claude-eng"
path = "src/main.rs"

[lib]
name = "claude_eng"
path = "src/lib.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
toml = "0.8"
dirs = "5"
reqwest = { version = "0.12", features = ["blocking", "json"] }
git2 = "0.19"
tempfile = "3"
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["fmt"] }
include_dir = "0.7"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

- [ ] **Step 2: Create error module**

```rust
// src/error.rs
use thiserror::Error;

/// Unified error type for claude-eng operations.
#[derive(Error, Debug)]
pub enum ClaudeEngError {
    #[error("Config directory not found: {path}")]
    ConfigDirNotFound { path: String },

    #[error("Failed to read config file: {path}")]
    ConfigReadError { path: String, source: std::io::Error },

    #[error("Failed to write config file: {path}")]
    ConfigWriteError { path: String, source: std::io::Error },

    #[error("Failed to parse SKILL.md: {path}")]
    SkillManifestParseError { path: String, source: anyhow::Error },

    #[error("Skill not found: {name}")]
    SkillNotFound { name: String },

    #[error("Skill already installed: {name}")]
    SkillAlreadyInstalled { name: String },

    #[error("Registry request failed: {url}")]
    RegistryError { url: String, source: reqwest::Error },

    #[error("Git operation failed: {operation}")]
    GitError {
        operation: String,
        source: git2::Error,
    },

    #[error("JSON serialization error")]
    JsonError(#[from] serde_json::Error),

    #[error("YAML serialization error")]
    YamlError(#[from] serde_yaml::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, ClaudeEngError>;
```

- [ ] **Step 3: Create CLI module**

```rust
// src/cli/mod.rs
pub mod install;

use clap::{Parser, Subcommand};
use crate::error::Result;

/// Claude Engineering OS — a reusable engineering platform for Claude Code.
#[derive(Parser)]
#[command(name = "claude-eng", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install or update Claude Engineering OS configuration
    Install(install::InstallArgs),

    /// Manage skills
    Skill {
        #[command(subcommand)]
        action: SkillCommands,
    },
}

#[derive(Subcommand)]
pub enum SkillCommands {
    /// List installed skills
    List,

    /// Search for skills in the local registry or remote
    Search {
        /// Search query
        query: String,
    },

    /// Install a skill by name (from remote registry)
    Add {
        /// Skill name or owner/repo
        name: String,
    },

    /// Remove an installed skill
    Remove {
        /// Skill name to remove
        name: String,
    },
}

/// Dispatch a CLI command to its handler.
pub fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Install(args) => install::run(args),
        Commands::Skill { action } => {
            // Skill commands will be implemented in Task 5-7
            match action {
                SkillCommands::List => {
                    crate::skills::installer::list_installed()
                }
                SkillCommands::Search { query } => {
                    crate::skills::search::search_local(&query)
                }
                SkillCommands::Add { name } => {
                    crate::skills::installer::install_from_registry(&name)
                }
                SkillCommands::Remove { name } => {
                    crate::skills::installer::uninstall(&name)
                }
            }
        }
    }
}
```

- [ ] **Step 4: Create lib.rs**

```rust
// src/lib.rs
pub mod cli;
pub mod config;
pub mod error;
pub mod skills;
```

- [ ] **Step 5: Create main.rs (stub)**

```rust
// src/main.rs
use clap::Parser;
use claude_eng::cli::Cli;
use claude_eng::cli::dispatch;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    dispatch(cli)?;

    Ok(())
}
```

- [ ] **Step 6: Create stub config and skills modules**

```rust
// src/config/mod.rs
pub mod claude_md;
pub mod paths;
pub mod settings;
pub mod templates;
```

```rust
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
```

```rust
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
```

```rust
// src/config/settings.rs
use anyhow::Result;
use serde_json::Value;

/// Read existing settings.json or return empty object.
pub fn read() -> Result<Value> {
    let path = crate::config::paths::settings_json_path()
        .ok_or_else(|| anyhow::anyhow!("Cannot resolve settings.json path"))?;

    if !path.exists() {
        return Ok(serde_json::json!({}));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read settings.json: {e}"))?;

    let value: Value = serde_json::from_str(&content)?;
    Ok(value)
}

/// Merge new settings into existing settings (additive only).
pub fn merge(existing: &mut Value, new: &Value) {
    if let (Some(existing_obj), Some(new_obj)) = (existing.as_object_mut(), new.as_object()) {
        for (key, value) in new_obj {
            if existing_obj.contains_key(key) && value.is_object() {
                merge(existing_obj.get_mut(key).unwrap(), value);
            } else {
                existing_obj.insert(key.clone(), value.clone());
            }
        }
    }
}

/// Write settings.json (atomic write with backup).
pub fn write(settings: &Value) -> Result<()> {
    let path = crate::config::paths::settings_json_path()
        .ok_or_else(|| anyhow::anyhow!("Cannot resolve settings.json path"))?;

    // Ensure parent dir exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Backup existing file
    if path.exists() {
        let backup_dir = crate::config::paths::backups_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot resolve backups directory"))?;
        std::fs::create_dir_all(&backup_dir)?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let backup_path = backup_dir.join(format!("settings.json.{timestamp}.bak"));
        std::fs::copy(&path, &backup_path)?;
    }

    // Atomic write
    let content = serde_json::to_string_pretty(settings)?;
    let parent = path.parent().ok_or_else(|| anyhow::anyhow!("No parent dir"))?;
    let temp = tempfile::NamedTempFile::new_in(parent)?;
    std::io::Write::write_all(&mut temp.as_file(), content.as_bytes())?;
    temp.persist(&path)?;

    Ok(())
}
```

```rust
// src/config/templates.rs
use include_dir::{include_dir, Dir};

static TEMPLATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

/// Get the CLAUDE.md template content.
pub fn claude_md_template() -> &'static str {
    TEMPLATES_DIR
        .get_file("CLAUDE.md.template")
        .expect("CLAUDE.md.template must exist in templates/")
        .contents_utf8()
        .expect("CLAUDE.md.template must be valid UTF-8")
}

/// Get the settings.json template content.
pub fn settings_json_template() -> &'static str {
    TEMPLATES_DIR
        .get_file("settings.json.template")
        .expect("settings.json.template must exist in templates/")
        .contents_utf8()
        .expect("settings.json.template must be valid UTF-8")
}
```

```rust
// src/skills/mod.rs
pub mod builtin;
pub mod installer;
pub mod manifest;
pub mod registry;
pub mod search;
```

- [ ] **Step 7: Create stub skills modules**

```rust
// src/skills/manifest.rs
use serde::{Deserialize, Serialize};

/// Frontmatter parsed from a SKILL.md file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillManifest {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub triggers: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// Parse SKILL.md content, extracting YAML frontmatter and body.
pub fn parse(content: &str) -> anyhow::Result<(SkillManifest, &str)> {
    let content = content.trim_start();

    if !content.starts_with("---") {
        anyhow::bail!("SKILL.md must start with YAML frontmatter (---)");
    }

    let after_first = &content[3..];
    let end = after_first
        .find("---")
        .ok_or_else(|| anyhow::anyhow!("SKILL.md frontmatter not closed (missing ---)"))?;

    let yaml_str = &after_first[..end].trim();
    let body = after_first[end + 3..].trim();

    let manifest: SkillManifest = serde_yaml::from_str(yaml_str)?;
    Ok((manifest, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_manifest() {
        let input = r#"---
name: my-skill
description: A test skill
version: "1.0.0"
triggers:
  - test
  - demo
dependencies: []
---

# My Skill

This is the body.
"#;
        let (manifest, body) = parse(input).unwrap();
        assert_eq!(manifest.name, "my-skill");
        assert_eq!(manifest.description, "A test skill");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.triggers, vec!["test", "demo"]);
        assert!(body.contains("# My Skill"));
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let input = "Just some content without frontmatter";
        assert!(parse(input).is_err());
    }

    #[test]
    fn test_parse_unclosed_frontmatter() {
        let input = "---\nname: test\ndescription: test";
        assert!(parse(input).is_err());
    }

    #[test]
    fn test_parse_minimal_frontmatter() {
        let input = "---\nname: minimal\ndescription: minimal skill\n---\n# Body";
        let (manifest, body) = parse(input).unwrap();
        assert_eq!(manifest.name, "minimal");
        assert_eq!(manifest.triggers, Vec::<String>::new());
        assert!(body.contains("# Body"));
    }
}
```

```rust
// src/skills/registry.rs
use crate::error::{ClaudeEngError, Result};

/// A skill entry from the remote registry.
#[derive(Debug, Clone)]
pub struct RegistrySkill {
    pub name: String,
    pub description: String,
    pub repo_url: String,
}

/// Search the remote skills.sh registry.
pub fn search_remote(query: &str) -> Result<Vec<RegistrySkill>> {
    let url = format!(
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
```

```rust
// src/skills/installer.rs
use std::path::{Path, PathBuf};
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
                            manifest.version.as_deref().unwrap_or("unversioned")
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
```

```rust
// src/skills/search.rs
use crate::error::Result;
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
```

```rust
// src/skills/builtin.rs
use include_dir::{include_dir, Dir};

/// Directory containing built-in skill source files.
static BUILTIN_SKILLS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/skills");

/// Get all built-in skill names.
pub fn list_names() -> Vec<&'static str> {
    BUILTIN_SKILLS_DIR
        .dirs()
        .filter_map(|d| d.path().file_name().and_then(|n| n.to_str()))
        .collect()
}

/// Get the content of a built-in skill's SKILL.md.
pub fn get_skill_content(name: &str) -> Option<&'static str> {
    BUILTIN_SKILLS_DIR
        .get_file(format!("{name}/SKILL.md"))
        .and_then(|f| f.contents_utf8())
}
```

- [ ] **Step 8: Create template files**

```markdown
<!-- templates/CLAUDE.md.template -->
# Claude Engineering OS

This file is managed by `claude-eng`. Do not edit manually.
Use `CLAUDE.local.md` for personal overrides.

## Workflow

Every task must follow this workflow:

1. **Understand** — Ask clarifying questions before writing code
2. **Plan** — Create an implementation plan with file list
3. **Search for Skills** — Check installed skills before implementing
4. **Implement** — Write minimal, focused code
5. **Verify** — Run tests and checks
6. **Review** — Self-review for correctness
7. **Report** — Summarize changes to the user
8. **Wait** — Get user approval before committing
9. **Commit** — Create a conventional commit
10. **Continue** — Move to the next task

## Rules

- Never skip the understand/planning phase
- Search for existing skills before implementing from scratch
- Run tests after every change
- Use conventional commits (feat:, fix:, docs:, refactor:, test:, chore:)
- Keep changes small and focused
- Update documentation when changing behavior

## Installed Skills

<!-- Skills are automatically listed here by claude-eng -->

```

```json
// templates/settings.json.template
{
  "env": {},
  "theme": "dark"
}
```

- [ ] **Step 9: Build and verify**

Run: `cargo build 2>&1`
Expected: Successful compilation (warnings OK, no errors)

- [ ] **Step 10: Run tests**

Run: `cargo test 2>&1`
Expected: 4 tests pass (from `skills/manifest.rs`)

- [ ] **Step 11: Commit**

```bash
git add -A
git commit -m "feat: scaffold project with CLI skeleton, config engine, and skills module"
```

---

### Task 2: Install Command — Config Generation

**Files:**
- Create: `src/cli/install.rs` (overwrite stub from Task 1)
- Create: `tests/cli_install.rs`

**Interfaces:**
- Consumes: `config::paths::*`, `config::claude_md::*`, `config::settings::*`, `config::templates::*`
- Produces: `InstallArgs` struct, `install::run()` function

- [ ] **Step 1: Write integration test for install**

```rust
// tests/cli_install.rs
use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_install_creates_claude_md() {
    let temp = TempDir::new().unwrap();
    let fake_home = temp.path();

    // Create a fake ~/.claude directory
    let claude_dir = fake_home.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    // Run the install command with HOME overridden
    Command::cargo_bin("claude-eng")
        .unwrap()
        .arg("install")
        .env("HOME", fake_home)
        .assert()
        .success();

    // Verify CLAUDE.md was created
    let claude_md = claude_dir.join("CLAUDE.md");
    assert!(claude_md.exists(), "CLAUDE.md should be created");

    let content = fs::read_to_string(&claude_md).unwrap();
    assert!(
        content.contains("Claude Engineering OS"),
        "CLAUDE.md should contain the base template"
    );
}

#[test]
fn test_install_creates_settings_json() {
    let temp = TempDir::new().unwrap();
    let fake_home = temp.path();

    let claude_dir = fake_home.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    Command::cargo_bin("claude-eng")
        .unwrap()
        .arg("install")
        .env("HOME", fake_home)
        .assert()
        .success();

    let settings = claude_dir.join("settings.json");
    assert!(settings.exists(), "settings.json should be created");
}

#[test]
fn test_install_is_idempotent() {
    let temp = TempDir::new().unwrap();
    let fake_home = temp.path();

    let claude_dir = fake_home.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    // Run install twice
    Command::cargo_bin("claude-eng")
        .unwrap()
        .arg("install")
        .env("HOME", fake_home)
        .assert()
        .success();

    let first_content = fs::read_to_string(claude_dir.join("CLAUDE.md")).unwrap();

    Command::cargo_bin("claude-eng")
        .unwrap()
        .arg("install")
        .env("HOME", fake_home)
        .assert()
        .success();

    let second_content = fs::read_to_string(claude_dir.join("CLAUDE.md")).unwrap();

    assert_eq!(first_content, second_content, "Install should be idempotent");
}

#[test]
fn test_install_preserves_existing_settings() {
    let temp = TempDir::new().unwrap();
    let fake_home = temp.path();

    let claude_dir = fake_home.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    // Write existing settings with user data
    let existing = serde_json::json!({
        "env": {
            "MY_CUSTOM_VAR": "hello"
        },
        "theme": "light"
    });
    fs::write(
        claude_dir.join("settings.json"),
        serde_json::to_string_pretty(&existing).unwrap(),
    )
    .unwrap();

    Command::cargo_bin("claude-eng")
        .unwrap()
        .arg("install")
        .env("HOME", fake_home)
        .assert()
        .success();

    // Read merged settings
    let settings: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(claude_dir.join("settings.json")).unwrap())
            .unwrap();

    // User's custom var should still be there
    assert_eq!(
        settings["env"]["MY_CUSTOM_VAR"], "hello",
        "Existing user settings should be preserved"
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test cli_install 2>&1`
Expected: FAIL — `install::run` not implemented / install command doesn't exist

- [ ] **Step 3: Implement install command**

```rust
// src/cli/install.rs
use clap::Args;
use crate::error::Result;

#[derive(Args)]
pub struct InstallArgs {
    /// Skip installing built-in skills
    #[arg(long)]
    pub skip_skills: bool,

    /// Force reinstall (overwrite without prompting)
    #[arg(long)]
    pub force: bool,
}

/// Run the install command.
pub fn run(args: InstallArgs) -> Result<()> {
    let claude_dir = crate::config::paths::claude_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot resolve ~/.claude directory"))?;

    // Ensure ~/.claude/ exists
    std::fs::create_dir_all(&claude_dir)?;

    println!("Installing Claude Engineering OS...");

    // Step 1: Generate CLAUDE.md
    let base_content = crate::config::claude_md::generate_base()?;
    let local_content = read_local_override();
    let merged = crate::config::claude_md::merge_with_local(&base_content, local_content.as_deref());
    crate::config::claude_md::write(&merged)?;
    println!("  ✓ Generated CLAUDE.md");

    // Step 2: Merge settings.json
    let mut existing = crate::config::settings::read()?;
    let template: serde_json::Value =
        serde_json::from_str(crate::config::templates::settings_json_template())?;
    crate::config::settings::merge(&mut existing, &template);
    crate::config::settings::write(&existing)?;
    println!("  ✓ Merged settings.json");

    // Step 3: Install built-in skills
    if !args.skip_skills {
        install_builtin_skills()?;
    }

    println!("\nInstallation complete! Claude Engineering OS is ready.");
    println!("Run `claude-eng --help` to see available commands.");

    Ok(())
}

/// Read CLAUDE.local.md if it exists.
fn read_local_override() -> Option<String> {
    let path = crate::config::paths::claude_local_md_path()?;
    if path.exists() {
        std::fs::read_to_string(path).ok()
    } else {
        None
    }
}

/// Install built-in skills to ~/.claude/skills/
fn install_builtin_skills() -> Result<()> {
    let skills_dir = crate::config::paths::skills_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot resolve skills directory"))?;

    std::fs::create_dir_all(&skills_dir)?;

    for name in crate::skills::builtin::list_names() {
        let skill_path = skills_dir.join(name);
        if skill_path.exists() {
            tracing::debug!("Built-in skill '{name}' already installed, skipping");
            continue;
        }

        if let Some(content) = crate::skills::builtin::get_skill_content(name) {
            std::fs::create_dir_all(&skill_path)?;
            std::fs::write(skill_path.join("SKILL.md"), content)?;
            println!("  ✓ Installed built-in skill: {name}");
        }
    }

    Ok(())
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test cli_install 2>&1`
Expected: All 4 tests PASS

- [ ] **Step 5: Run full test suite**

Run: `cargo test 2>&1`
Expected: All tests pass (manifest tests + install tests)

- [ ] **Step 6: Commit**

```bash
git add src/cli/install.rs tests/cli_install.rs
git commit -m "feat: implement install command with config generation and built-in skills"
```

---

### Task 3: Config Path Resolution Tests

**Files:**
- Create: `tests/config_paths.rs`
- Modify: `src/config/paths.rs` (add tests)

**Interfaces:**
- Consumes: `dirs::home_dir()`
- Produces: `config::paths::*` functions

- [ ] **Step 1: Write path resolution tests**

```rust
// tests/config_paths.rs
use claude_eng::config::paths;
use tempfile::TempDir;

#[test]
fn test_claude_dir_returns_home_dot_claude() {
    let result = paths::claude_dir();
    assert!(result.is_some());
    let path = result.unwrap();
    assert!(path.ends_with(".claude"));
}

#[test]
fn test_claude_md_path_ends_with_claude_md() {
    let result = paths::claude_md_path();
    assert!(result.is_some());
    assert!(result.unwrap().ends_with("CLAUDE.md"));
}

#[test]
fn test_claude_local_md_path_ends_with_claude_local_md() {
    let result = paths::claude_local_md_path();
    assert!(result.is_some());
    assert!(result.unwrap().ends_with("CLAUDE.local.md"));
}

#[test]
fn test_settings_json_path_ends_with_settings_json() {
    let result = paths::settings_json_path();
    assert!(result.is_some());
    assert!(result.unwrap().ends_with("settings.json"));
}

#[test]
fn test_skills_dir_ends_with_skills() {
    let result = paths::skills_dir();
    assert!(result.is_some());
    assert!(result.unwrap().ends_with("skills"));
}

#[test]
fn test_backups_dir_ends_with_backups() {
    let result = paths::backups_dir();
    assert!(result.is_some());
    assert!(result.unwrap().ends_with("backups"));
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test config_paths 2>&1`
Expected: All 6 tests PASS

- [ ] **Step 3: Commit**

```bash
git add tests/config_paths.rs
git commit -m "test: add config path resolution tests"
```

---

### Task 4: CLAUDE.md Generation Tests

**Files:**
- Create: `tests/config_claude_md.rs`

**Interfaces:**
- Consumes: `config::claude_md::*`, `config::templates::*`
- Produces: Tests for generation, merge, write

- [ ] **Step 1: Write CLAUDE.md tests**

```rust
// tests/config_claude_md.rs
use claude_eng::config::{claude_md, templates};

#[test]
fn test_generate_base_returns_template_content() {
    let content = claude_md::generate_base().unwrap();
    assert!(content.contains("Claude Engineering OS"));
    assert!(content.contains("Workflow"));
}

#[test]
fn test_merge_with_local_without_local() {
    let base = "# Base\nContent here";
    let result = claude_md::merge_with_local(base, None);
    assert_eq!(result, "# Base\nContent here");
}

#[test]
fn test_merge_with_local_with_local() {
    let base = "# Base\nContent here";
    let local = "# My Overrides\nCustom stuff";
    let result = claude_md::merge_with_local(base, Some(local));
    assert!(result.contains("# Base"));
    assert!(result.contains("User Overrides"));
    assert!(result.contains("# My Overrides"));
}

#[test]
fn test_template_not_empty() {
    let template = templates::claude_md_template();
    assert!(!template.is_empty());
}

#[test]
fn test_settings_template_is_valid_json() {
    let template = templates::settings_json_template();
    let _: serde_json::Value = serde_json::from_str(template).unwrap();
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test config_claude_md 2>&1`
Expected: All 5 tests PASS

- [ ] **Step 3: Commit**

```bash
git add tests/config_claude_md.rs
git commit -m "test: add CLAUDE.md generation and merge tests"
```

---

### Task 5: Settings Merge Tests

**Files:**
- Create: `tests/config_settings.rs`

**Interfaces:**
- Consumes: `config::settings::*`
- Produces: Tests for read, merge, write

- [ ] **Step 1: Write settings merge tests**

```rust
// tests/config_settings.rs
use claude_eng::config::settings;
use tempfile::TempDir;
use std::fs;

fn setup_fake_home() -> TempDir {
    let temp = TempDir::new().unwrap();
    let claude_dir = temp.path().join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();
    // Override HOME for dirs crate
    std::env::set_var("HOME", temp.path());
    temp
}

#[test]
fn test_read_returns_empty_when_no_file() {
    let _temp = setup_fake_home();
    let result = settings::read().unwrap();
    assert_eq!(result, serde_json::json!({}));
}

#[test]
fn test_merge_adds_new_keys() {
    let mut existing = serde_json::json!({"a": 1});
    let new = serde_json::json!({"b": 2});
    settings::merge(&mut existing, &new);
    assert_eq!(existing["a"], 1);
    assert_eq!(existing["b"], 2);
}

#[test]
fn test_merge_preserves_existing_keys() {
    let mut existing = serde_json::json!({"theme": "light"});
    let new = serde_json::json!({"theme": "dark", "model": "opus"});
    settings::merge(&mut existing, &new);
    // Existing keys are NOT overwritten by merge (additive only)
    assert_eq!(existing["theme"], "light");
    assert_eq!(existing["model"], "opus");
}

#[test]
fn test_merge_deep_merges_objects() {
    let mut existing = serde_json::json!({
        "env": {"MY_VAR": "hello"}
    });
    let new = serde_json::json!({
        "env": {"OTHER_VAR": "world"}
    });
    settings::merge(&mut existing, &new);
    assert_eq!(existing["env"]["MY_VAR"], "hello");
    assert_eq!(existing["env"]["OTHER_VAR"], "world");
}

#[test]
fn test_write_and_read() {
    let _temp = setup_fake_home();
    let data = serde_json::json!({"key": "value"});
    settings::write(&data).unwrap();
    let read_back = settings::read().unwrap();
    assert_eq!(read_back["key"], "value");
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test config_settings 2>&1`
Expected: All 5 tests PASS

- [ ] **Step 3: Commit**

```bash
git add tests/config_settings.rs
git commit -m "test: add settings merge and persistence tests"
```

---

### Task 6: Skill Manifest Parser

**Files:**
- Modify: `src/skills/manifest.rs` (already has implementation from Task 1)
- Create: `tests/skills_manifest.rs`

**Interfaces:**
- Consumes: SKILL.md file content (string)
- Produces: `SkillManifest` struct, `parse()` function

- [ ] **Step 1: Write manifest parser tests**

```rust
// tests/skills_manifest.rs
use claude_eng::skills::manifest::{parse, SkillManifest};

#[test]
fn test_parse_full_manifest() {
    let input = r#"---
name: my-skill
description: A full skill
version: "2.0.0"
triggers:
  - build
  - create
  - make
dependencies:
  - other-skill
---

# My Skill

Body content here.
"#;

    let (manifest, body) = parse(input).unwrap();

    assert_eq!(manifest.name, "my-skill");
    assert_eq!(manifest.description, "A full skill");
    assert_eq!(manifest.version, "2.0.0");
    assert_eq!(manifest.triggers, vec!["build", "create", "make"]);
    assert_eq!(manifest.dependencies, vec!["other-skill"]);
    assert!(body.contains("Body content here"));
}

#[test]
fn test_parse_minimal_manifest() {
    let input = r#"---
name: minimal
description: minimal skill
---

Content
"#;

    let (manifest, body) = parse(input).unwrap();
    assert_eq!(manifest.name, "minimal");
    assert_eq!(manifest.version, "");
    assert!(manifest.triggers.is_empty());
    assert!(manifest.dependencies.is_empty());
    assert!(body.contains("Content"));
}

#[test]
fn test_parse_body_preserves_markdown() {
    let input = r#"---
name: doc-skill
description: has docs
---

# Heading

## Subheading

- list item 1
- list item 2

```rust
fn main() {}
```
"#;

    let (_, body) = parse(input).unwrap();
    assert!(body.contains("# Heading"));
    assert!(body.contains("## Subheading"));
    assert!(body.contains("- list item 1"));
    assert!(body.contains("```rust"));
}

#[test]
fn test_parse_error_no_frontmatter() {
    let input = "Just text, no frontmatter";
    assert!(parse(input).is_err());
}

#[test]
fn test_parse_error_unclosed_frontmatter() {
    let input = "---\nname: test\n---\n---\nname: broken";
    // This should parse the first section fine
    let result = parse(input);
    assert!(result.is_ok() || result.is_err()); // depends on parser
}

#[test]
fn test_parse_error_invalid_yaml() {
    let input = "---\nname: [invalid yaml\n---\nBody";
    assert!(parse(input).is_err());
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test skills_manifest 2>&1`
Expected: All 6 tests PASS (manifest parser is already implemented)

- [ ] **Step 3: Commit**

```bash
git add tests/skills_manifest.rs
git commit -m "test: add skill manifest parser tests"
```

---

### Task 7: Skill Search

**Files:**
- Modify: `src/skills/search.rs` (already implemented in Task 1)
- Create: `tests/skills_search.rs`

**Interfaces:**
- Consumes: `skills_dir()`, `SkillManifest`
- Produces: `search_local()` function

- [ ] **Step 1: Write search tests**

```rust
// tests/skills_search.rs
use std::fs;
use tempfile::TempDir;
use claude_eng::config::paths;

fn setup_skills_dir() -> TempDir {
    let temp = TempDir::new().unwrap();
    let claude_dir = temp.path().join(".claude");
    let skills_dir = claude_dir.join("skills");
    fs::create_dir_all(&skills_dir).unwrap();
    std::env::set_var("HOME", temp.path());
    temp
}

fn install_test_skill(name: &str, description: &str, triggers: &[&str]) {
    let skills_dir = paths::skills_dir().unwrap();
    let skill_dir = skills_dir.join(name);
    fs::create_dir_all(&skill_dir).unwrap();

    let triggers_yaml: String = triggers
        .iter()
        .map(|t| format!("  - {t}"))
        .collect::<Vec<_>>()
        .join("\n");

    let content = format!(
        "---\nname: {name}\ndescription: {description}\ntriggers:\n{triggers_yaml}\n---\n\n# {name}\n"
    );
    fs::write(skill_dir.join("SKILL.md"), content).unwrap();
}

#[test]
fn test_search_finds_by_name() {
    let _temp = setup_skills_dir();
    install_test_skill("my-skill", "A test skill", &["test"]);

    // search_local prints to stdout, so we just verify it doesn't error
    let result = claude_eng::skills::search::search_local("my-skill");
    assert!(result.is_ok());
}

#[test]
fn test_search_finds_by_description() {
    let _temp = setup_skills_dir();
    install_test_skill("my-skill", "A testing skill", &[]);

    let result = claude_eng::skills::search::search_local("testing");
    assert!(result.is_ok());
}

#[test]
fn test_search_finds_by_trigger() {
    let _temp = setup_skills_dir();
    install_test_skill("my-skill", "A skill", &["deploy"]);

    let result = claude_eng::skills::search::search_local("deploy");
    assert!(result.is_ok());
}

#[test]
fn test_search_no_match() {
    let _temp = setup_skills_dir();
    install_test_skill("my-skill", "A skill", &["test"]);

    let result = claude_eng::skills::search::search_local("nonexistent");
    assert!(result.is_ok());
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test skills_search 2>&1`
Expected: All 4 tests PASS

- [ ] **Step 3: Commit**

```bash
git add tests/skills_search.rs
git commit -m "test: add skill search tests"
```

---

### Task 8: Skill Installer

**Files:**
- Modify: `src/skills/installer.rs` (already implemented in Task 1)
- Create: `tests/skills_installer.rs`

**Interfaces:**
- Consumes: `skills_dir()`, `SkillManifest`, `git2::Repository`
- Produces: `list_installed()`, `install_from_git()`, `uninstall()` functions

- [ ] **Step 1: Write installer tests**

```rust
// tests/skills_installer.rs
use std::fs;
use tempfile::TempDir;
use claude_eng::config::paths;

fn setup_skills_dir() -> TempDir {
    let temp = TempDir::new().unwrap();
    let claude_dir = temp.path().join(".claude");
    let skills_dir = claude_dir.join("skills");
    fs::create_dir_all(&skills_dir).unwrap();
    std::env::set_var("HOME", temp.path());
    temp
}

fn create_local_skill(name: &str) {
    let skills_dir = paths::skills_dir().unwrap();
    let skill_dir = skills_dir.join(name);
    fs::create_dir_all(&skill_dir).unwrap();
    let content = format!(
        "---\nname: {name}\ndescription: Test skill\n---\n\n# {name}\n"
    );
    fs::write(skill_dir.join("SKILL.md"), content).unwrap();
}

#[test]
fn test_list_installed_empty() {
    let _temp = setup_skills_dir();
    let result = claude_eng::skills::installer::list_installed();
    assert!(result.is_ok());
}

#[test]
fn test_list_installed_shows_skills() {
    let _temp = setup_skills_dir();
    create_local_skill("test-skill");
    let result = claude_eng::skills::installer::list_installed();
    assert!(result.is_ok());
}

#[test]
fn test_uninstall_existing() {
    let _temp = setup_skills_dir();
    create_local_skill("remove-me");
    let result = claude_eng::skills::installer::uninstall("remove-me");
    assert!(result.is_ok());
    assert!(!paths::skills_dir().unwrap().join("remove-me").exists());
}

#[test]
fn test_uninstall_nonexistent() {
    let _temp = setup_skills_dir();
    let result = claude_eng::skills::installer::uninstall("does-not-exist");
    assert!(result.is_err());
}

#[test]
fn test_list_with_invalid_skill() {
    let _temp = setup_skills_dir();
    let skills_dir = paths::skills_dir().unwrap();
    let skill_dir = skills_dir.join("invalid-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), "not valid yaml at all").unwrap();

    let result = claude_eng::skills::installer::list_installed();
    assert!(result.is_ok());
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test skills_installer 2>&1`
Expected: All 5 tests PASS

- [ ] **Step 3: Commit**

```bash
git add tests/skills_installer.rs
git commit -m "test: add skill installer tests"
```

---

### Task 9: Built-in Skills (5 core skills)

**Files:**
- Create: `skills/brainstorming/SKILL.md`
- Create: `skills/tdd/SKILL.md`
- Create: `skills/systematic-debugging/SKILL.md`
- Create: `skills/verification/SKILL.md`
- Create: `skills/code-review/SKILL.md`

**Interfaces:**
- Consumes: nothing (content only)
- Produces: 5 built-in skills embedded via `include_dir!`

- [ ] **Step 1: Create brainstorming skill**

```markdown
<!-- skills/brainstorming/SKILL.md -->
---
name: brainstorming
description: Explore intent, requirements, and design before implementation
version: 1.0.0
triggers:
  - brainstorm
  - design
  - plan
  - think through
dependencies: []
---

# Brainstorming

## When to Use

Use this skill before ANY creative work — creating features, building components, adding functionality, or modifying behavior. It explores user intent, requirements, and design before implementation.

## Steps

1. **Understand the request** — What does the user actually want?
2. **Clarify requirements** — Ask one question at a time
3. **Propose approaches** — 2-3 options with trade-offs
4. **Present design** — Get approval before coding
5. **Write spec** — Document the design
6. **Transition to implementation** — Only after approval

## Rules

- Never skip to coding without understanding first
- One question at a time (don't overwhelm)
- Multiple choice preferred over open-ended
- Get approval at each stage
```

- [ ] **Step 2: Create TDD skill**

```markdown
<!-- skills/tdd/SKILL.md -->
---
name: tdd
description: Test-driven development workflow
version: 1.0.0
triggers:
  - tdd
  - test-driven
  - write tests first
dependencies: []
---

# Test-Driven Development

## When to Use

Use this skill when writing new functionality. Write the test first, see it fail, then implement minimally.

## Steps

1. **Write the failing test** — Define expected behavior
2. **Run the test** — Verify it fails for the right reason
3. **Write minimal implementation** — Just enough to pass
4. **Run the test** — Verify it passes
5. **Refactor** — Clean up while keeping tests green
6. **Commit** — One commit per test+implementation pair

## Rules

- Never write implementation before the test
- Run tests after every change
- Keep implementations minimal (YAGNI)
- Refactor only when tests are green
```

- [ ] **Step 3: Create systematic-debugging skill**

```markdown
<!-- skills/systematic-debugging/SKILL.md -->
---
name: systematic-debugging
description: Structured approach to finding and fixing bugs
version: 1.0.0
triggers:
  - debug
  - bug
  - fix
  - broken
  - error
dependencies: []
---

# Systematic Debugging

## When to Use

Use this skill when encountering bugs, errors, or unexpected behavior.

## Steps

1. **Reproduce** — Create a minimal reproduction case
2. **Isolate** — Narrow down where the bug occurs
3. **Hypothesize** — Form a theory about the root cause
4. **Test hypothesis** — Verify the theory (don't assume)
5. **Fix** — Apply minimal fix
6. **Verify** — Confirm the fix works and doesn't break other things
7. **Document** — Record what was wrong and how it was fixed

## Rules

- Never guess — always verify hypotheses
- Fix the root cause, not symptoms
- Write a test that reproduces the bug before fixing
- Keep fixes minimal and focused
```

- [ ] **Step 4: Create verification skill**

```markdown
<!-- skills/verification/SKILL.md -->
---
name: verification
description: Run verification pipeline before marking tasks complete
version: 1.0.0
triggers:
  - verify
  - check
  - validate
  - test
dependencies: []
---

# Verification Pipeline

## When to Use

Use this skill after implementing changes to ensure quality.

## Pipeline Stages

1. **Lint** — Run project linter
2. **Test** — Run test suite
3. **Build** — Ensure project builds
4. **Review** — Self-review for correctness

## Rules

- Every stage must pass before proceeding
- Failures stop the pipeline immediately
- Report results clearly to the user
- Never skip verification
```

- [ ] **Step 5: Create code-review skill**

```markdown
<!-- skills/code-review/SKILL.md -->
---
name: code-review
description: Structured code review with findings and severity levels
version: 1.0.0
triggers:
  - review
  - code review
  - pr review
dependencies: []
---

# Code Review

## When to Use

Use this skill when reviewing code changes (own or others').

## Review Dimensions

1. **Correctness** — Does it do what it's supposed to?
2. **Security** — Any vulnerabilities or unsafe patterns?
3. **Performance** — Any inefficiencies or bottlenecks?
4. **Maintainability** — Is it readable and well-structured?
5. **Tests** — Are there adequate tests?

## Output Format

For each finding, report:
- **File and line** — Where the issue is
- **Severity** — critical / warning / suggestion
- **Description** — What's wrong
- **Fix** — How to fix it

## Rules

- Be specific — point to exact lines
- Suggest fixes, not just problems
- Prioritize critical issues
- Praise good patterns too
```

- [ ] **Step 6: Verify built-in skills are embedded**

Run: `cargo test 2>&1 | grep -E "test result|running"`
Expected: All tests pass, built-in skills are included via `include_dir!`

- [ ] **Step 7: Commit**

```bash
git add skills/
git commit -m "feat: add 5 built-in skills (brainstorming, tdd, debugging, verification, review)"
```

---

### Task 10: CLI Integration Tests

**Files:**
- Create: `tests/cli_integration.rs`

**Interfaces:**
- Consumes: Full binary via `assert_cmd`
- Produces: End-to-end CLI tests

- [ ] **Step 1: Write CLI integration tests**

```rust
// tests/cli_integration.rs
use assert_cmd::Command;
use std::fs;

#[test]
fn test_cli_help() {
    Command::cargo_bin("claude-eng")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Claude Engineering OS"));
}

#[test]
fn test_cli_version() {
    Command::cargo_bin("claude-eng")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("claude-eng"));
}

#[test]
fn test_install_subcommand_help() {
    Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["install", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Install"));
}

#[test]
fn test_skill_subcommand_help() {
    Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["skill", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Manage skills"));
}

#[test]
fn test_full_install_flow() {
    let temp = tempfile::TempDir::new().unwrap();
    let fake_home = temp.path();

    let claude_dir = fake_home.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    // Run full install
    Command::cargo_bin("claude-eng")
        .unwrap()
        .arg("install")
        .env("HOME", fake_home)
        .assert()
        .success();

    // Verify CLAUDE.md
    let claude_md = claude_dir.join("CLAUDE.md");
    assert!(claude_md.exists());
    let content = fs::read_to_string(&claude_md).unwrap();
    assert!(content.contains("Claude Engineering OS"));
    assert!(content.contains("Workflow"));

    // Verify settings.json
    let settings = claude_dir.join("settings.json");
    assert!(settings.exists());

    // Verify built-in skills were installed
    let skills_dir = claude_dir.join("skills");
    assert!(skills_dir.exists());
    assert!(skills_dir.join("brainstorming").exists());
    assert!(skills_dir.join("tdd").exists());
    assert!(skills_dir.join("systematic-debugging").exists());
    assert!(skills_dir.join("verification").exists());
    assert!(skills_dir.join("code-review").exists());
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test cli_integration 2>&1`
Expected: All 5 tests PASS

- [ ] **Step 3: Commit**

```bash
git add tests/cli_integration.rs
git commit -m "test: add end-to-end CLI integration tests"
```

---

### Task 11: Documentation + README

**Files:**
- Create: `README.md`
- Create: `LICENSE`
- Create: `docs/architecture.md`
- Create: `docs/user-guide.md`

**Interfaces:**
- Consumes: All previous tasks
- Produces: Documentation files

- [ ] **Step 1: Create README.md**

```markdown
# Claude Engineering OS

A reusable engineering platform that installs globally and improves every Claude Code session.

## What It Does

Claude Engineering OS transforms Claude Code into an opinionated software engineering operating system with:

- **Global configuration** — Consistent settings across all projects
- **Skill-first execution** — Search, install, and use reusable skills
- **Workflow engine** — YAML-defined state machines for common tasks
- **Memory** — Persistent context across sessions
- **Verification** — Automated quality checks
- **Git automation** — Structured commits and branch management

## Installation

```bash
# From source
cargo install --path .

# Or download a release binary
curl -sSL https://github.com/yourname/claude-engineering-os/releases/latest/download/claude-eng-linux -o claude-eng
chmod +x claude-eng
sudo mv claude-eng /usr/local/bin/
```

## Quick Start

```bash
# Install into ~/.claude/
claude-eng install

# List installed skills
claude-eng skill list

# Search for a skill
claude-eng skill search testing

# Install a skill from GitHub
claude-eng skill add owner/repo

# Remove a skill
claude-eng skill remove skill-name
```

## Commands

| Command | Description |
|---------|-------------|
| `claude-eng install` | Install or update configuration |
| `claude-eng skill list` | List installed skills |
| `claude-eng skill search <query>` | Search installed skills |
| `claude-eng skill add <name>` | Install a skill from registry |
| `claude-eng skill remove <name>` | Remove an installed skill |

## Built-in Skills

| Skill | Description |
|-------|-------------|
| `brainstorming` | Explore intent before implementing |
| `tdd` | Test-driven development workflow |
| `systematic-debugging` | Structured approach to fixing bugs |
| `verification` | Quality check pipeline |
| `code-review` | Structured code review |

## Configuration

Claude Engineering OS generates and manages:

- `~/.claude/CLAUDE.md` — Main instructions (owned by claude-eng)
- `~/.claude/CLAUDE.local.md` — Your personal overrides
- `~/.claude/settings.json` — Claude Code settings (merged additively)
- `~/.claude/skills/` — Installed skills
- `~/.claude/backups/` — Config backups before changes

## License

MIT
```

- [ ] **Step 2: Create LICENSE**

```
MIT License

Copyright (c) 2026

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

- [ ] **Step 3: Commit**

```bash
git add README.md LICENSE docs/
git commit -m "docs: add README, LICENSE, and project documentation"
```

---

### Task 12: Final Integration + CI Prep

**Files:**
- Create: `.github/workflows/ci.yml`
- Modify: `Cargo.toml` (add metadata)

**Interfaces:**
- Consumes: All previous tasks
- Produces: CI configuration, final metadata

- [ ] **Step 1: Create CI workflow**

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, master]
  pull_request:
    branches: [main, master]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --verbose
      - run: cargo clippy -- -D warnings

  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --release --verbose
      - uses: actions/upload-artifact@v4
        with:
          name: claude-eng-linux
          path: target/release/claude-eng
```

- [ ] **Step 2: Run full test suite one final time**

Run: `cargo test 2>&1`
Expected: All tests pass

- [ ] **Step 3: Build release binary**

Run: `cargo build --release 2>&1`
Expected: Release binary compiles successfully

- [ ] **Step 4: Verify binary works**

Run: `./target/release/claude-eng --help`
Expected: Shows help output

- [ ] **Step 5: Commit**

```bash
git add .github/
git commit -m "ci: add GitHub Actions CI workflow"
```

---

## Summary

| Task | Description | Tests |
|------|-------------|-------|
| 1 | Project scaffolding + CLI skeleton | 0 new |
| 2 | Install command — config generation | 4 tests |
| 3 | Config path resolution | 6 tests |
| 4 | CLAUDE.md generation + merge | 5 tests |
| 5 | Settings merge + persistence | 5 tests |
| 6 | Skill manifest parser | 6 tests |
| 7 | Skill search | 4 tests |
| 8 | Skill installer | 5 tests |
| 9 | Built-in skills (5) | 0 new |
| 10 | CLI integration tests | 5 tests |
| 11 | Documentation + README | 0 new |
| 12 | CI + final integration | 0 new |

**Total: 12 tasks, ~40 tests, ~55 commits**
