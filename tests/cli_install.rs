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
