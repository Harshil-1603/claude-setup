use crate::workflows::engine::WorkflowInstance;
use crate::config::paths;
use std::path::PathBuf;

/// Get the path to a workflow's progress file.
pub fn progress_path(workflow_name: &str) -> PathBuf {
    let workflows_dir = paths::claude_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("workflows");
    workflows_dir.join(format!("{workflow_name}.json"))
}

/// Save workflow progress to disk (atomic write).
pub fn save(instance: &WorkflowInstance, path: &PathBuf) -> anyhow::Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(instance)?;

    // Atomic write
    let parent = path.parent().ok_or_else(|| anyhow::anyhow!("No parent dir"))?;
    let temp = tempfile::NamedTempFile::new_in(parent)?;
    std::io::Write::write_all(&mut temp.as_file(), content.as_bytes())?;
    temp.persist(path)?;

    Ok(())
}

/// Load workflow progress from disk.
pub fn load(path: &PathBuf) -> anyhow::Result<WorkflowInstance> {
    let content = std::fs::read_to_string(path)?;
    let instance: WorkflowInstance = serde_json::from_str(&content)?;
    Ok(instance)
}
