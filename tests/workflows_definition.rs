use claude_setup::workflows::definition::parse;

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

#[test]
fn test_defaults_applied() {
    let yaml = r#"
name: no-desc
states:
  - id: only
    name: "Only"
"#;
    let workflow = parse(yaml).unwrap();
    assert_eq!(workflow.description, "");
    assert_eq!(workflow.version, "1.0.0");
}
