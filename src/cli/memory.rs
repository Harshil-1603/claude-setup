// src/cli/memory.rs
use clap::Subcommand;
use crate::error::Result;

/// Commands for managing persistent memory across sessions.
#[derive(Subcommand)]
pub enum MemoryAction {
    /// Store a memory entry
    Store {
        /// Memory kind: decision, progress, context, error
        #[arg(short, long)]
        kind: String,
        /// Comma-separated tags
        #[arg(short, long)]
        tags: Option<String>,
        /// Project path (for project-scoped memories)
        #[arg(short, long)]
        project: Option<String>,
        /// Memory content
        content: String,
    },
    /// Recall memories matching a query
    Recall {
        /// Search query
        query: String,
        /// Scope to project path
        #[arg(short, long)]
        project: Option<String>,
        /// Max results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// List all memories
    List {
        /// Filter by kind
        #[arg(short, long)]
        kind: Option<String>,
        /// Scope to project path
        #[arg(short, long)]
        project: Option<String>,
    },
    /// Delete a memory by ID
    Delete {
        /// Memory ID
        id: String,
    },
    /// Generate context markdown from recent memories
    Context {
        /// Scope to project path
        #[arg(short, long)]
        project: Option<String>,
        /// Max memories
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
}

/// Run the memory command.
pub fn run(action: MemoryAction) -> Result<()> {
    let store = crate::memory::store::Store::open_default()?;

    match action {
        MemoryAction::Store { kind, tags, project, content } => {
            let id = store.store(&kind, project.as_deref(), &content, tags.as_deref(), None)?;
            println!("Stored memory: {id}");
            println!("  Kind: {kind}");
            if let Some(p) = &project {
                println!("  Project: {p}");
            }
            if let Some(t) = &tags {
                println!("  Tags: {t}");
            }
        }
        MemoryAction::Recall { query, project, limit } => {
            let results = store.recall(&query, project.as_deref(), limit)?;
            if results.is_empty() {
                println!("No memories found matching '{query}'.");
            } else {
                println!("Found {} memories:\n", results.len());
                for mem in &results {
                    let proj = mem.project.as_deref().unwrap_or("global");
                    let tags = mem.tags.as_deref().unwrap_or("");
                    println!("[{}] {} ({})", mem.id, mem.kind, proj);
                    println!("  {}", mem.content);
                    if !tags.is_empty() {
                        println!("  tags: {tags}");
                    }
                    println!();
                }
            }
        }
        MemoryAction::List { kind, project } => {
            let memories = store.list(kind.as_deref(), project.as_deref())?;
            if memories.is_empty() {
                println!("No memories found.");
            } else {
                println!("{} memories:\n", memories.len());
                for mem in &memories {
                    let proj = mem.project.as_deref().unwrap_or("global");
                    println!("[{}] {} ({}) — {}", mem.id, mem.kind, proj, mem.content);
                }
            }
        }
        MemoryAction::Delete { id } => {
            let deleted = store.delete(&id)?;
            if deleted {
                println!("Deleted memory: {id}");
            } else {
                println!("Memory not found: {id}");
            }
        }
        MemoryAction::Context { project, limit } => {
            let ctx = store.context(project.as_deref(), limit)?;
            if ctx.is_empty() {
                println!("<!-- No memories found -->");
            } else {
                print!("{ctx}");
            }
        }
    }

    Ok(())
}
