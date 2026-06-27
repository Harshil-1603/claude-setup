use assert_cmd::Command;

#[test]
fn test_verify_help() {
    Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["verify", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("verification pipeline"));
}
