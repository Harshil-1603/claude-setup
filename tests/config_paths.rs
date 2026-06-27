// tests/config_paths.rs
use claude_setup::config::paths;

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
