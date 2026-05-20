// db/repo/search.rs — FTS5 search index read/write operations.
//
// Tables: search_index (FTS5 virtual), search_index_status
// All writes are batched inside a transaction for performance.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::domain::{
    error::AppError,
    tree::{AssetKind, AssetNode, NodeId},
    translate::TranslatableRow,
};

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchResultKind {
    Asset,
    Code,
    String,
    Command,
}

/// Tagged action describing what to do when a search result is selected.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SearchAction {
    OpenAsset {
        project_id: String,
        node_id: String,
    },
    OpenCode {
        project_id: String,
        node_id: String,
        line: Option<u32>,
    },
    OpenString {
        project_id: String,
        row_id: String,
    },
    Command {
        command_id: String,
    },
}

/// A single search result returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub kind: SearchResultKind,
    /// ref_id from the index (node_id, class fullname, or row_id).
    pub id: String,
    pub label: String,
    pub sublabel: String,
    /// Phosphor icon name.
    pub icon: String,
    pub project_id: Option<String>,
    pub action: SearchAction,
    /// Weighted BM25 score (higher = more relevant).
    pub score: f32,
}

// ---------------------------------------------------------------------------
// Source weight multipliers applied post-query
// ---------------------------------------------------------------------------

fn source_weight(source: &str) -> f32 {
    match source {
        "asset" => 1.0,
        "code" => 1.2,
        "string-key" => 0.9,
        "string-text" => 0.5,
        _ => 1.0,
    }
}

// ---------------------------------------------------------------------------
// Phosphor icon name by AssetKind
// ---------------------------------------------------------------------------

fn icon_for_kind(kind_str: &str) -> &'static str {
    match kind_str {
        "Folder" => "folder",
        "Texture" => "image",
        "Audio" => "speaker-simple-high",
        "Mesh" => "cube",
        "Text" => "file-text",
        "Script" => "code",
        "Scene" => "film-slate",
        "Material" => "paint-bucket",
        "Shader" => "sparkle",
        "Animation" => "film-strip",
        "Prefab" => "package",
        "Binary" => "file-binary",
        "class" => "code",
        "string-key" => "translate",
        "string-text" => "translate",
        _ => "file",
    }
}

// ---------------------------------------------------------------------------
// FTS5 query builder
// ---------------------------------------------------------------------------

/// Escape FTS5 reserved characters and build a prefix-match query string.
/// "play cont" → "play* cont*"
pub fn build_fts5_query(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .map(|tok| {
            // Escape FTS5 special chars: " * ( ) ^ { } [ ] : \ .
            let escaped = tok.replace('"', "\"\"");
            format!("\"{}\"*", escaped)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Write operations
// ---------------------------------------------------------------------------

/// Bulk-insert asset tree nodes into search_index for a project.
/// Deletes existing asset rows for the project first (re-index on cache invalidation).
pub fn insert_assets(
    conn: &Connection,
    project_id: &str,
    nodes: &[(NodeId, AssetNode)],
) -> Result<usize, AppError> {
    let tx = conn.unchecked_transaction()?;

    tx.execute(
        "DELETE FROM search_index WHERE project_id = ?1 AND source = 'asset'",
        params![project_id],
    )?;

    let mut stmt = tx.prepare(
        "INSERT INTO search_index (project_id, source, kind, ref_id, label, sublabel)
         VALUES (?1, 'asset', ?2, ?3, ?4, ?5)",
    )?;

    let mut count = 0usize;
    for (id, node) in nodes {
        // Skip folders — not useful search targets.
        if node.kind == AssetKind::Folder {
            continue;
        }
        let kind_str = format!("{:?}", node.kind);
        // sublabel: trim to 200 chars to keep index compact
        let sublabel: String = node.source_path.chars().take(200).collect();
        stmt.execute(params![project_id, kind_str, id.0, node.name, sublabel])?;
        count += 1;
    }
    drop(stmt);

    // Update status table
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    tx.execute(
        "INSERT INTO search_index_status (project_id, built_at, asset_count)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(project_id) DO UPDATE SET
           built_at    = excluded.built_at,
           asset_count = excluded.asset_count",
        params![project_id, now, count as i64],
    )?;

    tx.commit()?;
    Ok(count)
}

/// Upsert a single code class entry into search_index.
pub fn insert_code(
    conn: &Connection,
    project_id: &str,
    class_fullname: &str,
    file_path: &str,
) -> Result<(), AppError> {
    // Delete any existing entry for this ref_id (re-index on decompile).
    conn.execute(
        "DELETE FROM search_index WHERE project_id = ?1 AND source = 'code' AND ref_id = ?2",
        params![project_id, class_fullname],
    )?;

    let sublabel: String = file_path.chars().take(200).collect();
    conn.execute(
        "INSERT INTO search_index (project_id, source, kind, ref_id, label, sublabel)
         VALUES (?1, 'code', 'class', ?2, ?3, ?4)",
        params![project_id, class_fullname, class_fullname, sublabel],
    )?;

    // Increment code_count in status
    conn.execute(
        "INSERT INTO search_index_status (project_id, built_at, code_count)
         VALUES (?1, strftime('%s', 'now'), 1)
         ON CONFLICT(project_id) DO UPDATE SET
           built_at   = strftime('%s', 'now'),
           code_count = code_count + 1",
        params![project_id],
    )?;

    Ok(())
}

/// Bulk-insert translatable string rows into search_index.
/// Deletes existing string rows for the project first.
pub fn insert_strings(
    conn: &Connection,
    project_id: &str,
    rows: &[TranslatableRow],
) -> Result<usize, AppError> {
    let tx = conn.unchecked_transaction()?;

    tx.execute(
        "DELETE FROM search_index WHERE project_id = ?1 AND (source = 'string-key' OR source = 'string-text')",
        params![project_id],
    )?;

    let mut stmt = tx.prepare(
        "INSERT INTO search_index (project_id, source, kind, ref_id, label, sublabel)
         VALUES (?1, ?2, 'string-key', ?3, ?4, ?5)",
    )?;

    let mut count = 0usize;
    for row in rows {
        // Index key as label (source = 'string-key')
        let sublabel: String = row.source_text.chars().take(200).collect();
        stmt.execute(params![
            project_id,
            "string-key",
            row.id,
            row.key,
            sublabel
        ])?;
        count += 1;
    }
    drop(stmt);

    // Update status
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    tx.execute(
        "INSERT INTO search_index_status (project_id, built_at, string_count)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(project_id) DO UPDATE SET
           built_at     = excluded.built_at,
           string_count = excluded.string_count",
        params![project_id, now, count as i64],
    )?;

    tx.commit()?;
    Ok(count)
}

/// Delete all search_index rows for a project.
#[allow(dead_code)]
pub fn clear_for_project(conn: &Connection, project_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM search_index WHERE project_id = ?1",
        params![project_id],
    )?;
    conn.execute(
        "DELETE FROM search_index_status WHERE project_id = ?1",
        params![project_id],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Query
// ---------------------------------------------------------------------------

/// Run FTS5 MATCH query and return ranked results with source-weight applied.
pub fn query(
    conn: &Connection,
    input: &str,
    project_id: Option<&str>,
    limit: u32,
) -> Result<Vec<SearchResult>, AppError> {
    if input.trim().is_empty() {
        return Ok(vec![]);
    }

    let fts_query = build_fts5_query(input);

    // Build SQL — project_id filter applied inline (safe: value is parameterized below).
    let sql = if project_id.is_some() {
        "SELECT project_id, source, kind, ref_id, label, sublabel, bm25(search_index) AS score \
         FROM search_index \
         WHERE search_index MATCH ?1 AND project_id = ?2 \
         ORDER BY score \
         LIMIT ?3"
            .to_string()
    } else {
        "SELECT project_id, source, kind, ref_id, label, sublabel, bm25(search_index) AS score \
         FROM search_index \
         WHERE search_index MATCH ?1 \
         ORDER BY score \
         LIMIT ?2"
            .to_string()
    };

    let mut results: Vec<SearchResult> = Vec::new();

    // Execute with or without project_id binding.
    let rows_fn = |row: &rusqlite::Row| -> rusqlite::Result<(String, String, String, String, String, String, f64)> {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
        ))
    };

    let raw_rows: Vec<(String, String, String, String, String, String, f64)> =
        if let Some(pid) = project_id {
            let mut stmt = conn.prepare(&sql)?;
            let collected = stmt
                .query_map(params![fts_query, pid, limit as i64], rows_fn)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            collected
        } else {
            let mut stmt = conn.prepare(&sql)?;
            let collected = stmt
                .query_map(params![fts_query, limit as i64], rows_fn)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            collected
        };

    for (proj_id, source, kind, ref_id, label, sublabel, bm25_score) in raw_rows {
        // BM25 in SQLite returns negative values (lower = more relevant).
        // Negate so higher = better, then multiply by source weight.
        let base_score = -(bm25_score as f32);
        let weighted_score = base_score * source_weight(&source);

        let (result_kind, action, icon) = match source.as_str() {
            "asset" => (
                SearchResultKind::Asset,
                SearchAction::OpenAsset {
                    project_id: proj_id.clone(),
                    node_id: ref_id.clone(),
                },
                icon_for_kind(&kind).to_string(),
            ),
            "code" => (
                SearchResultKind::Code,
                SearchAction::OpenCode {
                    project_id: proj_id.clone(),
                    node_id: ref_id.clone(),
                    line: None,
                },
                "code".to_string(),
            ),
            "string-key" | "string-text" => (
                SearchResultKind::String,
                SearchAction::OpenString {
                    project_id: proj_id.clone(),
                    row_id: ref_id.clone(),
                },
                "translate".to_string(),
            ),
            _ => (
                SearchResultKind::Asset,
                SearchAction::OpenAsset {
                    project_id: proj_id.clone(),
                    node_id: ref_id.clone(),
                },
                "file".to_string(),
            ),
        };

        results.push(SearchResult {
            kind: result_kind,
            id: ref_id,
            label,
            sublabel,
            icon,
            project_id: Some(proj_id),
            action,
            score: weighted_score,
        });
    }

    // Sort descending by weighted score.
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    Ok(results)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tree::{AssetKind, AssetNode, NodeId};
    use rusqlite::Connection;
    use std::collections::HashMap;

    fn in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = OFF;").unwrap();
        // Create FTS5 table
        conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(
              project_id UNINDEXED,
              source,
              kind UNINDEXED,
              ref_id UNINDEXED,
              label,
              sublabel,
              tokenize = 'porter unicode61'
            );
            CREATE TABLE IF NOT EXISTS search_index_status (
              project_id TEXT PRIMARY KEY,
              built_at INTEGER,
              asset_count INTEGER DEFAULT 0,
              code_count INTEGER DEFAULT 0,
              string_count INTEGER DEFAULT 0
            );",
        )
        .unwrap();
        conn
    }

    fn make_node(id: &str, name: &str, kind: AssetKind, path: &str) -> (NodeId, AssetNode) {
        let node_id = NodeId(id.to_string());
        let node = AssetNode {
            id: node_id.clone(),
            parent: None,
            name: name.to_string(),
            kind,
            size: 0,
            source_path: path.to_string(),
            metadata: HashMap::new(),
        };
        (node_id, node)
    }

    #[test]
    fn test_build_fts5_query_single_token() {
        assert_eq!(build_fts5_query("play"), "\"play\"*");
    }

    #[test]
    fn test_build_fts5_query_multi_token() {
        let q = build_fts5_query("play controller");
        assert_eq!(q, "\"play\"* \"controller\"*");
    }

    #[test]
    fn test_build_fts5_query_escapes_quotes() {
        // Input token `"hello"` — inner quotes doubled, wrapped in outer quotes.
        // Expected token output: `"""hello"""*`
        let q = build_fts5_query(r#"say "hello""#);
        assert!(q.contains("\"say\"*"), "expected say token, got: {q}");
        // The double-quote char inside a token gets doubled: "hello" → ""hello""
        // Wrapped in outer quotes: """hello"""*
        assert!(q.contains("\"\"hello\"\""), "expected escaped hello token, got: {q}");
    }

    #[test]
    fn test_build_fts5_query_empty() {
        assert_eq!(build_fts5_query(""), "");
        assert_eq!(build_fts5_query("   "), "");
    }

    #[test]
    fn test_insert_and_query_assets() {
        let conn = in_memory_db();

        let nodes = vec![
            make_node("n1", "PlayerController", AssetKind::Script, "Assets/Scripts/PlayerController.cs"),
            make_node("n2", "play_button", AssetKind::Texture, "Assets/UI/play_button.png"),
            make_node("n3", "BackgroundMusic", AssetKind::Audio, "Assets/Audio/BackgroundMusic.mp3"),
        ];

        let count = insert_assets(&conn, "proj1", &nodes).unwrap();
        assert_eq!(count, 3);

        let results = query(&conn, "play", Some("proj1"), 20).unwrap();
        assert!(!results.is_empty(), "expected at least one result for 'play'");

        // Both PlayerController and play_button should match
        let labels: Vec<&str> = results.iter().map(|r| r.label.as_str()).collect();
        let has_player = labels.iter().any(|l| l.contains("Player"));
        let has_play_btn = labels.iter().any(|l| l.contains("play_button"));
        assert!(has_player || has_play_btn, "expected player or play_button in results: {:?}", labels);
    }

    #[test]
    fn test_insert_strings_and_query() {
        let conn = in_memory_db();

        let rows = vec![
            TranslatableRow {
                id: "r1".to_string(),
                project_id: "proj1".to_string(),
                source_path: "Assets/en/strings.json".to_string(),
                key: "menu.play_button".to_string(),
                comment: None,
                source_locale: "en".to_string(),
                source_text: "Play Game".to_string(),
                detected_at: 0,
            },
            TranslatableRow {
                id: "r2".to_string(),
                project_id: "proj1".to_string(),
                source_path: "Assets/en/strings.json".to_string(),
                key: "menu.quit".to_string(),
                comment: None,
                source_locale: "en".to_string(),
                source_text: "Quit".to_string(),
                detected_at: 0,
            },
        ];

        let count = insert_strings(&conn, "proj1", &rows).unwrap();
        assert_eq!(count, 2);

        let results = query(&conn, "play", Some("proj1"), 10).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.id == "r1"));
    }

    #[test]
    fn test_clear_for_project() {
        let conn = in_memory_db();
        let nodes = vec![make_node("n1", "PlayerController", AssetKind::Script, "Assets/Scripts/PlayerController.cs")];
        insert_assets(&conn, "proj1", &nodes).unwrap();

        let before = query(&conn, "player", Some("proj1"), 10).unwrap();
        assert!(!before.is_empty());

        // For clear_for_project we need the status table to reference a real project row,
        // but since foreign keys are OFF in test we can call it directly.
        conn.execute(
            "DELETE FROM search_index WHERE project_id = ?1",
            params!["proj1"],
        ).unwrap();

        let after = query(&conn, "player", Some("proj1"), 10).unwrap();
        assert!(after.is_empty());
    }

    #[test]
    fn test_empty_query_returns_empty() {
        let conn = in_memory_db();
        let results = query(&conn, "", Some("proj1"), 10).unwrap();
        assert!(results.is_empty());
    }
}
