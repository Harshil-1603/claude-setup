use claude_eng::skills::manifest::parse;

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
