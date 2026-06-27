// src/cli/verify.rs
use clap::Args;
use std::path::PathBuf;
use crate::error::Result;

/// Arguments for the verify command.
#[derive(Args)]
pub struct VerifyArgs {
    /// Comma-separated list of stages to run (default: all)
    #[arg(short, long)]
    pub stages: Option<String>,
    /// Project directory (default: current directory)
    #[arg(short, long)]
    pub project: Option<PathBuf>,
    /// Output results as JSON
    #[arg(long)]
    pub json: bool,
}

/// Run the verification command.
pub fn run(args: VerifyArgs) -> Result<()> {
    let project_dir = args.project.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let mut config = crate::verification::config::VerifyConfig::load(&project_dir)?;

    // Filter stages if specified
    if let Some(stages_str) = &args.stages {
        let requested: Vec<String> = stages_str.split(',').map(|s| s.trim().to_string()).collect();
        config.stages.retain(|s| requested.contains(s));
    }

    if config.stages.is_empty() {
        println!("No verification stages to run.");
        return Ok(());
    }

    println!(
        "Running verification stages: {}\n",
        config.stages.join(", ")
    );

    let result = crate::verification::pipeline::run(&config, &project_dir)?;

    if args.json {
        let json = crate::verification::pipeline::format_json(&result);
        println!("{json}");
    } else {
        let output = crate::verification::pipeline::format_results(&result);
        print!("{output}");
    }

    if !result.all_passed {
        std::process::exit(1);
    }

    Ok(())
}
