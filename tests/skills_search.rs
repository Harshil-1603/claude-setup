use std::fs;
use tempfile::TempDir;
use claude_eng::config::paths;

fn setup_skills_dir() -> TempDir {
    let temp = TempDir::new().unwrap();
    let claude_dir = temp.path().join(".claude");
    let skills_dir = claude_dir.join("skills");
    fs::create_dir_all(&skills_dir).unwrap();
    std::env::set_var("HOME", temp.path());
    temp
}

fn install_test_skill(name: &str, description: &str, triggers: &[&str]) {
    let skills_dir = paths::skills_dir().unwrap();
    let skill_dir = skills_dir.join(name);
    fs::create_dir_all(&skill_dir).unwrap();

    let triggers_yaml: String = triggers
        .iter()
        .map(|t| format!("  - {t}"))
        .collect::<Vec<_>>()
        .join("\n");

    let content = format!(
        "---\nname: {name}\ndescription: {description}\ntriggers:\n{triggers_yaml}\n---\n\n# {name}\n"
    );
    fs::write(skill_dir.join("SKILL.md"), content).unwrap();
}

#[test]
fn test_search_finds_by_name() {
    let _temp = setup_skills_dir();
    install_test_skill("my-skill", "A test skill", &["test"]);

    let result = claude_eng::skills::search::search_local("my-skill");
    assert!(result.is_ok());
}

#[test]
fn test_search_finds_by_description() {
    let _temp = setup_skills_dir();
    install_test_skill("my-skill", "A testing skill", &[]);

    let result = claude_eng::skills::search::search_local("testing");
    assert!(result.is_ok());
}

#[test]
fn test_search_finds_by_trigger() {
    let _temp = setup_skills_dir();
    install_test_skill("my-skill", "A skill", &["deploy"]);

    let result = claude_eng::skills::search::search_local("deploy");
    assert!(result.is_ok());
}

#[test]
fn test_search_no_match() {
    let _temp = setup_skills_dir();
    install_test_skill("my-skill", "A skill", &["test"]);

    let result = claude_eng::skills::search::search_local("nonexistent");
    assert!(result.is_ok());
}
