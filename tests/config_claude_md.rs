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
