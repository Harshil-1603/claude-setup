// src/cli/install.rs
use clap::Args;
use crate::error::Result;

/// Arguments for the install command.
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
///
/// Performs the full installation sequence:
/// 1. Generate `CLAUDE.md` from the embedded template
/// 2. Merge `CLAUDE.local.md` if present
/// 3. Write `CLAUDE.md` (with backup + atomic write)
/// 4. Read existing `settings.json`, merge template, write back
/// 5. Install built-in skills from the embedded directory
pub fn run(args: InstallArgs) -> Result<()> {
    let claude_dir = crate::config::paths::claude_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot resolve ~/.claude directory"))?;

    // Ensure ~/.claude/ exists
    std::fs::create_dir_all(&claude_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create ~/.claude directory: {e}"))?;

    println!("Installing Claude Engineering OS...");

    // Step 1: Generate CLAUDE.md
    let base_content = crate::config::claude_md::generate_base()?;
    let local_content = read_local_override();
    let merged = crate::config::claude_md::merge_with_local(
        &base_content,
        local_content.as_deref(),
    );
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

    // Step 4: Initialize memory database
    init_memory_db()?;
    println!("  ✓ Initialized memory database");

    println!("\nInstallation complete! Claude Engineering OS is ready.");
    println!("Run `claude-eng --help` to see available commands.");

    Ok(())
}

/// Read `CLAUDE.local.md` if it exists, returning its contents.
fn read_local_override() -> Option<String> {
    let path = crate::config::paths::claude_local_md_path()?;
    if path.exists() {
        std::fs::read_to_string(path).ok()
    } else {
        None
    }
}

/// Install all built-in skills to `~/.claude/skills/`.
///
/// Skills that already exist on disk are skipped so the operation is
/// idempotent.
fn install_builtin_skills() -> Result<()> {
    let skills_dir = crate::config::paths::skills_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot resolve skills directory"))?;

    std::fs::create_dir_all(&skills_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create skills directory: {e}"))?;

    for name in crate::skills::builtin::list_names() {
        let skill_path = skills_dir.join(name);
        if skill_path.exists() {
            tracing::debug!("Built-in skill '{name}' already installed, skipping");
            continue;
        }

        if let Some(content) = crate::skills::builtin::get_skill_content(name) {
            std::fs::create_dir_all(&skill_path)
                .map_err(|e| anyhow::anyhow!("Failed to create skill dir '{name}': {e}"))?;
            std::fs::write(skill_path.join("SKILL.md"), content)
                .map_err(|e| anyhow::anyhow!("Failed to write SKILL.md for '{name}': {e}"))?;
            println!("  ✓ Installed built-in skill: {name}");
        }
    }

    Ok(())
}

/// Initialize the memory database at `~/.claude/memory.db`.
fn init_memory_db() -> Result<()> {
    crate::memory::store::Store::open_default()?;
    Ok(())
}
