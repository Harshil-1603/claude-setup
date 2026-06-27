// src/main.rs
use clap::Parser;
use claude_eng::cli::Cli;
use claude_eng::cli::dispatch;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    dispatch(cli)?;

    Ok(())
}
