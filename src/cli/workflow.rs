use clap::Subcommand;
use crate::error::Result;

/// Subcommands for managing workflows.
#[derive(Subcommand)]
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
pub fn run(action: WorkflowAction) -> Result<()> {
    match action {
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
    let workflow = crate::workflows::builtin::get_workflow(name)?;

    match crate::workflows::engine::load_instance(name)? {
        Some(mut instance) => {
            println!("Resuming workflow '{name}' from state: {}", instance.current_state);

            let current = crate::workflows::engine::current_state(&instance, &workflow)
                .ok_or_else(|| anyhow::anyhow!("Current state not found"))?;

            println!("\nCurrent state: {}", current.name);
            println!("Actions to complete:");
            for action in &current.actions {
                println!("  - {action}");
            }

            if crate::workflows::engine::advance(&mut instance, &workflow)? {
                println!("\nAdvanced to next state: {}", instance.current_state);
            } else {
                println!("\nWorkflow complete!");
            }
        }
        None => {
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
