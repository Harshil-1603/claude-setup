use claude_eng::workflows::definition::parse;
use claude_eng::workflows::engine::{advance, create_instance, current_actions, is_complete, WorkflowInstance};

fn test_workflow() -> claude_eng::workflows::definition::Workflow {
    parse(
        r#"
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
"#,
    )
    .unwrap()
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
