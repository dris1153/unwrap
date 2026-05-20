// commands/search.rs — Tauri command surface for FTS5 search + index management.

use std::sync::Arc;

use tauri::State;
use tokio::sync::Mutex;
use tracing::info;

use crate::{
    db,
    domain::{error::AppError, translate::TranslatableRow},
};

pub use db::repo::search::SearchResult;

/// Full-text search across indexed assets, code, and strings.
///
/// Returns up to `limit` (default 50) results ranked by weighted BM25 score.
/// Results scoped to `project_id` when provided.
#[tauri::command]
pub async fn search(
    query: String,
    project_id: Option<String>,
    limit: Option<u32>,
    db_state: State<'_, Arc<Mutex<rusqlite::Connection>>>,
) -> Result<Vec<SearchResult>, AppError> {
    let limit = limit.unwrap_or(50).min(200);
    let conn = db_state.lock().await;
    tokio::task::block_in_place(|| {
        db::repo::search::query(&conn, &query, project_id.as_deref(), limit)
    })
}

/// Index all asset tree nodes for a project (background task after open).
///
/// Loads the cached tree from SQLite and bulk-inserts into search_index.
#[tauri::command]
pub async fn index_assets(
    project_id: String,
    db_state: State<'_, Arc<Mutex<rusqlite::Connection>>>,
) -> Result<(), AppError> {
    let conn = db_state.lock().await;
    let count = tokio::task::block_in_place(|| {
        // Load tree blob directly — reuse the same logic as tree command.
        let mut stmt = conn
            .prepare("SELECT json_blob FROM asset_trees WHERE project_id = ?1")
            .map_err(|e| AppError::Db(e.to_string()))?;
        let mut rows = stmt
            .query(rusqlite::params![project_id])
            .map_err(|e| AppError::Db(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| AppError::Db(e.to_string()))? {
            let blob: Vec<u8> = row.get(0).map_err(|e| AppError::Db(e.to_string()))?;

            // Decompress if gzip (copy of asset_trees logic — avoids circular dep).
            let json = if blob.starts_with(&[0x1f, 0x8b]) {
                use std::io::Read;
                let cursor = std::io::Cursor::new(&blob);
                let decoder = flate2::read::GzDecoder::new(cursor);
                let mut out = Vec::new();
                decoder
                    .take(50 * 1024 * 1024)
                    .read_to_end(&mut out)
                    .map_err(|e| AppError::Io(format!("gz decompress: {e}")))?;
                out
            } else {
                blob
            };

            let tree: crate::domain::tree::AssetTree = serde_json::from_slice(&json)
                .map_err(|e| AppError::Db(format!("deserialize tree: {e}")))?;

            let node_pairs: Vec<(crate::domain::tree::NodeId, crate::domain::tree::AssetNode)> =
                tree.nodes
                    .into_values()
                    .map(|v| (v.id.clone(), v))
                    .collect();

            db::repo::search::insert_assets(&conn, &project_id, &node_pairs)
        } else {
            Ok(0)
        }
    })?;

    info!(project_id = %project_id, count = count, "search index: assets indexed");
    Ok(())
}

/// Upsert a single decompiled class into the search index.
/// Called fire-and-forget after a successful decompile.
#[tauri::command]
pub async fn index_code(
    project_id: String,
    class_fullname: String,
    file_path: String,
    db_state: State<'_, Arc<Mutex<rusqlite::Connection>>>,
) -> Result<(), AppError> {
    let conn = db_state.lock().await;
    tokio::task::block_in_place(|| {
        db::repo::search::insert_code(&conn, &project_id, &class_fullname, &file_path)
    })?;
    info!(project_id = %project_id, class = %class_fullname, "search index: code entry indexed");
    Ok(())
}

/// Bulk-index translatable string rows for a project.
/// Reads directly from translatable_rows table — no rows param needed over IPC.
#[tauri::command]
pub async fn index_strings(
    project_id: String,
    db_state: State<'_, Arc<Mutex<rusqlite::Connection>>>,
) -> Result<(), AppError> {
    let conn = db_state.lock().await;
    let count = tokio::task::block_in_place(|| {
        // Load rows from translatable_rows directly.
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, source_path, key, comment, source_locale, source_text, detected_at \
                 FROM translatable_rows WHERE project_id = ?1",
            )
            .map_err(|e| AppError::Db(e.to_string()))?;

        let rows: Vec<TranslatableRow> = stmt
            .query_map(rusqlite::params![project_id], |row| {
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
            })
            .map_err(|e| AppError::Db(e.to_string()))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| AppError::Db(e.to_string()))?;

        db::repo::search::insert_strings(&conn, &project_id, &rows)
    })?;

    info!(project_id = %project_id, count = count, "search index: strings indexed");
    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helper — callable from other modules without going through IPC.
// Used by handlers that already hold the db lock or need sync indexing.
// ---------------------------------------------------------------------------

/// Synchronous asset indexing — for use inside block_in_place contexts.
pub fn index_assets_sync(
    conn: &rusqlite::Connection,
    project_id: &str,
) -> Result<usize, AppError> {
    let mut stmt = conn
        .prepare("SELECT json_blob FROM asset_trees WHERE project_id = ?1")
        .map_err(|e| AppError::Db(e.to_string()))?;
    let mut rows = stmt
        .query(rusqlite::params![project_id])
        .map_err(|e| AppError::Db(e.to_string()))?;

    if let Some(row) = rows.next().map_err(|e| AppError::Db(e.to_string()))? {
        let blob: Vec<u8> = row.get(0).map_err(|e| AppError::Db(e.to_string()))?;

        let json = if blob.starts_with(&[0x1f, 0x8b]) {
            use std::io::Read;
            let cursor = std::io::Cursor::new(&blob);
            let decoder = flate2::read::GzDecoder::new(cursor);
            let mut out = Vec::new();
            decoder
                .take(50 * 1024 * 1024)
                .read_to_end(&mut out)
                .map_err(|e| AppError::Io(format!("gz decompress: {e}")))?;
            out
        } else {
            blob
        };

        let tree: crate::domain::tree::AssetTree = serde_json::from_slice(&json)
            .map_err(|e| AppError::Db(format!("deserialize tree: {e}")))?;

        let node_pairs: Vec<(crate::domain::tree::NodeId, crate::domain::tree::AssetNode)> = tree
            .nodes
            .into_values()
            .map(|v| (v.id.clone(), v))
            .collect();

        db::repo::search::insert_assets(conn, project_id, &node_pairs)
    } else {
        Ok(0)
    }
}

/// Synchronous string indexing — callable inside block_in_place.
pub fn index_strings_sync(
    conn: &rusqlite::Connection,
    project_id: &str,
) -> Result<usize, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, source_path, key, comment, source_locale, source_text, detected_at \
             FROM translatable_rows WHERE project_id = ?1",
        )
        .map_err(|e| AppError::Db(e.to_string()))?;

    let rows: Vec<TranslatableRow> = stmt
        .query_map(rusqlite::params![project_id], |row| {
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
        })
        .map_err(|e| AppError::Db(e.to_string()))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| AppError::Db(e.to_string()))?;

    db::repo::search::insert_strings(conn, project_id, &rows)
}
