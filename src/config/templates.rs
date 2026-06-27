// src/config/templates.rs
use include_dir::{include_dir, Dir};

static TEMPLATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

/// Get the CLAUDE.md template content.
pub fn claude_md_template() -> &'static str {
    TEMPLATES_DIR
        .get_file("CLAUDE.md.template")
        .expect("CLAUDE.md.template must exist in templates/")
        .contents_utf8()
        .expect("CLAUDE.md.template must be valid UTF-8")
}

/// Get the settings.json template content.
pub fn settings_json_template() -> &'static str {
    TEMPLATES_DIR
        .get_file("settings.json.template")
        .expect("settings.json.template must exist in templates/")
        .contents_utf8()
        .expect("settings.json.template must be valid UTF-8")
}
