use assert_cmd::Command;

#[test]
fn test_workflow_help() {
    Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["workflow", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Manage workflows"));
}

#[test]
fn test_workflow_list() {
    Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["workflow", "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Available workflows"));
}

#[test]
fn test_workflow_list_shows_builtins() {
    Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["workflow", "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("feature-dev"));
}

#[test]
fn test_workflow_status_nonexistent() {
    Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["workflow", "status", "nonexistent"])
        .assert()
        .success()
        .stdout(predicates::str::contains("No active workflow"));
}
