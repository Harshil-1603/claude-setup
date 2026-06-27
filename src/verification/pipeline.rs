// src/verification/pipeline.rs
use anyhow::Result;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use super::config::VerifyConfig;

/// Result of running a single verification stage.
#[derive(Debug, Clone)]
pub struct StageResult {
    pub name: String,
    pub passed: bool,
    pub output: String,
    pub duration_ms: u64,
}

/// Result of running the full verification pipeline.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub stages: Vec<StageResult>,
    pub all_passed: bool,
    pub duration_ms: u64,
}

/// Run the full verification pipeline.
pub fn run(config: &VerifyConfig, project_dir: &Path) -> Result<PipelineResult> {
    let start = Instant::now();
    let mut stages = Vec::new();
    let mut all_passed = true;

    for stage_name in &config.stages {
        let command = config
            .stage_commands
            .get(stage_name)
            .map(|s| s.as_str())
            .unwrap_or("echo 'no command configured'");

        let result = run_stage(stage_name, command, project_dir)?;
        if !result.passed {
            all_passed = false;
        }
        stages.push(result);
    }

    Ok(PipelineResult {
        stages,
        all_passed,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

/// Run a single verification stage.
pub fn run_stage(name: &str, command: &str, project_dir: &Path) -> Result<StageResult> {
    let start = Instant::now();

    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(project_dir)
        .output()?;

    let passed = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let mut full_output = stdout;
    if !stderr.is_empty() {
        if !full_output.is_empty() {
            full_output.push('\n');
        }
        full_output.push_str(&stderr);
    }

    Ok(StageResult {
        name: name.to_string(),
        passed,
        output: full_output,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

/// Format pipeline results for human-readable output.
pub fn format_results(result: &PipelineResult) -> String {
    let mut output = String::new();
    for stage in &result.stages {
        let icon = if stage.passed { "✓" } else { "✗" };
        output.push_str(&format!(
            "  {} {} ({}ms)\n",
            icon, stage.name, stage.duration_ms
        ));
        if !stage.passed && !stage.output.is_empty() {
            for line in stage.output.lines().take(10) {
                output.push_str(&format!("    {line}\n"));
            }
            if stage.output.lines().count() > 10 {
                output.push_str("    ... (truncated)\n");
            }
        }
    }
    let summary = if result.all_passed {
        "All stages passed"
    } else {
        "Some stages failed"
    };
    output.push_str(&format!(
        "\n{} ({}ms total)\n",
        summary, result.duration_ms
    ));
    output
}

/// Format pipeline results as JSON.
pub fn format_json(result: &PipelineResult) -> String {
    let mut json = serde_json::Map::new();
    json.insert(
        "all_passed".into(),
        serde_json::Value::Bool(result.all_passed),
    );
    json.insert(
        "duration_ms".into(),
        serde_json::Value::Number(result.duration_ms.into()),
    );

    let stages: Vec<serde_json::Value> = result
        .stages
        .iter()
        .map(|s| {
            let mut obj = serde_json::Map::new();
            obj.insert("name".into(), serde_json::Value::String(s.name.clone()));
            obj.insert(
                "passed".into(),
                serde_json::Value::Bool(s.passed),
            );
            obj.insert(
                "duration_ms".into(),
                serde_json::Value::Number(s.duration_ms.into()),
            );
            if !s.output.is_empty() {
                obj.insert(
                    "output".into(),
                    serde_json::Value::String(s.output.clone()),
                );
            }
            serde_json::Value::Object(obj)
        })
        .collect();

    json.insert("stages".into(), serde_json::Value::Array(stages));
    serde_json::to_string_pretty(&serde_json::Value::Object(json)).unwrap_or_default()
}
