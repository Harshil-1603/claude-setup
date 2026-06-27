use serde::{Deserialize, Serialize};

/// A workflow definition parsed from YAML.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workflow {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,
    pub states: Vec<State>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// A single state in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub actions: Vec<String>,
    #[serde(default)]
    pub next: Option<String>,
}

/// Parse a workflow definition from YAML content.
pub fn parse(content: &str) -> anyhow::Result<Workflow> {
    let workflow: Workflow = serde_yaml::from_str(content)?;

    // Validate: states must not be empty
    if workflow.states.is_empty() {
        anyhow::bail!("Workflow must have at least one state");
    }

    // Validate: first state should be the start
    let state_ids: Vec<&str> = workflow.states.iter().map(|s| s.id.as_str()).collect();
    for state in &workflow.states {
        if let Some(ref next) = state.next {
            if !state_ids.contains(&next.as_str()) {
                anyhow::bail!("State '{}' references unknown next state '{}'", state.id, next);
            }
        }
    }

    Ok(workflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_workflow() {
        let yaml = r#"
name: test-workflow
description: A test workflow
version: "1.0.0"
states:
  - id: start
    name: "Start"
    actions:
      - do_something
    next: end
  - id: end
    name: "End"
    actions:
      - finish
"#;
        let workflow = parse(yaml).unwrap();
        assert_eq!(workflow.name, "test-workflow");
        assert_eq!(workflow.states.len(), 2);
        assert_eq!(workflow.states[0].id, "start");
        assert_eq!(workflow.states[0].next, Some("end".to_string()));
    }

    #[test]
    fn test_parse_minimal_workflow() {
        let yaml = r#"
name: minimal
states:
  - id: only
    name: "Only State"
"#;
        let workflow = parse(yaml).unwrap();
        assert_eq!(workflow.states.len(), 1);
        assert!(workflow.states[0].actions.is_empty());
        assert!(workflow.states[0].next.is_none());
    }

    #[test]
    fn test_parse_empty_states_fails() {
        let yaml = r#"
name: empty
states: []
"#;
        assert!(parse(yaml).is_err());
    }

    #[test]
    fn test_parse_invalid_next_state_fails() {
        let yaml = r#"
name: bad
states:
  - id: a
    name: "A"
    next: nonexistent
"#;
        assert!(parse(yaml).is_err());
    }
}
