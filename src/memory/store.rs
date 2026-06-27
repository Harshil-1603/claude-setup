// src/memory/store.rs

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

use crate::config::paths;

/// A single memory entry.
#[derive(Debug, Clone)]
pub struct Memory {
    pub id: String,
    pub kind: String,
    pub project: Option<String>,
    pub content: String,
    pub tags: Option<String>,
    pub created_at: String,
    pub accessed_at: String,
    pub expires_at: Option<String>,
}

/// SQLite-backed memory store with FTS5 full-text search.
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Open (or create) the memory database at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path).context("Failed to open SQLite database")?;
        Self::initialize(&conn)?;
        Ok(Self { conn })
    }

    /// Open the default memory database at `~/.claude/memory.db`.
    pub fn open_default() -> Result<Self> {
        let path = paths::claude_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot resolve ~/.claude directory"))?
            .join("memory.db");
        Self::open(&path)
    }

    /// Create tables if they do not exist.
    fn initialize(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memories (
                id          TEXT PRIMARY KEY,
                kind        TEXT NOT NULL CHECK(kind IN ('decision','progress','context','error')),
                project     TEXT,
                content     TEXT NOT NULL,
                tags        TEXT,
                created_at  TEXT DEFAULT (datetime('now')),
                accessed_at TEXT DEFAULT (datetime('now')),
                expires_at  TEXT
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
                content, tags, content=memories, content_rowid=rowid
            );",
        )?;

        // Triggers to keep FTS index in sync
        conn.execute_batch(
            "CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
                INSERT INTO memories_fts(rowid, content, tags) VALUES (new.rowid, new.content, new.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS memories_ad AFTER DELETE ON memories BEGIN
                INSERT INTO memories_fts(memories_fts, rowid, content, tags) VALUES('delete', old.rowid, old.content, old.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS memories_au AFTER UPDATE ON memories BEGIN
                INSERT INTO memories_fts(memories_fts, rowid, content, tags) VALUES('delete', old.rowid, old.content, old.tags);
                INSERT INTO memories_fts(rowid, content, tags) VALUES (new.rowid, new.content, new.tags);
            END;",
        )?;

        Ok(())
    }

    /// Store a new memory entry. Returns the generated ID.
    pub fn store(
        &self,
        kind: &str,
        project: Option<&str>,
        content: &str,
        tags: Option<&str>,
        expires_at: Option<&str>,
    ) -> Result<String> {
        let id = generate_id();
        self.conn.execute(
            "INSERT INTO memories (id, kind, project, content, tags, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, kind, project, content, tags, expires_at],
        )?;
        Ok(id)
    }

    /// Recall memories matching a full-text query, optionally scoped by project.
    pub fn recall(&self, query: &str, project: Option<&str>, limit: usize) -> Result<Vec<Memory>> {
        let mut memories = Vec::new();

        match project {
            Some(proj) => {
                let mut stmt = self.conn.prepare(
                    "SELECT m.id, m.kind, m.project, m.content, m.tags, m.created_at, m.accessed_at, m.expires_at
                     FROM memories m
                     JOIN memories_fts fts ON m.rowid = fts.rowid
                     WHERE memories_fts MATCH ?1 AND m.project = ?2
                     ORDER BY rank
                     LIMIT ?3",
                )?;

                let rows = stmt.query_map(params![query, proj, limit as i64], |row| {
                    Ok(Memory {
                        id: row.get(0)?,
                        kind: row.get(1)?,
                        project: row.get(2)?,
                        content: row.get(3)?,
                        tags: row.get(4)?,
                        created_at: row.get(5)?,
                        accessed_at: row.get(6)?,
                        expires_at: row.get(7)?,
                    })
                })?;

                for row in rows {
                    let mem = row?;
                    // Update accessed_at
                    self.conn.execute(
                        "UPDATE memories SET accessed_at = datetime('now') WHERE id = ?1",
                        params![mem.id],
                    )?;
                    memories.push(mem);
                }
            }
            None => {
                let mut stmt = self.conn.prepare(
                    "SELECT m.id, m.kind, m.project, m.content, m.tags, m.created_at, m.accessed_at, m.expires_at
                     FROM memories m
                     JOIN memories_fts fts ON m.rowid = fts.rowid
                     WHERE memories_fts MATCH ?1
                     ORDER BY rank
                     LIMIT ?2",
                )?;

                let rows = stmt.query_map(params![query, limit as i64], |row| {
                    Ok(Memory {
                        id: row.get(0)?,
                        kind: row.get(1)?,
                        project: row.get(2)?,
                        content: row.get(3)?,
                        tags: row.get(4)?,
                        created_at: row.get(5)?,
                        accessed_at: row.get(6)?,
                        expires_at: row.get(7)?,
                    })
                })?;

                for row in rows {
                    let mem = row?;
                    self.conn.execute(
                        "UPDATE memories SET accessed_at = datetime('now') WHERE id = ?1",
                        params![mem.id],
                    )?;
                    memories.push(mem);
                }
            }
        }

        Ok(memories)
    }

    /// List memories with optional filters.
    pub fn list(&self, kind: Option<&str>, project: Option<&str>) -> Result<Vec<Memory>> {
        let mut sql = String::from("SELECT id, kind, project, content, tags, created_at, accessed_at, expires_at FROM memories WHERE 1=1");
        let mut param_values: Vec<String> = Vec::new();

        if let Some(k) = kind {
            sql.push_str(" AND kind = ?");
            param_values.push(k.to_string());
        }
        if let Some(p) = project {
            sql.push_str(" AND project = ?");
            param_values.push(p.to_string());
        }
        sql.push_str(" ORDER BY created_at DESC");

        let mut stmt = self.conn.prepare(&sql)?;

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values
            .iter()
            .map(|s| s as &dyn rusqlite::types::ToSql)
            .collect();

        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(Memory {
                id: row.get(0)?,
                kind: row.get(1)?,
                project: row.get(2)?,
                content: row.get(3)?,
                tags: row.get(4)?,
                created_at: row.get(5)?,
                accessed_at: row.get(6)?,
                expires_at: row.get(7)?,
            })
        })?;

        let mut memories = Vec::new();
        for row in rows {
            memories.push(row?);
        }
        Ok(memories)
    }

    /// Delete a memory by ID.
    pub fn delete(&self, id: &str) -> Result<bool> {
        let deleted = self
            .conn
            .execute("DELETE FROM memories WHERE id = ?1", params![id])?;
        Ok(deleted > 0)
    }

    /// Generate context markdown from recent memories.
    pub fn context(&self, project: Option<&str>, limit: usize) -> Result<String> {
        let memories = match project {
            Some(proj) => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, kind, project, content, tags, created_at, accessed_at, expires_at
                     FROM memories WHERE project = ?1 ORDER BY created_at DESC LIMIT ?2",
                )?;
                let rows = stmt.query_map(params![proj, limit as i64], |row| {
                    Ok(Memory {
                        id: row.get(0)?,
                        kind: row.get(1)?,
                        project: row.get(2)?,
                        content: row.get(3)?,
                        tags: row.get(4)?,
                        created_at: row.get(5)?,
                        accessed_at: row.get(6)?,
                        expires_at: row.get(7)?,
                    })
                })?;
                rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
            }
            None => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, kind, project, content, tags, created_at, accessed_at, expires_at
                     FROM memories ORDER BY created_at DESC LIMIT ?1",
                )?;
                let rows = stmt.query_map(params![limit as i64], |row| {
                    Ok(Memory {
                        id: row.get(0)?,
                        kind: row.get(1)?,
                        project: row.get(2)?,
                        content: row.get(3)?,
                        tags: row.get(4)?,
                        created_at: row.get(5)?,
                        accessed_at: row.get(6)?,
                        expires_at: row.get(7)?,
                    })
                })?;
                rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
            }
        };

        if memories.is_empty() {
            return Ok(String::new());
        }

        let mut output = String::from("## Recent Memories\n\n");
        for mem in &memories {
            let tags_str = match &mem.tags {
                Some(t) if !t.is_empty() => format!(" `{t}`"),
                _ => String::new(),
            };
            let proj_str = match &mem.project {
                Some(p) => format!(" [{p}]"),
                _ => String::new(),
            };
            output.push_str(&format!(
                "- **{}**{}{}: {} ({})\n",
                mem.kind, proj_str, tags_str, mem.content, mem.created_at
            ));
        }
        Ok(output)
    }

    /// Delete expired memories.
    pub fn cleanup(&self) -> Result<usize> {
        let deleted = self.conn.execute(
            "DELETE FROM memories WHERE expires_at IS NOT NULL AND expires_at < datetime('now')",
            params![],
        )?;
        Ok(deleted)
    }
}

/// Generate a unique ID using timestamp + random suffix.
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    // Use simple hex encoding of the timestamp for uniqueness
    format!("{:x}", ts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_store() -> (Store, TempDir) {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("test.db");
        let store = Store::open(&db_path).unwrap();
        (store, temp)
    }

    #[test]
    fn test_open_creates_database() {
        let (store, _temp) = setup_store();
        // Store an item to verify the database works
        let id = store.store("context", None, "test content", None, None).unwrap();
        assert!(!id.is_empty());
    }

    #[test]
    fn test_store_and_recall() {
        let (store, _temp) = setup_store();
        store
            .store("decision", None, "Used JWT for auth", Some("auth,security"), None)
            .unwrap();

        let results = store.recall("JWT", None, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Used JWT for auth");
        assert_eq!(results[0].kind, "decision");
    }

    #[test]
    fn test_recall_with_project_filter() {
        let (store, _temp) = setup_store();
        store
            .store("decision", Some("/project/a"), "Used Postgres", None, None)
            .unwrap();
        store
            .store("decision", Some("/project/b"), "Used MongoDB", None, None)
            .unwrap();

        let results = store.recall("Used", Some("/project/a"), 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Used Postgres");
    }

    #[test]
    fn test_list_filters_by_kind() {
        let (store, _temp) = setup_store();
        store.store("decision", None, "Decision 1", None, None).unwrap();
        store.store("progress", None, "Progress 1", None, None).unwrap();
        store.store("decision", None, "Decision 2", None, None).unwrap();

        let decisions = store.list(Some("decision"), None).unwrap();
        assert_eq!(decisions.len(), 2);

        let progress = store.list(Some("progress"), None).unwrap();
        assert_eq!(progress.len(), 1);
    }

    #[test]
    fn test_delete_removes_memory() {
        let (store, _temp) = setup_store();
        let id = store.store("context", None, "To be deleted", None, None).unwrap();

        let deleted = store.delete(&id).unwrap();
        assert!(deleted);

        let results = store.recall("deleted", None, 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_context_generates_markdown() {
        let (store, _temp) = setup_store();
        store
            .store("decision", None, "Use Rust for CLI", Some("tech"), None)
            .unwrap();
        store
            .store("progress", None, "Phase 1 complete", None, None)
            .unwrap();

        let ctx = store.context(None, 10).unwrap();
        assert!(ctx.contains("## Recent Memories"));
        assert!(ctx.contains("Use Rust for CLI"));
        assert!(ctx.contains("Phase 1 complete"));
        assert!(ctx.contains("`tech`"));
    }

    #[test]
    fn test_cleanup_removes_expired() {
        let (store, _temp) = setup_store();
        store
            .store("context", None, "Temporary", None, Some("2020-01-01T00:00:00Z"))
            .unwrap();
        store
            .store("context", None, "Permanent", None, None)
            .unwrap();

        let deleted = store.cleanup().unwrap();
        assert_eq!(deleted, 1);

        let remaining = store.list(None, None).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].content, "Permanent");
    }
}
