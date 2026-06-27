use claude_eng::git::commit;

#[test]
fn test_validate_conventional_valid() {
    assert!(commit::validate_conventional("feat: add login").is_ok());
    assert!(commit::validate_conventional("fix(auth): handle null token").is_ok());
    assert!(commit::validate_conventional("docs: update README").is_ok());
    assert!(commit::validate_conventional("chore: bump version").is_ok());
    assert!(commit::validate_conventional("refactor: extract helper").is_ok());
}

#[test]
fn test_validate_conventional_invalid() {
    assert!(commit::validate_conventional("added login").is_err());
    assert!(commit::validate_conventional("Feat: add login").is_err());
    assert!(commit::validate_conventional("random stuff").is_err());
}
