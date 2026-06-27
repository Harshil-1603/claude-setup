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
