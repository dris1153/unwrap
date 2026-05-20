// db/repo/translations.rs — CRUD for translatable_rows + translations tables.

use std::collections::HashMap;

use rusqlite::{params, Connection};

use crate::domain::{
    error::AppError,
    translate::{TranslatableRow, TranslationEntry, TranslationStatus},
};

// ---------------------------------------------------------------------------
// translatable_rows
// ---------------------------------------------------------------------------

/// Bulk-insert rows in a single transaction; ON CONFLICT(id) DO NOTHING (idempotent).
pub fn bulk_insert_rows(conn: &Connection, rows: &[TranslatableRow]) -> Result<(), AppError> {
    if rows.is_empty() {
        return Ok(());
    }
    let tx = conn.unchecked_transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO translatable_rows
               (id, project_id, source_path, key, comment, source_locale, source_text, detected_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO NOTHING",
        )?;
        for row in rows {
            stmt.execute(params![
                row.id,
                row.project_id,
                row.source_path,
                row.key,
                row.comment,
                row.source_locale,
                row.source_text,
                row.detected_at,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// List rows for a project filtered by source locale.
/// Optional `filter` is applied to key or source_text (case-insensitive substring).
pub fn list_rows(
    conn: &Connection,
    project_id: &str,
    source_locale: &str,
    filter: Option<&str>,
) -> Result<Vec<TranslatableRow>, AppError> {
    let sql = "SELECT id, project_id, source_path, key, comment, source_locale, source_text, detected_at
               FROM translatable_rows
               WHERE project_id = ?1 AND source_locale = ?2
               ORDER BY source_path, key";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![project_id, source_locale], row_to_translatable)?;

    let mut result = Vec::new();
    for r in rows {
        let row = r.map_err(|e| AppError::Db(e.to_string()))?;
        if let Some(f) = filter {
            let f_lower = f.to_lowercase();
            if !row.key.to_lowercase().contains(&f_lower)
                && !row.source_text.to_lowercase().contains(&f_lower)
            {
                continue;
            }
        }
        result.push(row);
    }
    Ok(result)
}

/// Upsert (insert or update) a translation entry.
pub fn upsert_translation(conn: &Connection, entry: &TranslationEntry) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO translations (row_id, target_locale, text, status, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(row_id, target_locale) DO UPDATE SET
           text       = excluded.text,
           status     = excluded.status,
           updated_at = excluded.updated_at",
        params![
            entry.row_id,
            entry.target_locale,
            entry.text,
            entry.status.as_str(),
            entry.updated_at,
        ],
    )?;
    Ok(())
}

/// Fetch all translations for a project + target locale as a HashMap keyed by row_id.
pub fn get_translations_for_project(
    conn: &Connection,
    project_id: &str,
    target_locale: &str,
) -> Result<HashMap<String, TranslationEntry>, AppError> {
    let sql = "SELECT t.row_id, t.target_locale, t.text, t.status, t.updated_at
               FROM translations t
               JOIN translatable_rows r ON r.id = t.row_id
               WHERE r.project_id = ?1 AND t.target_locale = ?2";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![project_id, target_locale], |row| {
        Ok(TranslationEntry {
            row_id: row.get(0)?,
            target_locale: row.get(1)?,
            text: row.get(2)?,
            status: {
                let s: String = row.get(3)?;
                TranslationStatus::from_db_str(&s)
            },
            updated_at: row.get(4)?,
        })
    })?;

    let mut map = HashMap::new();
    for r in rows {
        let entry = r.map_err(|e| AppError::Db(e.to_string()))?;
        map.insert(entry.row_id.clone(), entry);
    }
    Ok(map)
}

/// Count rows by translation status for progress display.
#[allow(dead_code)]
pub struct StatusCounts {
    pub translated: u64,
    pub pending: u64,
    pub review: u64,
    pub total: u64,
}

#[allow(dead_code)]
pub fn count_by_status(
    conn: &Connection,
    project_id: &str,
    target_locale: &str,
) -> Result<StatusCounts, AppError> {
    let total: u64 = conn.query_row(
        "SELECT COUNT(*) FROM translatable_rows WHERE project_id = ?1",
        params![project_id],
        |r| r.get(0),
    )?;

    let translated: u64 = conn.query_row(
        "SELECT COUNT(*) FROM translations t
         JOIN translatable_rows r ON r.id = t.row_id
         WHERE r.project_id = ?1 AND t.target_locale = ?2 AND t.status = 'Translated'",
        params![project_id, target_locale],
        |r| r.get(0),
    )?;

    let review: u64 = conn.query_row(
        "SELECT COUNT(*) FROM translations t
         JOIN translatable_rows r ON r.id = t.row_id
         WHERE r.project_id = ?1 AND t.target_locale = ?2 AND t.status = 'Review'",
        params![project_id, target_locale],
        |r| r.get(0),
    )?;

    let pending = total.saturating_sub(translated).saturating_sub(review);

    Ok(StatusCounts { translated, pending, review, total })
}

/// Distinct source locales detected for a project, with row counts.
pub fn list_source_locales(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<(String, u64)>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT source_locale, COUNT(*) as cnt
         FROM translatable_rows WHERE project_id = ?1
         GROUP BY source_locale ORDER BY cnt DESC",
    )?;
    let rows = stmt.query_map(params![project_id], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, u64>(1)?))
    })?;
    let mut result = Vec::new();
    for r in rows {
        result.push(r.map_err(|e| AppError::Db(e.to_string()))?);
    }
    Ok(result)
}

/// Mark `translatable_scanned = 1` for a project so scan runs only once.
pub fn mark_scanned(conn: &Connection, project_id: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE projects SET translatable_scanned = 1 WHERE id = ?1",
        params![project_id],
    )?;
    Ok(())
}

/// Returns true if `translatable_scanned = 1` for the project.
pub fn is_scanned(conn: &Connection, project_id: &str) -> Result<bool, AppError> {
    let val: i64 = conn.query_row(
        "SELECT translatable_scanned FROM projects WHERE id = ?1",
        params![project_id],
        |r| r.get(0),
    )?;
    Ok(val != 0)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn row_to_translatable(row: &rusqlite::Row<'_>) -> rusqlite::Result<TranslatableRow> {
    Ok(TranslatableRow {
        id: row.get(0)?,
        project_id: row.get(1)?,
        source_path: row.get(2)?,
        key: row.get(3)?,
        comment: row.get(4)?,
        source_locale: row.get(5)?,
        source_text: row.get(6)?,
        detected_at: row.get(7)?,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connect_and_migrate;
    use tempfile::NamedTempFile;

    /// Open a temp-file DB with all migrations applied and a seed project row.
    fn open_test_db() -> (NamedTempFile, Connection) {
        let f = NamedTempFile::new().unwrap();
        let conn = connect_and_migrate(f.path()).unwrap();
        conn.execute(
            "INSERT INTO projects (id, root_path, format_id, engine_version, scripting_backend, created_at, last_opened)
             VALUES ('proj-test', '/tmp/game', 'unity', NULL, 'mono', 0, 0)",
            [],
        )
        .unwrap();
        (f, conn)
    }

    fn make_row(id: &str) -> TranslatableRow {
        TranslatableRow {
            id: id.to_string(),
            project_id: "proj-test".to_string(),
            source_path: "Assets/Strings.txt".to_string(),
            key: format!("key_{id}"),
            comment: None,
            source_locale: "en".to_string(),
            source_text: format!("Hello from {id}"),
            detected_at: 1_000_000,
        }
    }

    #[test]
    fn bulk_insert_and_list_round_trip() {
        let (_f, conn) = open_test_db();
        let rows = vec![make_row("r1"), make_row("r2"), make_row("r3")];
        bulk_insert_rows(&conn, &rows).unwrap();

        let listed = list_rows(&conn, "proj-test", "en", None).unwrap();
        assert_eq!(listed.len(), 3);
        let ids: Vec<&str> = listed.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"r1"));
        assert!(ids.contains(&"r2"));
        assert!(ids.contains(&"r3"));
    }

    #[test]
    fn bulk_insert_idempotent_on_conflict() {
        let (_f, conn) = open_test_db();
        let rows = vec![make_row("r1")];
        bulk_insert_rows(&conn, &rows).unwrap();
        // Second insert must not fail or duplicate
        bulk_insert_rows(&conn, &rows).unwrap();
        let listed = list_rows(&conn, "proj-test", "en", None).unwrap();
        assert_eq!(listed.len(), 1);
    }

    #[test]
    fn upsert_and_retrieve_translation() {
        let (_f, conn) = open_test_db();
        bulk_insert_rows(&conn, &[make_row("r1")]).unwrap();

        let entry = TranslationEntry {
            row_id: "r1".to_string(),
            target_locale: "vi".to_string(),
            text: "Xin chào".to_string(),
            status: TranslationStatus::Translated,
            updated_at: 2_000_000,
        };
        upsert_translation(&conn, &entry).unwrap();

        let map = get_translations_for_project(&conn, "proj-test", "vi").unwrap();
        let fetched = map.get("r1").expect("translation must exist");
        assert_eq!(fetched.text, "Xin chào");
        assert_eq!(fetched.status, TranslationStatus::Translated);
    }

    #[test]
    fn upsert_translation_updates_on_conflict() {
        let (_f, conn) = open_test_db();
        bulk_insert_rows(&conn, &[make_row("r1")]).unwrap();

        let first = TranslationEntry {
            row_id: "r1".to_string(),
            target_locale: "vi".to_string(),
            text: "first".to_string(),
            status: TranslationStatus::Pending,
            updated_at: 1,
        };
        upsert_translation(&conn, &first).unwrap();

        let second = TranslationEntry {
            row_id: "r1".to_string(),
            target_locale: "vi".to_string(),
            text: "second".to_string(),
            status: TranslationStatus::Review,
            updated_at: 2,
        };
        upsert_translation(&conn, &second).unwrap();

        let map = get_translations_for_project(&conn, "proj-test", "vi").unwrap();
        let fetched = map.get("r1").unwrap();
        assert_eq!(fetched.text, "second");
        assert_eq!(fetched.status, TranslationStatus::Review);
    }

    #[test]
    fn list_rows_filter_narrows_results() {
        let (_f, conn) = open_test_db();
        let mut r1 = make_row("r1");
        r1.source_text = "Hello world".to_string();
        let mut r2 = make_row("r2");
        r2.source_text = "Goodbye world".to_string();
        bulk_insert_rows(&conn, &[r1, r2]).unwrap();

        let filtered = list_rows(&conn, "proj-test", "en", Some("Hello")).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "r1");
    }
}
