// src/git/review.rs

use anyhow::Result;
use std::path::Path;

/// Generate a PR description from the diff between HEAD and a base branch.
pub fn generate_pr_description(repo_path: &Path) -> Result<String> {
    let repo = open_repo(repo_path)?;

    // Find the base branch (main or master)
    let base_name = find_base_branch(&repo)?;

    diff_summary(repo_path, &base_name)
}

/// Get a diff summary between HEAD and the given base reference.
pub fn diff_summary(repo_path: &Path, base: &str) -> Result<String> {
    let repo = open_repo(repo_path)?;

    let head_oid = repo
        .head()?
        .target()
        .ok_or_else(|| anyhow::anyhow!("HEAD has no target"))?;

    let base_ref = repo.find_branch(base, git2::BranchType::Local)?;
    let base_oid = base_ref.get().target().ok_or_else(|| {
        anyhow::anyhow!("Base branch '{base}' has no target")
    })?;

    let head_commit = repo.find_commit(head_oid)?;
    let base_commit = repo.find_commit(base_oid)?;

    let head_tree = head_commit.tree()?;
    let base_tree = base_commit.tree()?;

    let diff = repo.diff_tree_to_tree(Some(&base_tree), Some(&head_tree), None)?;

    let stats = diff.stats()?;
    let files_changed = stats.files_changed() as u32;
    let additions = stats.insertions() as u32;
    let deletions = stats.deletions() as u32;

    let mut changed_files = Vec::new();
    for delta in diff.deltas() {
        if let Some(name) = delta.new_file().path().or_else(|| delta.old_file().path()) {
            changed_files.push(name.display().to_string());
        }
    }

    let mut description = format!(
        "## Summary\n\nChanges from `{base}` to `HEAD`.\n\n\
         **Files changed:** {files_changed} | **+{additions}** / **-{deletions}**\n"
    );

    if !changed_files.is_empty() {
        description.push_str("\n## Files Changed\n\n");
        for file in &changed_files {
            description.push_str(&format!("- `{file}`\n"));
        }
    }

    // Try to infer a conventional commit type from changed files
    if let Some(suggested_type) = infer_commit_type(&changed_files) {
        description.push_str(&format!(
            "\n## Suggested Commit\n\n`{suggested_type}`\n"
        ));
    }

    Ok(description)
}

fn find_base_branch(repo: &git2::Repository) -> Result<String> {
    for name in &["main", "master", "develop"] {
        if repo.find_branch(name, git2::BranchType::Local).is_ok() {
            return Ok(name.to_string());
        }
    }
    Ok("main".to_string())
}

fn infer_commit_type(files: &[String]) -> Option<String> {
    let has_docs = files.iter().any(|f| f.contains("doc") || f.contains("README"));
    let has_test = files.iter().any(|f| f.contains("test") || f.contains("spec"));
    let has_config = files.iter().any(|f| {
        f.contains("Cargo.toml") || f.contains("package.json") || f.contains("pyproject.toml")
    });

    if has_docs {
        Some("docs: update documentation".into())
    } else if has_test {
        Some("test: add/update tests".into())
    } else if has_config {
        Some("chore: update dependencies/config".into())
    } else {
        Some("feat: new feature".into())
    }
}

fn open_repo(path: &Path) -> Result<git2::Repository> {
    git2::Repository::discover(path)
        .map_err(|e| anyhow::anyhow!("Failed to open git repository at {}: {e}", path.display()))
}
