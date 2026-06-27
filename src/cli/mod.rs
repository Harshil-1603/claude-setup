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
                    Ok(crate::skills::installer::list_installed()?)
                }
                SkillCommands::Search { query } => {
                    Ok(crate::skills::search::search_local(&query)?)
                }
                SkillCommands::Add { name } => {
                    Ok(crate::skills::installer::install_from_registry(&name)?)
                }
                SkillCommands::Remove { name } => {
                    Ok(crate::skills::installer::uninstall(&name)?)
                }
            }
        }
    }
}
