// src/cli/git.rs
use clap::Subcommand;
use std::path::PathBuf;

/// Commands for git automation.
#[derive(Subcommand)]
pub enum GitAction {
    /// Create a conventional commit (stages all + Co-Authored-By trailer)
    Commit {
        /// Commit message in conventional format (e.g., "feat: add login")
        message: String,
    },
    /// Create a new feature branch
    Branch {
        /// Branch name
        name: String,
    },
    /// Generate a PR description from current changes
    Review,
    /// Show recent git log
    Log {
        /// Number of commits to show
        #[arg(short, long, default_value = "10")]
        count: usize,
    },
}

/// Run the git command.
pub fn run(action: GitAction) -> anyhow::Result<()> {
    let repo_path = find_repo_root()?;

    match action {
        GitAction::Commit { message } => {
            crate::git::commit::validate_conventional(&message)?;
            crate::git::commit::commit_all(&repo_path, &message)?;
            println!("Committed: {message}");
            println!("Co-Authored-By: Claude <noreply@anthropic.com>");
        }
        GitAction::Branch { name } => {
            crate::git::branch::create_branch(&repo_path, &name)?;
        }
        GitAction::Review => {
            let description = crate::git::review::generate_pr_description(&repo_path)?;
            println!("{description}");
        }
        GitAction::Log { count } => {
            show_log(&repo_path, count)?;
        }
    }

    Ok(())
}

/// Find the git repository root starting from the current directory.
fn find_repo_root() -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let repo = git2::Repository::discover(&cwd)
        .map_err(|e| anyhow::anyhow!("Not in a git repository: {e}"))?;
    let workdir = repo.workdir()
        .ok_or_else(|| anyhow::anyhow!("Git repository has no working directory"))?;
    Ok(workdir.to_path_buf())
}

/// Show a pretty git log.
fn show_log(repo_path: &PathBuf, count: usize) -> anyhow::Result<()> {
    let repo = git2::Repository::discover(repo_path)?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut shown = 0;
    for (i, oid_result) in revwalk.enumerate() {
        if i >= count {
            break;
        }
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

        let short_id = oid.to_string()[..7].to_string();
        let summary = commit.summary().unwrap_or("(no message)");

        // Check for Co-Authored-By trailer
        let has_coauthor = commit
            .body()
            .map(|b| b.contains("Co-Authored-By"))
            .unwrap_or(false);

        let coauthor_mark = if has_coauthor { " 🤖" } else { "" };

        println!("  {short_id} {summary}{coauthor_mark}");
        shown += 1;
    }

    if shown == 0 {
        println!("No commits found.");
    }

    Ok(())
}
