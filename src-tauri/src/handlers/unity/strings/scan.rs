// handlers/unity/strings/scan.rs
// Walks the AssetTree, finds translatable nodes, parses them, bulk-inserts into DB.
// Idempotent: ON CONFLICT(id) DO NOTHING in bulk_insert_rows.

use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use sha2::{Digest, Sha256};
use tracing::{debug, warn};

use crate::{
    commands::search::index_strings_sync,
    domain::{
        error::AppError,
        translate::TranslatableRow,
        tree::{AssetKind, AssetTree},
    },
    db::repo::translations,
};

use super::{locale_detect, parse_stringtable, parse_textasset};

/// Walk the asset tree, parse translatable nodes, bulk-insert rows.
/// Safe to call multiple times — duplicate rows are silently skipped.
pub fn scan_translatable(
    project_id: &str,
    tree: &AssetTree,
    conn: &Connection,
) -> Result<usize, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let mut rows: Vec<TranslatableRow> = Vec::new();

    for node in tree.nodes.values() {
        // Only process text-kind nodes.
        if node.kind != AssetKind::Text {
            continue;
        }

        let source_path = &node.source_path;
        let name_lower = node.name.to_lowercase();

        // Heuristic: skip files that are not locale-like.
        // Accept if name contains locale clue or path has Locali(z|s)ation segment.
        if !is_translatable_candidate(source_path, &name_lower) {
            continue;
        }

        let locale = locale_detect::extract_locale(source_path);

        // Read file content — skip if not accessible.
        let raw = match std::fs::read(source_path) {
            Ok(b) => b,
            Err(e) => {
                warn!(path = source_path, "cannot read text asset: {e}");
                continue;
            }
        };

        // Dispatch to parser based on extension / name hint.
        let pairs = if name_lower.ends_with(".asset") {
            parse_stringtable::parse(source_path, &raw)
        } else {
            parse_textasset::parse(source_path, &raw)
        };

        let pairs = match pairs {
            Ok(p) => p,
            Err(e) => {
                warn!(path = source_path, "parse error: {e}");
                continue;
            }
        };

        debug!(
            path = source_path,
            locale = locale,
            count = pairs.len(),
            "parsed text asset"
        );

        for (key, text) in pairs {
            let id = stable_id(source_path, &key);
            rows.push(TranslatableRow {
                id,
                project_id: project_id.to_string(),
                source_path: source_path.clone(),
                key,
                comment: None,
                source_locale: locale.clone(),
                source_text: text,
                detected_at: now,
            });
        }
    }

    let count = rows.len();
    translations::bulk_insert_rows(conn, &rows)?;
    translations::mark_scanned(conn, project_id)?;

    // Fire-and-forget FTS5 string indexing (runs in same sync context — acceptable latency).
    if let Err(e) = index_strings_sync(conn, project_id) {
        tracing::warn!(project_id = %project_id, error = %e, "search index: string indexing failed");
    }

    Ok(count)
}

/// Returns true if the path looks like it contains localizable strings.
fn is_translatable_candidate(source_path: &str, name_lower: &str) -> bool {
    let path_lower = source_path.to_lowercase();

    // Explicit localization directory segments.
    if path_lower.contains("/locali")
        || path_lower.contains("\\locali")
        || path_lower.contains("/locale")
        || path_lower.contains("\\locale")
    {
        return true;
    }

    // StringTable YAML assets from Unity Localization package.
    if name_lower.contains("stringtable") && name_lower.ends_with(".asset") {
        return true;
    }

    // Filename patterns like "strings_en.json", "ui_strings.txt", "dialogue_vi.csv".
    if (name_lower.contains("string") || name_lower.contains("text") || name_lower.contains("dialogue"))
        && (name_lower.ends_with(".json")
            || name_lower.ends_with(".csv")
            || name_lower.ends_with(".txt"))
    {
        return true;
    }

    false
}

/// Stable row ID: hex(sha256(source_path + "\x00" + key)[..16]).
fn stable_id(source_path: &str, key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_path.as_bytes());
    hasher.update(b"\x00");
    hasher.update(key.as_bytes());
    hex::encode(&hasher.finalize()[..8])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stable_id_deterministic() {
        let id1 = stable_id("Assets/en/strings.json", "menu.title");
        let id2 = stable_id("Assets/en/strings.json", "menu.title");
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 16); // 8 bytes = 16 hex chars
    }

    #[test]
    fn test_stable_id_different_keys() {
        let id1 = stable_id("Assets/en/strings.json", "menu.title");
        let id2 = stable_id("Assets/en/strings.json", "menu.quit");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_is_translatable_candidate_localization_dir() {
        assert!(is_translatable_candidate(
            "Assets/Localization/en/strings.json",
            "strings.json"
        ));
    }

    #[test]
    fn test_is_translatable_candidate_stringtable() {
        assert!(is_translatable_candidate(
            "Assets/UI/StringTable_en.asset",
            "stringtable_en.asset"
        ));
    }

    #[test]
    fn test_is_translatable_candidate_negative() {
        assert!(!is_translatable_candidate(
            "Assets/Scripts/PlayerController.cs",
            "playercontroller.cs"
        ));
    }
}
