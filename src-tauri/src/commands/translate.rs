// commands/translate.rs — IPC commands for the translatable text view.

use std::{collections::HashMap, sync::Arc};

use tauri::State;
use tokio::sync::Mutex;

use crate::{
    db::repo::translations,
    domain::{
        error::AppError,
        translate::{
            LocaleEntry, LocaleKind, TranslatableRowWithDraft, TranslationEntry, TranslationStatus,
        },
    },
    handlers::FormatHandlerRegistry,
};

// ISO-639-1 fallback list shown in the target locale picker.
const ISO_LOCALES: &[&str] = &[
    "af", "sq", "ar", "hy", "az", "eu", "be", "bn", "bs", "bg", "ca", "zh", "zh-CN", "zh-TW",
    "hr", "cs", "da", "nl", "en", "eo", "et", "fi", "fr", "gl", "ka", "de", "el", "gu", "ht",
    "he", "hi", "hu", "id", "ga", "it", "ja", "kn", "kk", "ko", "ku", "la", "lv", "lt", "mk",
    "ms", "ml", "mt", "mr", "mn", "ne", "no", "fa", "pl", "pt", "pa", "ro", "ru", "sr", "sk",
    "sl", "es", "sw", "sv", "ta", "te", "th", "tr", "uk", "ur", "uz", "vi", "cy", "yi",
];

/// List translatable rows for a project, optionally filtered.
/// Triggers a lazy scan on first call if `translatable_scanned` is 0.
///
/// # Arguments
/// * `handle_id` — project id string
/// * `source_locale` — e.g. "en"
/// * `filter` — optional substring filter on key or source_text
#[tauri::command]
pub async fn list_translatable(
    handle_id: String,
    source_locale: String,
    filter: Option<String>,
    target_locale: String,
    db: State<'_, Arc<Mutex<rusqlite::Connection>>>,
    _registry: State<'_, Arc<FormatHandlerRegistry>>,
) -> Result<Vec<TranslatableRowWithDraft>, AppError> {
    let conn = db.lock().await;

    // Lazy scan: run only once per project.
    let already_scanned =
        tokio::task::block_in_place(|| translations::is_scanned(&conn, &handle_id))?;

    if !already_scanned {
        // Retrieve tree from DB to walk nodes.
        let tree = tokio::task::block_in_place(|| {
            let mut stmt = conn
                .prepare("SELECT json_blob FROM asset_trees WHERE project_id = ?1")
                .map_err(|e| AppError::Db(e.to_string()))?;
            let mut rows = stmt
                .query(rusqlite::params![handle_id])
                .map_err(|e| AppError::Db(e.to_string()))?;
            if let Some(row) = rows.next().map_err(|e| AppError::Db(e.to_string()))? {
                let blob: Vec<u8> = row.get(0).map_err(|e| AppError::Db(e.to_string()))?;
                // Decompress if gzip-prefixed.
                let json = decompress_blob(&blob)?;
                let tree: crate::domain::tree::AssetTree = serde_json::from_slice(&json)
                    .map_err(|e| AppError::Db(format!("deserialize tree: {e}")))?;
                Ok::<_, AppError>(Some(tree))
            } else {
                Ok(None)
            }
        })?;

        if let Some(tree) = tree {
            tokio::task::block_in_place(|| {
                crate::handlers::unity::strings::scan::scan_translatable(
                    &handle_id,
                    &tree,
                    &conn,
                )
            })?;
        }
    }

    // Fetch rows + join translations.
    let rows = tokio::task::block_in_place(|| {
        translations::list_rows(&conn, &handle_id, &source_locale, filter.as_deref())
    })?;

    let translation_map = tokio::task::block_in_place(|| {
        translations::get_translations_for_project(&conn, &handle_id, &target_locale)
    })?;

    let result = rows
        .into_iter()
        .map(|row| {
            let translation = translation_map.get(&row.id).cloned();
            TranslatableRowWithDraft { row, translation }
        })
        .collect();

    Ok(result)
}

/// Save (upsert) a translation entry.
#[tauri::command]
pub async fn save_translation(
    row_id: String,
    target_locale: String,
    text: String,
    status: String,
    db: State<'_, Arc<Mutex<rusqlite::Connection>>>,
) -> Result<(), AppError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let entry = TranslationEntry {
        row_id,
        target_locale,
        text,
        status: TranslationStatus::from_db_str(&status),
        updated_at: now,
    };

    let conn = db.lock().await;
    tokio::task::block_in_place(|| translations::upsert_translation(&conn, &entry))
}

/// List available locales for a project.
/// Returns source locales (detected from files) + ISO-639 fallback targets.
#[tauri::command]
pub async fn list_locales(
    handle_id: String,
    db: State<'_, Arc<Mutex<rusqlite::Connection>>>,
) -> Result<Vec<LocaleEntry>, AppError> {
    let conn = db.lock().await;
    let source_locales =
        tokio::task::block_in_place(|| translations::list_source_locales(&conn, &handle_id))?;

    let source_codes: std::collections::HashSet<String> =
        source_locales.iter().map(|(c, _)| c.clone()).collect();

    let mut result: Vec<LocaleEntry> = source_locales
        .into_iter()
        .map(|(code, count)| LocaleEntry {
            code,
            count,
            kind: LocaleKind::Source,
        })
        .collect();

    // Add ISO fallback targets not already in source list.
    for &code in ISO_LOCALES {
        if !source_codes.contains(code) {
            result.push(LocaleEntry {
                code: code.to_string(),
                count: 0,
                kind: LocaleKind::Target,
            });
        }
    }

    Ok(result)
}

/// Export translations to a CSV or JSON file at `dest_path`.
#[tauri::command]
pub async fn export_translations(
    handle_id: String,
    target_locale: String,
    format: String,
    dest_path: String,
    db: State<'_, Arc<Mutex<rusqlite::Connection>>>,
) -> Result<(), AppError> {
    // Validate dest_path is not empty.
    if dest_path.trim().is_empty() {
        return Err(AppError::InvalidPath("dest_path is empty".into()));
    }

    let conn = db.lock().await;

    let rows = tokio::task::block_in_place(|| {
        translations::list_rows(&conn, &handle_id, "en", None)
    })?;

    let translation_map: HashMap<String, TranslationEntry> = tokio::task::block_in_place(|| {
        translations::get_translations_for_project(&conn, &handle_id, &target_locale)
    })?;

    // Ensure parent directory exists.
    if let Some(parent) = std::path::Path::new(&dest_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Io(format!("create export dir: {e}")))?;
    }

    match format.to_lowercase().as_str() {
        "csv" => export_csv(&dest_path, &rows, &translation_map),
        "json" => export_json(&dest_path, &rows, &translation_map),
        other => Err(AppError::InvalidPath(format!("unknown export format: {other}"))),
    }
}

// ---------------------------------------------------------------------------
// Export helpers
// ---------------------------------------------------------------------------

fn export_csv(
    dest_path: &str,
    rows: &[crate::domain::translate::TranslatableRow],
    translations: &HashMap<String, TranslationEntry>,
) -> Result<(), AppError> {
    let mut wtr = csv::Writer::from_path(dest_path)
        .map_err(|e| AppError::Io(format!("csv open: {e}")))?;

    wtr.write_record(["key", "source", "translation", "status"])
        .map_err(|e| AppError::Io(format!("csv header: {e}")))?;

    for row in rows {
        let (translation_text, status) = match translations.get(&row.id) {
            Some(t) => (t.text.as_str(), t.status.as_str()),
            None => ("", "Pending"),
        };
        wtr.write_record([
            row.key.as_str(),
            row.source_text.as_str(),
            translation_text,
            status,
        ])
        .map_err(|e| AppError::Io(format!("csv write: {e}")))?;
    }

    wtr.flush()
        .map_err(|e| AppError::Io(format!("csv flush: {e}")))?;
    Ok(())
}

fn export_json(
    dest_path: &str,
    rows: &[crate::domain::translate::TranslatableRow],
    translations: &HashMap<String, TranslationEntry>,
) -> Result<(), AppError> {
    let mut map = serde_json::Map::new();
    for row in rows {
        if let Some(t) = translations.get(&row.id) {
            if !t.text.is_empty() {
                map.insert(row.key.clone(), serde_json::Value::String(t.text.clone()));
            }
        }
    }
    let json = serde_json::to_string_pretty(&serde_json::Value::Object(map))
        .map_err(|e| AppError::Db(format!("json serialize: {e}")))?;
    std::fs::write(dest_path, json).map_err(|e| AppError::Io(format!("write json: {e}")))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Blob decompression (mirrors asset_trees.rs logic)
// ---------------------------------------------------------------------------

fn decompress_blob(blob: &[u8]) -> Result<Vec<u8>, AppError> {
    use std::io::Read;
    if blob.starts_with(&[0x1f, 0x8b]) {
        let cursor = std::io::Cursor::new(blob);
        let decoder = flate2::read::GzDecoder::new(cursor);
        let mut out = Vec::new();
        decoder
            .take(50 * 1024 * 1024)
            .read_to_end(&mut out)
            .map_err(|e| AppError::Io(format!("gz decompress: {e}")))?;
        Ok(out)
    } else {
        Ok(blob.to_vec())
    }
}
