use claude_setup::verification::config::VerifyConfig;
use claude_setup::verification::pipeline;
use std::path::Path;

#[test]
fn test_default_config_loads() {
    let config = VerifyConfig::default_config();
    assert_eq!(config.stages, vec!["lint", "test", "build"]);
}

#[test]
fn test_pipeline_runs_stages() {
    let config = VerifyConfig {
        stages: vec!["lint".into()],
        stage_commands: {
            let mut map = std::collections::HashMap::new();
            map.insert("lint".into(), "echo hello".into());
            map
        },
    };

    let result = pipeline::run(&config, Path::new("/tmp")).unwrap();
    assert!(result.all_passed);
    assert_eq!(result.stages.len(), 1);
    assert!(result.stages[0].passed);
}

#[test]
fn test_stage_failure_stops_pipeline() {
    let config = VerifyConfig {
        stages: vec!["lint".into(), "test".into()],
        stage_commands: {
            let mut map = std::collections::HashMap::new();
            map.insert("lint".into(), "exit 1".into());
            map.insert("test".into(), "echo ok".into());
            map
        },
    };

    let result = pipeline::run(&config, Path::new("/tmp")).unwrap();
    assert!(!result.all_passed);
    assert!(!result.stages[0].passed);
}

#[test]
fn test_custom_command_overrides() {
    let config = VerifyConfig {
        stages: vec!["test".into()],
        stage_commands: {
            let mut map = std::collections::HashMap::new();
            map.insert("test".into(), "echo custom-test-output".into());
            map
        },
    };

    let result = pipeline::run(&config, Path::new("/tmp")).unwrap();
    assert!(result.stages[0].output.contains("custom-test-output"));
}

#[test]
fn test_format_results() {
    let result = pipeline::PipelineResult {
        stages: vec![
            pipeline::StageResult {
                name: "lint".into(),
                passed: true,
                output: String::new(),
                duration_ms: 100,
            },
            pipeline::StageResult {
                name: "test".into(),
                passed: false,
                output: "error: test failed".into(),
                duration_ms: 500,
            },
        ],
        all_passed: false,
        duration_ms: 600,
    };

    let formatted = pipeline::format_results(&result);
    assert!(formatted.contains("✓ lint"));
    assert!(formatted.contains("✗ test"));
    assert!(formatted.contains("Some stages failed"));
}

#[test]
fn test_format_json() {
    let result = pipeline::PipelineResult {
        stages: vec![pipeline::StageResult {
            name: "lint".into(),
            passed: true,
            output: String::new(),
            duration_ms: 100,
        }],
        all_passed: true,
        duration_ms: 100,
    };

    let json = pipeline::format_json(&result);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["all_passed"], true);
    assert_eq!(parsed["stages"][0]["name"], "lint");
}
