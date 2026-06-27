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
