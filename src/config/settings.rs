// src/config/settings.rs
use anyhow::Result;
use serde_json::Value;

/// Read existing settings.json or return empty object.
pub fn read() -> Result<Value> {
    let path = crate::config::paths::settings_json_path()
        .ok_or_else(|| anyhow::anyhow!("Cannot resolve settings.json path"))?;

    if !path.exists() {
        return Ok(serde_json::json!({}));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read settings.json: {e}"))?;

    let value: Value = serde_json::from_str(&content)?;
    Ok(value)
}

/// Merge new settings into existing settings (additive only).
pub fn merge(existing: &mut Value, new: &Value) {
    if let (Some(existing_obj), Some(new_obj)) = (existing.as_object_mut(), new.as_object()) {
        for (key, value) in new_obj {
            if existing_obj.contains_key(key) && value.is_object() {
                merge(existing_obj.get_mut(key).unwrap(), value);
            } else {
                existing_obj.insert(key.clone(), value.clone());
            }
        }
    }
}

/// Write settings.json (atomic write with backup).
pub fn write(settings: &Value) -> Result<()> {
    let path = crate::config::paths::settings_json_path()
        .ok_or_else(|| anyhow::anyhow!("Cannot resolve settings.json path"))?;

    // Ensure parent dir exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Backup existing file
    if path.exists() {
        let backup_dir = crate::config::paths::backups_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot resolve backups directory"))?;
        std::fs::create_dir_all(&backup_dir)?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let backup_path = backup_dir.join(format!("settings.json.{timestamp}.bak"));
        std::fs::copy(&path, &backup_path)?;
    }

    // Atomic write
    let content = serde_json::to_string_pretty(settings)?;
    let parent = path.parent().ok_or_else(|| anyhow::anyhow!("No parent dir"))?;
    let temp = tempfile::NamedTempFile::new_in(parent)?;
    std::io::Write::write_all(&mut temp.as_file(), content.as_bytes())?;
    temp.persist(&path)?;

    Ok(())
}
