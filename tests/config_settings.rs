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
