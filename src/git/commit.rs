// src/git/commit.rs

use anyhow::Result;
use std::path::Path;

/// Valid conventional commit prefixes.
const VALID_PREFIXES: &[&str] = &[
    "feat", "fix", "docs", "refactor", "test", "chore", "ci", "perf", "build", "style", "revert",
];

/// Validate that a message follows conventional commit format.
///
/// Format: `type(scope?): description`
pub fn validate_conventional(message: &str) -> Result<()> {
    let first_line = message.lines().next().unwrap_or("");

    let prefix = first_line
        .split('(')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .trim();

    if !VALID_PREFIXES.contains(&prefix) {
        anyhow::bail!(
            "Invalid conventional commit prefix '{prefix}'. Expected one of: {}",
            VALID_PREFIXES.join(", ")
        );
    }

    // Must have a colon and space after prefix
    let after_colon = first_line
        .strip_prefix(prefix)
        .unwrap_or(first_line);

    // Allow type: or type(scope):
    let has_colon = after_colon.starts_with(':') || after_colon.contains(':');
    if !has_colon {
        anyhow::bail!("Conventional commit must have ': description' after the prefix");
    }

    Ok(())
}

/// Stage all changes and create a commit with the Co-Authored-By trailer.
pub fn commit_all(repo_path: &Path, message: &str) -> Result<()> {
    let repo = open_repo(repo_path)?;

    // Stage all changes
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    create_commit_inner(&repo, message)
}

/// Create a commit from the current staged changes.
pub fn create_commit(repo_path: &Path, message: &str) -> Result<()> {
    let repo = open_repo(repo_path)?;
    create_commit_inner(&repo, message)
}

fn create_commit_inner(repo: &git2::Repository, message: &str) -> Result<()> {
    let sig = repo.signature()?;

    // Get the index and write tree
    let mut index = repo.index()?;
    index.write()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    // Build commit message with Co-Authored-By trailer
    let full_message = format!(
        "{message}\n\nCo-Authored-By: Claude <noreply@anthropic.com>"
    );

    // Find HEAD commit (if any)
    let parent_commit = match repo.head() {
        Ok(head) => {
            let oid = head.target().ok_or_else(|| anyhow::anyhow!("HEAD has no target"))?;
            Some(repo.find_commit(oid)?)
        }
        Err(_) => None,
    };

    let parents: Vec<&git2::Commit> = match &parent_commit {
        Some(c) => vec![c],
        None => vec![],
    };

    repo.commit(Some("HEAD"), &sig, &sig, &full_message, &tree, &parents)?;

    Ok(())
}

fn open_repo(path: &Path) -> Result<git2::Repository> {
    git2::Repository::discover(path)
        .map_err(|e| anyhow::anyhow!("Failed to open git repository at {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_conventional_valid() {
        assert!(validate_conventional("feat: add login").is_ok());
        assert!(validate_conventional("fix(auth): handle null token").is_ok());
        assert!(validate_conventional("docs: update README").is_ok());
        assert!(validate_conventional("chore: bump version").is_ok());
        assert!(validate_conventional("refactor: extract helper").is_ok());
    }

    #[test]
    fn test_validate_conventional_invalid() {
        assert!(validate_conventional("added login").is_err());
        assert!(validate_conventional("Feat: add login").is_err());
        assert!(validate_conventional("random stuff").is_err());
    }
}
