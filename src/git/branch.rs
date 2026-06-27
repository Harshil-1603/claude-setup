// src/git/branch.rs

use anyhow::Result;
use std::path::Path;

/// Create a new branch from the current HEAD.
pub fn create_branch(repo_path: &Path, name: &str) -> Result<()> {
    let repo = open_repo(repo_path)?;
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;
    repo.branch(name, &commit, false)?;
    println!("Created branch: {name}");
    Ok(())
}

/// List local branch names.
pub fn list_branches(repo_path: &Path) -> Result<Vec<String>> {
    let repo = open_repo(repo_path)?;
    let mut branches = Vec::new();

    for branch_result in repo.branches(Some(git2::BranchType::Local))? {
        let (branch, _branch_type) = branch_result?;
        if let Some(name) = branch.name()? {
            branches.push(name.to_string());
        }
    }

    Ok(branches)
}

/// Get the name of the current branch.
pub fn current_branch(repo_path: &Path) -> Result<String> {
    let repo = open_repo(repo_path)?;
    let head = repo.head()?;
    let name = head
        .shorthand()
        .ok_or_else(|| anyhow::anyhow!("HEAD is not a branch"))?;
    Ok(name.to_string())
}

fn open_repo(path: &Path) -> Result<git2::Repository> {
    git2::Repository::discover(path)
        .map_err(|e| anyhow::anyhow!("Failed to open git repository at {}: {e}", path.display()))
}
