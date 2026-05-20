// asset_trees.rs — persist / retrieve compressed AssetTree blobs.
//
// Compression: blobs > 4 KiB are gzip-compressed via flate2 before storage.
// A 1-byte magic prefix distinguishes compressed from raw:
//   0x1f 0x8b … = gzip magic (compressed)
//   anything else = raw JSON
//
// Security: decompression is bounded to 50 MiB to prevent decompression bombs.
// Uses std::io::Read::take(50 * 1024 * 1024) before reading into memory.
#![allow(dead_code)]

use std::io::{Read, Write};

use rusqlite::{params, Connection};

use crate::domain::{error::AppError, tree::AssetTree};

/// Maximum inflated size accepted during decompression (50 MiB).
const MAX_INFLATED_BYTES: u64 = 50 * 1024 * 1024;

/// Threshold above which JSON blobs are gzip-compressed before storage.
const COMPRESS_THRESHOLD: usize = 4096;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Persist a serialized (optionally compressed) asset tree for a project.
pub fn put(
    conn: &Connection,
    project_id: &str,
    source_hash: &str,
    tree: &AssetTree,
) -> Result<(), AppError> {
    let json = serde_json::to_vec(tree)?;

    let blob = if json.len() > COMPRESS_THRESHOLD {
        let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        enc.write_all(&json)
            .map_err(|e| AppError::Io(format!("gz compress: {e}")))?;
        enc.finish()
            .map_err(|e| AppError::Io(format!("gz finish: {e}")))?
    } else {
        json
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO asset_trees (project_id, source_hash, json_blob, built_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(project_id) DO UPDATE SET
           source_hash = excluded.source_hash,
           json_blob   = excluded.json_blob,
           built_at    = excluded.built_at",
        params![project_id, source_hash, blob, now],
    )?;

    Ok(())
}

/// Retrieve a cached asset tree for `project_id`.
///
/// Returns `None` if not present or if `expected_hash` differs from the stored hash
/// (stale cache — caller should rebuild).
pub fn get(
    conn: &Connection,
    project_id: &str,
    expected_hash: &str,
) -> Result<Option<AssetTree>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT source_hash, json_blob FROM asset_trees WHERE project_id = ?1",
    )?;

    let mut rows = stmt.query(params![project_id])?;

    if let Some(row) = rows.next()? {
        let cached_hash: String = row.get(0)?;
        if cached_hash != expected_hash {
            return Ok(None); // Stale — caller rebuilds.
        }
        let blob: Vec<u8> = row.get(1)?;
        let json = decompress_blob(&blob)?;
        let tree: AssetTree = serde_json::from_slice(&json)
            .map_err(|e| AppError::Db(format!("deserialize asset tree: {e}")))?;
        Ok(Some(tree))
    } else {
        Ok(None)
    }
}

/// Delete a cached tree (e.g. after project removal).
pub fn delete(conn: &Connection, project_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM asset_trees WHERE project_id = ?1",
        params![project_id],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Decompression helper
// ---------------------------------------------------------------------------

/// Decompress a blob if it begins with the gzip magic bytes (0x1f 0x8b).
/// Raw JSON blobs are returned as-is.
///
/// Bounded to MAX_INFLATED_BYTES to prevent decompression-bomb attacks.
fn decompress_blob(blob: &[u8]) -> Result<Vec<u8>, AppError> {
    // Gzip magic: 0x1f 0x8b
    if blob.starts_with(&[0x1f, 0x8b]) {
        let cursor = std::io::Cursor::new(blob);
        let decoder = flate2::read::GzDecoder::new(cursor);
        let mut out = Vec::new();
        decoder
            .take(MAX_INFLATED_BYTES)
            .read_to_end(&mut out)
            .map_err(|e| AppError::Io(format!("gz decompress: {e}")))?;
        Ok(out)
    } else {
        Ok(blob.to_vec())
    }
}
