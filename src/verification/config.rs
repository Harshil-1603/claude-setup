// src/verification/config.rs
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Verification configuration for a project.
#[derive(Debug, Clone)]
pub struct VerifyConfig {
    /// Ordered list of stages to run.
    pub stages: Vec<String>,
    /// Custom commands per stage (overrides defaults).
    pub stage_commands: HashMap<String, String>,
}

impl VerifyConfig {
    /// Load config from `claude-eng.yaml` in the given directory, or use defaults.
    pub fn load(project_dir: &Path) -> Result<Self> {
        let config_path = project_dir.join("claude-eng.yaml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
            return Self::from_yaml(&yaml);
        }
        Ok(Self::detect_or_default(project_dir))
    }

    /// Create config from YAML value.
    fn from_yaml(yaml: &serde_yaml::Value) -> Result<Self> {
        let mut stages = Vec::new();
        let mut stage_commands = HashMap::new();

        if let Some(verify) = yaml.get("verification") {
            if let Some(stage_list) = verify.get("stages") {
                if let Some(arr) = stage_list.as_sequence() {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            stages.push(s.to_string());
                        }
                    }
                }
            }
            // Check for custom commands per stage
            for stage_name in &["lint", "test", "build"] {
                if let Some(cmd) = verify.get(*stage_name).and_then(|s| s.get("command")) {
                    if let Some(cmd_str) = cmd.as_str() {
                        stage_commands.insert(stage_name.to_string(), cmd_str.to_string());
                    }
                }
            }
        }

        if stages.is_empty() {
            stages = vec!["lint".into(), "test".into(), "build".into()];
        }

        Ok(Self { stages, stage_commands })
    }

    /// Detect project type and return default config, or generic defaults.
    fn detect_or_default(project_dir: &Path) -> Self {
        let stages = vec!["lint".into(), "test".into(), "build".into()];
        let mut stage_commands = HashMap::new();

        // Auto-detect based on files present
        if project_dir.join("Cargo.toml").exists() {
            stage_commands.insert("lint".into(), "cargo fmt --check".into());
            stage_commands.insert("test".into(), "cargo test".into());
            stage_commands.insert("build".into(), "cargo build".into());
        } else if project_dir.join("package.json").exists() {
            stage_commands.insert("lint".into(), "npx eslint .".into());
            stage_commands.insert("test".into(), "npm test".into());
            stage_commands.insert("build".into(), "npm run build".into());
        } else if project_dir.join("pyproject.toml").exists()
            || project_dir.join("setup.py").exists()
            || project_dir.join("requirements.txt").exists()
        {
            stage_commands.insert("lint".into(), "ruff check .".into());
            stage_commands.insert("test".into(), "pytest".into());
            stage_commands.insert("build".into(), "python -m build".into());
        }

        Self { stages, stage_commands }
    }

    /// Default config with no custom commands.
    pub fn default_config() -> Self {
        Self {
            stages: vec!["lint".into(), "test".into(), "build".into()],
            stage_commands: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_loads() {
        let config = VerifyConfig::default_config();
        assert_eq!(config.stages, vec!["lint", "test", "build"]);
        assert!(config.stage_commands.is_empty());
    }

    #[test]
    fn test_from_yaml_uses_custom_stages() {
        let yaml: serde_yaml::Value = serde_yaml::from_str(
            r#"
verification:
  stages:
    - lint
    - test
  lint:
    command: "cargo clippy -- -D warnings"
"#,
        )
        .unwrap();
        let config = VerifyConfig::from_yaml(&yaml).unwrap();
        assert_eq!(config.stages, vec!["lint", "test"]);
        assert_eq!(
            config.stage_commands.get("lint").unwrap(),
            "cargo clippy -- -D warnings"
        );
    }

    #[test]
    fn test_from_yaml_empty_uses_defaults() {
        let yaml: serde_yaml::Value = serde_yaml::from_str("other_key: value").unwrap();
        let config = VerifyConfig::from_yaml(&yaml).unwrap();
        assert_eq!(config.stages, vec!["lint", "test", "build"]);
    }
}
