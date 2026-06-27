use assert_cmd::Command;
use tempfile::TempDir;

fn setup_fake_home() -> TempDir {
    let temp = TempDir::new().unwrap();
    let claude_dir = temp.path().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    temp
}

#[test]
fn test_memory_help() {
    Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["memory", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Manage persistent memory"));
}

#[test]
fn test_memory_store_and_recall() {
    let temp = setup_fake_home();
    let fake_home = temp.path();

    // Store a memory
    Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["memory", "store", "--kind", "decision", "Used JWT for auth"])
        .env("HOME", fake_home)
        .assert()
        .success();

    // Recall it
    let output = Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["memory", "recall", "JWT"])
        .env("HOME", fake_home)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("JWT"));
}

#[test]
fn test_memory_list() {
    let temp = setup_fake_home();
    let fake_home = temp.path();

    // Store a memory
    Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["memory", "store", "--kind", "progress", "Phase 3 done"])
        .env("HOME", fake_home)
        .assert()
        .success();

    // List
    let output = Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["memory", "list"])
        .env("HOME", fake_home)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Phase 3 done"));
}

#[test]
fn test_memory_delete() {
    let temp = setup_fake_home();
    let fake_home = temp.path();

    // Store a memory
    let output = Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["memory", "store", "--kind", "context", "temporary note"])
        .env("HOME", fake_home)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout.lines()
        .find(|l| l.starts_with("Stored memory:"))
        .and_then(|l| l.split_whitespace().last())
        .unwrap();

    // Delete it
    Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["memory", "delete", id])
        .env("HOME", fake_home)
        .assert()
        .success();
}

#[test]
fn test_memory_context() {
    let temp = setup_fake_home();
    let fake_home = temp.path();

    // Store a memory
    Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["memory", "store", "--kind", "decision", "Use Rust for CLI"])
        .env("HOME", fake_home)
        .assert()
        .success();

    // Get context
    let output = Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["memory", "context"])
        .env("HOME", fake_home)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Recent Memories"));
    assert!(stdout.contains("Use Rust for CLI"));
}
