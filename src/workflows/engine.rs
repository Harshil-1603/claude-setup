use crate::workflows::definition::{Workflow, State};
use crate::workflows::tracker;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Current state of a running workflow instance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowInstance {
    pub workflow_name: String,
    pub current_state: String,
    pub completed_states: Vec<String>,
    pub started_at: String,
    pub updated_at: String,
}

/// Get the current state definition for an instance.
pub fn current_state<'a>(instance: &WorkflowInstance, workflow: &'a Workflow) -> Option<&'a State> {
    workflow.states.iter().find(|s| s.id == instance.current_state)
}

/// Advance the workflow to the next state.
pub fn advance(instance: &mut WorkflowInstance, workflow: &Workflow) -> anyhow::Result<bool> {
    let current = current_state(instance, workflow)
        .ok_or_else(|| anyhow::anyhow!("Current state '{}' not found in workflow", instance.current_state))?;

    match &current.next {
        Some(next_id) => {
            // Verify next state exists
            workflow.states.iter().find(|s| s.id == *next_id)
                .ok_or_else(|| anyhow::anyhow!("Next state '{}' not found", next_id))?;

            instance.completed_states.push(instance.current_state.clone());
            instance.current_state = next_id.clone();
            instance.updated_at = current_timestamp();

            // Save progress
            let path = tracker::progress_path(&instance.workflow_name);
            tracker::save(instance, &path)?;

            Ok(true)
        }
        None => Ok(false), // No next state = workflow complete
    }
}

/// Create a new workflow instance from a workflow definition.
pub fn create_instance(workflow: &Workflow) -> anyhow::Result<WorkflowInstance> {
    let first_state = workflow.states.first()
        .ok_or_else(|| anyhow::anyhow!("Workflow has no states"))?;

    let instance = WorkflowInstance {
        workflow_name: workflow.name.clone(),
        current_state: first_state.id.clone(),
        completed_states: Vec::new(),
        started_at: current_timestamp(),
        updated_at: current_timestamp(),
    };

    // Save initial progress
    let path = tracker::progress_path(&workflow.name);
    tracker::save(&instance, &path)?;

    Ok(instance)
}

/// Load an existing workflow instance from disk.
pub fn load_instance(workflow_name: &str) -> anyhow::Result<Option<WorkflowInstance>> {
    let path = tracker::progress_path(workflow_name);
    if path.exists() {
        let instance = tracker::load(&path)?;
        Ok(Some(instance))
    } else {
        Ok(None)
    }
}

/// Check if a workflow instance is complete (no next state).
pub fn is_complete(instance: &WorkflowInstance, workflow: &Workflow) -> bool {
    current_state(instance, workflow)
        .and_then(|s| s.next.as_ref())
        .is_none()
}

/// Get the actions for the current state.
pub fn current_actions<'a>(instance: &WorkflowInstance, workflow: &'a Workflow) -> Vec<&'a str> {
    current_state(instance, workflow)
        .map(|s| s.actions.iter().map(|a| a.as_str()).collect())
        .unwrap_or_default()
}

fn current_timestamp() -> String {
    // Use a simple counter-based approach since SystemTime may not be available
    // In production, use chrono or similar
    "2026-01-01T00:00:00Z".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflows::definition::parse;

    fn test_workflow() -> Workflow {
        parse(r#"
name: test
states:
  - id: a
    name: "State A"
    actions: [action1]
    next: b
  - id: b
    name: "State B"
    actions: [action2]
    next: c
  - id: c
    name: "State C"
    actions: [action3]
"#).unwrap()
    }

    #[test]
    fn test_create_instance_starts_at_first_state() {
        let wf = test_workflow();
        let instance = create_instance(&wf).unwrap();
        assert_eq!(instance.current_state, "a");
        assert!(instance.completed_states.is_empty());
    }

    #[test]
    fn test_advance_moves_to_next_state() {
        let wf = test_workflow();
        let mut instance = create_instance(&wf).unwrap();

        let has_next = advance(&mut instance, &wf).unwrap();
        assert!(has_next);
        assert_eq!(instance.current_state, "b");
        assert_eq!(instance.completed_states, vec!["a"]);
    }

    #[test]
    fn test_advance_to_end_returns_false() {
        let wf = test_workflow();
        let mut instance = WorkflowInstance {
            workflow_name: "test".to_string(),
            current_state: "c".to_string(),
            completed_states: vec!["a".to_string(), "b".to_string()],
            started_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        };

        let has_next = advance(&mut instance, &wf).unwrap();
        assert!(!has_next);
    }

    #[test]
    fn test_current_actions() {
        let wf = test_workflow();
        let instance = create_instance(&wf).unwrap();
        let actions = current_actions(&instance, &wf);
        assert_eq!(actions, vec!["action1"]);
    }

    #[test]
    fn test_is_complete() {
        let wf = test_workflow();
        let instance = create_instance(&wf).unwrap();
        assert!(!is_complete(&instance, &wf));

        let complete_instance = WorkflowInstance {
            workflow_name: "test".to_string(),
            current_state: "c".to_string(),
            completed_states: vec!["a".to_string(), "b".to_string()],
            started_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        };
        assert!(is_complete(&complete_instance, &wf));
    }
}
