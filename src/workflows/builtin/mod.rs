use include_dir::{include_dir, Dir};

use crate::workflows::definition::{parse, Workflow};

/// Directory containing built-in workflow YAML files.
static BUILTIN_WORKFLOWS_DIR: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/src/workflows/builtin");

/// Get all built-in workflow names.
pub fn list_names() -> Vec<&'static str> {
    BUILTIN_WORKFLOWS_DIR
        .files()
        .filter(|f| f.path().extension().and_then(|e| e.to_str()) == Some("yaml"))
        .filter_map(|f| f.path().file_stem().and_then(|n| n.to_str()))
        .collect()
}

/// Get a built-in workflow definition by name.
pub fn get_workflow(name: &str) -> anyhow::Result<Workflow> {
    let path = format!("{name}.yaml");
    let content = BUILTIN_WORKFLOWS_DIR
        .get_file(&path)
        .ok_or_else(|| anyhow::anyhow!("Built-in workflow '{name}' not found"))?
        .contents_utf8()
        .ok_or_else(|| anyhow::anyhow!("Built-in workflow '{name}' is not valid UTF-8"))?;

    parse(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_names_finds_all_builtins() {
        let mut names = list_names();
        names.sort();
        assert!(names.contains(&"bug-fix"), "Missing bug-fix");
        assert!(names.contains(&"feature-dev"), "Missing feature-dev");
        assert!(names.contains(&"refactor"), "Missing refactor");
        assert_eq!(names.len(), 3);
    }

    #[test]
    fn test_get_workflow_parses_correctly() {
        let wf = get_workflow("feature-dev").unwrap();
        assert_eq!(wf.name, "feature-development");
        assert!(wf.states.len() >= 4);
        assert_eq!(wf.states[0].id, "understand");
    }

    #[test]
    fn test_get_workflow_not_found() {
        let result = get_workflow("does-not-exist");
        assert!(result.is_err());
    }
}
