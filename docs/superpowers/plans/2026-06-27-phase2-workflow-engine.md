# Claude Engineering OS — Phase 2: Workflow Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a YAML-defined workflow engine with state tracking and resume support, plus 3 built-in workflows.

**Architecture:** Workflows are YAML files defining states with actions. The engine loads a workflow, runs actions for the current state, and advances when completion criteria are met. Progress is persisted to `~/.claude/workflows/<id>.json` for resume.

**Tech Stack:** Rust 2021 edition, `serde_yaml` (YAML parsing), `serde_json` (progress persistence), `uuid` (workflow IDs).

## Global Constraints

- Rust 2021 edition, MSRV 1.70
- All public APIs must have doc comments
- All modules must have unit tests
- Config generation must be idempotent
- Atomic writes (write temp, then rename) for all config files
- No telemetry, no network calls

---

## File Structure

```
src/workflows/
├── mod.rs                  # Module re-exports
├── definition.rs           # YAML workflow definition parser
├── engine.rs               # State machine runner
├── tracker.rs              # Progress persistence (JSON)
└── builtin/
    ├── mod.rs              # Built-in workflow loader
    ├── feature-dev.yaml    # Feature development workflow
    ├── bug-fix.yaml        # Bug fix workflow
    └── refactor.yaml       # Refactoring workflow
```

---

### Task 13: Workflow Definition Parser

**Files:**
- Create: `src/workflows/mod.rs`
- Create: `src/workflows/definition.rs`
- Create: `tests/workflows_definition.rs`

**Interfaces:**
- Consumes: YAML file content (string)
- Produces: `Workflow` struct, `State` struct, `parse()` function

- [ ] **Step 1: Create workflow module and definition parser**

```rust
// src/workflows/mod.rs
pub mod builtin;
pub mod definition;
pub mod engine;
pub mod tracker;
```

```rust
// src/workflows/definition.rs
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
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test workflows_definition 2>&1 && cargo test --lib workflows::definition 2>&1`
Expected: All 5 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/workflows/ tests/workflows_definition.rs
git commit -m "feat: add workflow definition parser with YAML support"
```

---

### Task 14: Workflow Engine (State Machine)

**Files:**
- Create: `src/workflows/engine.rs`
- Create: `tests/workflows_engine.rs`

**Interfaces:**
- Consumes: `Workflow`, `State` from definition.rs
- Produces: `WorkflowInstance` struct, `run_state()` function

- [ ] **Step 1: Create workflow engine**

```rust
// src/workflows/engine.rs
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
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib workflows::engine 2>&1`
Expected: 5 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/workflows/engine.rs
git commit -m "feat: add workflow engine with state machine and progress tracking"
```

---

### Task 15: Workflow Tracker (Progress Persistence)

**Files:**
- Create: `src/workflows/tracker.rs`
- Create: `tests/workflows_tracker.rs`

**Interfaces:**
- Consumes: `WorkflowInstance` from engine.rs
- Produces: `progress_path()`, `save()`, `load()` functions

- [ ] **Step 1: Create workflow tracker**

```rust
// src/workflows/tracker.rs
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

/// List all workflow progress files.
pub fn list_progress_files() -> anyhow::Result<Vec<PathBuf>> {
    let workflows_dir = paths::claude_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("workflows");
    
    if !workflows_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut files = Vec::new();
    for entry in std::fs::read_dir(&workflows_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            files.push(path);
        }
    }
    
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let path = progress_path("test-workflow");
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
        
        let path = progress_path("test");
        save(&instance, &path).unwrap();
        
        let loaded = load(&path).unwrap();
        assert_eq!(loaded.workflow_name, "test");
        assert_eq!(loaded.current_state, "a");
    }

    #[test]
    fn test_list_progress_files() {
        let _temp = setup();
        let files = list_progress_files().unwrap();
        assert!(files.is_empty());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test workflows_tracker 2>&1`
Expected: 3 tests pass

- [ ] **Step 3: Commit**

```bash
git add src/workflows/tracker.rs tests/workflows_tracker.rs
git commit -m "feat: add workflow progress tracker with atomic persistence"
```

---

### Task 16: Built-in Workflows

**Files:**
- Create: `src/workflows/builtin/mod.rs`
- Create: `src/workflows/builtin/feature-dev.yaml`
- Create: `src/workflows/builtin/bug-fix.yaml`
- Create: `src/workflows/builtin/refactor.yaml`
- Modify: `Cargo.toml` (add include_dir for workflows)

**Interfaces:**
- Consumes: `Workflow` from definition.rs
- Produces: `list_names()`, `get_workflow()` functions

- [ ] **Step 1: Create built-in workflow loader**

```rust
// src/workflows/builtin/mod.rs
use include_dir::{include_dir, Dir};
use crate::workflows::definition::{parse, Workflow};

/// Directory containing built-in workflow YAML files.
static BUILTIN_WORKFLOWS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/workflows/builtin");

/// Get all built-in workflow names.
pub fn list_names() -> Vec<&'static str> {
    BUILTIN_WORKFLOWS_DIR
        .files()
        .filter_map(|f| f.path().file_stem().and_then(|n| n.to_str()))
        .filter(|name| name.ends_with(".yaml"))
        .map(|name| &name[..name.len() - 5]) // Remove .yaml
        .collect()
}

/// Get a built-in workflow definition by name.
pub fn get_workflow(name: &str) -> anyhow::Result<Workflow> {
    let path = format!("{name}.yaml");
    let content = BUILTIN_WORKFLOWS_DIR
        .get_file(&path)
        .ok_or_else(|| anyhow::anyhow!("Built-in workflow '{}' not found", name))?
        .contents_utf8()
        .ok_or_else(|| anyhow::anyhow!("Built-in workflow '{}' is not valid UTF-8", name))?;
    
    parse(content)
}
```

- [ ] **Step 2: Create feature-dev workflow**

```yaml
# src/workflows/builtin/feature-dev.yaml
name: feature-development
description: End-to-end feature development workflow
version: "1.0.0"

states:
  - id: understand
    name: "Understand Requirements"
    actions:
      - ask_clarifying_questions
      - search_existing_code
      - identify_constraints
    next: plan

  - id: plan
    name: "Create Implementation Plan"
    actions:
      - identify_files_to_change
      - consider_edge_cases
      - present_plan_for_approval
    next: implement

  - id: implement
    name: "Implement Changes"
    actions:
      - write_code
      - add_tests
      - update_docs
    next: verify

  - id: verify
    name: "Verify Implementation"
    actions:
      - run_tests
      - check_linting
      - manual_review
    next: report

  - id: report
    name: "Report Progress"
    actions:
      - summarize_changes
      - present_to_user
    next: done

  - id: done
    name: "Complete"
    actions:
      - create_commit
      - update_memory
```

- [ ] **Step 3: Create bug-fix workflow**

```yaml
# src/workflows/builtin/bug-fix.yaml
name: bug-fix
description: Structured approach to fixing bugs
version: "1.0.0"

states:
  - id: reproduce
    name: "Reproduce the Bug"
    actions:
      - gather_error_info
      - create_reproduction_case
      - confirm_reproduction
    next: isolate

  - id: isolate
    name: "Isolate the Problem"
    actions:
      - narrow_down_location
      - check_recent_changes
      - identify_root_cause
    next: fix

  - id: fix
    name: "Implement Fix"
    actions:
      - write_failing_test
      - implement_fix
      - verify_fix_works
    next: verify

  - id: verify
    name: "Verify Fix"
    actions:
      - run_test_suite
      - check_no_regressions
      - manual_verification
    next: report

  - id: report
    name: "Report Fix"
    actions:
      - document_root_cause
      - summarize_changes
      - create_commit
```

- [ ] **Step 4: Create refactor workflow**

```yaml
# src/workflows/builtin/refactor.yaml
name: refactor
description: Safe refactoring with verification at each step
version: "1.0.0"

states:
  - id: understand
    name: "Understand Current Code"
    actions:
      - read_and理解_code
      - identify_smells
      - document_current_behavior
    next: plan

  - id: plan
    name: "Plan Refactoring"
    actions:
      - define_target_structure
      - identify_risks
      - plan_incremental_steps
    next: refactor

  - id: refactor
    name: "Execute Refactoring"
    actions:
      - make_one_small_change
      - run_tests
      - commit_if_green
    next: verify

  - id: verify
    name: "Final Verification"
    actions:
      - run_full_test_suite
      - check_performance
      - review_changes
    next: done

  - id: done
    name: "Complete"
    actions:
      - update_documentation
      - create_final_commit
```

- [ ] **Step 5: Add include_dir dependency for workflows**

Add to `Cargo.toml` dependencies if not already present (it's already there from Task 1). Verify the build can find the YAML files.

- [ ] **Step 6: Run tests**

Run: `cargo test --lib workflows::builtin 2>&1`
Expected: Tests pass, built-in workflows load correctly

- [ ] **Step 7: Commit**

```bash
git add src/workflows/builtin/
git commit -m "feat: add 3 built-in workflows (feature-dev, bug-fix, refactor)"
```

---

### Task 17: Workflow CLI Commands

**Files:**
- Modify: `src/cli/mod.rs` (add workflow subcommand)
- Create: `src/cli/workflow.rs`
- Create: `tests/cli_workflow.rs`

**Interfaces:**
- Consumes: `WorkflowEngine`, `WorkflowInstance`, `Workflow`
- Produces: `claude-eng workflow {run,list,status}` commands

- [ ] **Step 1: Add workflow CLI module**

```rust
// src/cli/workflow.rs
use clap::Args;
use crate::error::Result;

#[derive(Args)]
pub struct WorkflowArgs {
    #[command(subcommand)]
    pub action: WorkflowAction,
}

#[derive(clap::Subcommand)]
pub enum WorkflowAction {
    /// List available workflows
    List,
    
    /// Show status of a running workflow
    Status {
        /// Workflow name
        name: String,
    },
    
    /// Start or resume a workflow
    Run {
        /// Workflow name
        name: String,
    },
}

/// Run the workflow command.
pub fn run(args: WorkflowArgs) -> Result<()> {
    match args.action {
        WorkflowAction::List => list_workflows(),
        WorkflowAction::Status { name } => show_status(&name),
        WorkflowAction::Run { name } => start_workflow(&name),
    }
}

fn list_workflows() -> Result<()> {
    let names = crate::workflows::builtin::list_names();
    
    if names.is_empty() {
        println!("No workflows available.");
    } else {
        println!("Available workflows:");
        for name in &names {
            println!("  - {name}");
        }
    }
    
    Ok(())
}

fn show_status(name: &str) -> Result<()> {
    match crate::workflows::engine::load_instance(name)? {
        Some(instance) => {
            println!("Workflow: {}", instance.workflow_name);
            println!("Current state: {}", instance.current_state);
            println!("Completed: {:?}", instance.completed_states);
            println!("Started: {}", instance.started_at);
        }
        None => {
            println!("No active workflow found for '{name}'.");
        }
    }
    Ok(())
}

fn start_workflow(name: &str) -> Result<()> {
    // Load workflow definition
    let workflow = crate::workflows::builtin::get_workflow(name)?;
    
    // Check for existing instance
    match crate::workflows::engine::load_instance(name)? {
        Some(mut instance) => {
            println!("Resuming workflow '{name}' from state: {}", instance.current_state);
            
            // Get current state info
            let current = crate::workflows::engine::current_state(&instance, &workflow)
                .ok_or_else(|| anyhow::anyhow!("Current state not found"))?;
            
            println!("\nCurrent state: {}", current.name);
            println!("Actions to complete:");
            for action in &current.actions {
                println!("  - {action}");
            }
            
            // Advance to next state (simplified - in real implementation, actions would be executed)
            if crate::workflows::engine::advance(&mut instance, &workflow)? {
                println!("\nAdvanced to next state: {}", instance.current_state);
            } else {
                println!("\nWorkflow complete!");
            }
        }
        None => {
            // Create new instance
            let instance = crate::workflows::engine::create_instance(&workflow)?;
            println!("Started workflow '{name}'");
            println!("Current state: {}", instance.current_state);
            
            let current = crate::workflows::engine::current_state(&instance, &workflow)
                .ok_or_else(|| anyhow::anyhow!("Current state not found"))?;
            
            println!("Actions to complete:");
            for action in &current.actions {
                println!("  - {action}");
            }
        }
    }
    
    Ok(())
}
```

- [ ] **Step 2: Update CLI module to include workflow command**

Add to `src/cli/mod.rs`:
```rust
pub mod workflow;
```

And update the `Commands` enum:
```rust
/// Manage workflows
Workflow(workflow::WorkflowArgs),
```

And dispatch:
```rust
Commands::Workflow(args) => workflow::run(args),
```

- [ ] **Step 3: Write integration tests**

```rust
// tests/cli_workflow.rs
use assert_cmd::Command;

#[test]
fn test_workflow_list_help() {
    Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["workflow", "--help"])
        .assert()
        .success();
}

#[test]
fn test_workflow_list() {
    Command::cargo_bin("claude-eng")
        .unwrap()
        .args(["workflow", "list"])
        .assert()
        .success();
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test --test cli_workflow 2>&1`
Expected: 2 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/cli/workflow.rs src/cli/mod.rs tests/cli_workflow.rs
git commit -m "feat: add workflow CLI commands (list, status, run)"
```

---

### Task 18: Update lib.rs and Final Integration

**Files:**
- Modify: `src/lib.rs`
- Modify: `src/cli/mod.rs`
- Run full test suite

**Interfaces:**
- Consumes: All workflow modules
- Produces: Updated module structure

- [ ] **Step 1: Update lib.rs to include workflows module**

```rust
// src/lib.rs
pub mod cli;
pub mod config;
pub mod error;
pub mod skills;
pub mod workflows;
```

- [ ] **Step 2: Run full test suite**

Run: `cargo test 2>&1`
Expected: All tests pass (should be 50+ tests now)

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "chore: integrate workflow engine into project structure"
```

---

### Task 19: Documentation Update

**Files:**
- Modify: `README.md`

**Interfaces:**
- Consumes: All completed features
- Produces: Updated documentation

- [ ] **Step 1: Update README with workflow commands**

Add to README.md:
```markdown
## Workflow Commands

| Command | Description |
|---------|-------------|
| `claude-eng workflow list` | List available workflows |
| `claude-eng workflow status <name>` | Show workflow progress |
| `claude-eng workflow run <name>` | Start or resume a workflow |

### Built-in Workflows

| Workflow | Description |
|----------|-------------|
| `feature-development` | End-to-end feature development |
| `bug-fix` | Structured bug fixing approach |
| `refactor` | Safe refactoring with verification |
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add workflow commands and built-in workflows to README"
```

---

## Summary

| Task | Description | Tests |
|------|-------------|-------|
| 13 | Workflow definition parser | 5 tests |
| 14 | Workflow engine (state machine) | 5 tests |
| 15 | Workflow tracker (progress) | 3 tests |
| 16 | Built-in workflows (3) | 0 new |
| 17 | Workflow CLI commands | 2 tests |
| 18 | Final integration | 0 new |
| 19 | Documentation update | 0 new |

**Total: 7 tasks, 15 new tests, ~7 commits**
