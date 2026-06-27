// src/hooks/definitions.rs

/// Generate hook configuration values for settings.json.
///
/// Returns a JSON object with hook definitions that can be merged into
/// the user's settings.
pub fn generate_hook_configs() -> serde_json::Value {
    serde_json::json!({
        "hooks": {
            "onSessionStart": [
                {
                    "type": "command",
                    "command": "claude-eng memory context 2>/dev/null || true"
                }
            ]
        }
    })
}
