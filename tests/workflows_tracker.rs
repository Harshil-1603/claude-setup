use claude_eng::workflows::tracker;
use claude_eng::workflows::engine::WorkflowInstance;
use tempfile::TempDir;

fn setup() -> TempDir {
    let temp = TempDir::new().unwrap();
    let claude_dir = temp.path().join(".claude").join("workflows");
    std::fs::create_dir_all(&claude_dir).unwrap();
    std::env::set_var("HOME", temp.path());
    temp
}

#[test]
fn test_progress_path() {
    let path = tracker::progress_path("test-workflow");
    assert!(path.to_string_lossy().contains("test-workflow.json"));
}

#[test]
fn test_save_and_load() {
    let _temp = setup();
    let instance = WorkflowInstance {
        workflow_name: "test".to_string(),
        current_state: "a".to_string(),
        completed_states: vec![],
        started_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
    };

    let path = tracker::progress_path("test");
    tracker::save(&instance, &path).unwrap();

    let loaded = tracker::load(&path).unwrap();
    assert_eq!(loaded.workflow_name, "test");
    assert_eq!(loaded.current_state, "a");
}

#[test]
fn test_list_progress_files() {
    let _temp = setup();
    let files = tracker::list_progress_files().unwrap();
    assert!(files.is_empty());
}
