// src/skills/builtin.rs
use include_dir::{include_dir, Dir};

/// Directory containing built-in skill source files.
static BUILTIN_SKILLS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/skills");

/// Get all built-in skill names.
pub fn list_names() -> Vec<&'static str> {
    BUILTIN_SKILLS_DIR
        .dirs()
        .filter_map(|d| d.path().file_name().and_then(|n| n.to_str()))
        .collect()
}

/// Get the content of a built-in skill's SKILL.md.
pub fn get_skill_content(name: &str) -> Option<&'static str> {
    BUILTIN_SKILLS_DIR
        .get_file(format!("{name}/SKILL.md"))
        .and_then(|f| f.contents_utf8())
}
