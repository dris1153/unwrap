// handlers/unity/strings/parse_textasset.rs
// Parses TextAsset files (.json / .csv / .txt) into flat key→text pairs.
// Max file size: 16 MiB. Invalid UTF-8 bytes are replaced with U+FFFD.

use std::collections::HashMap;

use crate::domain::error::AppError;

/// Maximum bytes read from a single TextAsset file.
const MAX_BYTES: usize = 16 * 1024 * 1024;

/// Parse a TextAsset file, returning `(key, text)` pairs.
/// Dispatches based on extension; falls back to key=value parser.
pub fn parse(path: &str, raw: &[u8]) -> Result<Vec<(String, String)>, AppError> {
    if raw.len() > MAX_BYTES {
        return Err(AppError::Io(format!(
            "file too large ({}B > 16MiB): {path}",
            raw.len()
        )));
    }

    // Replace invalid UTF-8 rather than hard-failing.
    let text = String::from_utf8_lossy(raw).into_owned();

    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "json" => parse_json(&text),
        "csv" => parse_csv(&text),
        _ => parse_kv(&text), // .txt, .asset, unknown
    }
}

// ---------------------------------------------------------------------------
// JSON parser — flat or nested (dot-flattened keys)
// ---------------------------------------------------------------------------

fn parse_json(text: &str) -> Result<Vec<(String, String)>, AppError> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| AppError::Io(format!("json parse: {e}")))?;

    let mut out = Vec::new();
    flatten_json("", &value, &mut out);
    Ok(out)
}

fn flatten_json(prefix: &str, value: &serde_json::Value, out: &mut Vec<(String, String)>) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                let key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                flatten_json(&key, v, out);
            }
        }
        serde_json::Value::String(s) => {
            if !prefix.is_empty() && !s.is_empty() {
                out.push((prefix.to_string(), s.clone()));
            }
        }
        // Numbers / bools treated as strings for completeness.
        other => {
            if !prefix.is_empty() {
                let s = other.to_string();
                if !s.is_empty() {
                    out.push((prefix.to_string(), s));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CSV parser — first column = key, second = text, skip header row
// ---------------------------------------------------------------------------

fn parse_csv(text: &str) -> Result<Vec<(String, String)>, AppError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());

    let mut out = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| AppError::Io(format!("csv parse: {e}")))?;
        let key = record.get(0).unwrap_or("").trim().to_string();
        let val = record.get(1).unwrap_or("").trim().to_string();
        if !key.is_empty() && !val.is_empty() {
            out.push((key, val));
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// key=value parser — lines of the form `key=value` or `key: value`
// Comments (#, //) and blank lines are skipped.
// ---------------------------------------------------------------------------

fn parse_kv(text: &str) -> Result<Vec<(String, String)>, AppError> {
    let mut out = Vec::new();
    // Track multi-key→value across all separators
    let sep_map: HashMap<char, ()> = [('=', ()), (':', ())].into();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        // Find first '=' or ':' separator
        if let Some(pos) = line.find(|c| sep_map.contains_key(&c)) {
            let key = line[..pos].trim().to_string();
            let val = line[pos + 1..].trim().to_string();
            if !key.is_empty() && !val.is_empty() {
                out.push((key, val));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_flat() {
        let json = r#"{"menu.title":"Start","menu.quit":"Quit"}"#;
        let rows = parse_json(json).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().any(|(k, v)| k == "menu.title" && v == "Start"));
    }

    #[test]
    fn test_json_nested() {
        let json = r#"{"menu":{"title":"Start","quit":"Quit"}}"#;
        let rows = parse_json(json).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().any(|(k, v)| k == "menu.title" && v == "Start"));
        assert!(rows.iter().any(|(k, v)| k == "menu.quit" && v == "Quit"));
    }

    #[test]
    fn test_csv_basic() {
        let csv = "key,text\nmenu.title,Start\nmenu.quit,Quit\n";
        let rows = parse_csv(csv).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], ("menu.title".to_string(), "Start".to_string()));
    }

    #[test]
    fn test_kv_equals() {
        let txt = "menu.title=Start Game\n# comment\nmenu.quit=Quit";
        let rows = parse_kv(txt).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "menu.title");
        assert_eq!(rows[0].1, "Start Game");
    }

    #[test]
    fn test_parse_dispatch_json() {
        let json = r#"{"a":"b"}"#;
        let rows = parse("strings.json", json.as_bytes()).unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn test_max_size_rejected() {
        let big = vec![b'x'; 17 * 1024 * 1024];
        assert!(parse("big.txt", &big).is_err());
    }
}
