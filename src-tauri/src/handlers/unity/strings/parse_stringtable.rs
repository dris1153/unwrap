// handlers/unity/strings/parse_stringtable.rs
// Parses Unity Localization package StringTable YAML (.asset) files.
// AssetRipper emits these as YAML documents. Parsing is best-effort and
// defensive — unparseable rows are skipped with a warning log.
//
// Expected structure (simplified):
//   MonoBehaviour:
//     m_TableEntries:
//     - Key: "menu.title"
//       Id: 1
//       Localized Values:
//         m_Localized: "Start Game"
//     ...

use tracing::warn;
use yaml_rust2::{Yaml, YamlLoader};

use crate::domain::error::AppError;

/// Parse a StringTable YAML asset, returning `(key, text)` pairs.
/// Skips entries that cannot be parsed cleanly.
pub fn parse(path: &str, raw: &[u8]) -> Result<Vec<(String, String)>, AppError> {
    let text = String::from_utf8_lossy(raw).into_owned();

    let docs = YamlLoader::load_from_str(&text).map_err(|e| {
        AppError::Io(format!("yaml parse error in {path}: {e}"))
    })?;

    let mut out = Vec::new();

    for doc in &docs {
        if let Yaml::Hash(root) = doc {
            // Walk to MonoBehaviour.m_TableEntries
            for (k, v) in root {
                if k.as_str() == Some("MonoBehaviour") {
                    collect_entries(v, path, &mut out);
                }
            }
        }
    }

    Ok(out)
}

fn collect_entries(mono: &Yaml, path: &str, out: &mut Vec<(String, String)>) {
    let entries = match mono {
        Yaml::Hash(h) => h.get(&Yaml::String("m_TableEntries".to_string())),
        _ => return,
    };

    let entries = match entries {
        Some(Yaml::Array(arr)) => arr,
        _ => return,
    };

    for entry in entries {
        match extract_entry(entry) {
            Some(pair) => out.push(pair),
            None => {
                warn!(path, "skipping unparseable StringTable entry");
            }
        }
    }
}

fn extract_entry(entry: &Yaml) -> Option<(String, String)> {
    let map = match entry {
        Yaml::Hash(h) => h,
        _ => return None,
    };

    // Key may be stored as "Key", "m_Key", or "Id" (fall back to numeric string).
    let key = map
        .get(&Yaml::String("Key".to_string()))
        .or_else(|| map.get(&Yaml::String("m_Key".to_string())))
        .and_then(|v| match v {
            Yaml::String(s) => Some(s.clone()),
            Yaml::Integer(i) => Some(i.to_string()),
            _ => None,
        })?;

    // Text lives under various nested shapes — try common ones.
    let text = find_text(map)?;

    if key.is_empty() || text.is_empty() {
        return None;
    }

    Some((key, text))
}

/// Try several common YAML shapes Unity Localization package uses for the string value.
fn find_text(map: &yaml_rust2::yaml::Hash) -> Option<String> {
    // Shape 1: direct "m_Localized" at top level
    if let Some(Yaml::String(s)) = map.get(&Yaml::String("m_Localized".to_string())) {
        return Some(s.clone());
    }

    // Shape 2: "Localized Values" → "m_Localized"
    if let Some(Yaml::Hash(lv)) = map.get(&Yaml::String("Localized Values".to_string())) {
        if let Some(Yaml::String(s)) = lv.get(&Yaml::String("m_Localized".to_string())) {
            return Some(s.clone());
        }
    }

    // Shape 3: flat "Value" key
    if let Some(Yaml::String(s)) = map.get(&Yaml::String("Value".to_string())) {
        return Some(s.clone());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stringtable_basic() {
        let yaml = r#"
MonoBehaviour:
  m_TableEntries:
  - Key: "menu.title"
    m_Localized: "Start Game"
  - Key: "menu.quit"
    m_Localized: "Quit"
"#;
        let rows = parse("table.asset", yaml.as_bytes()).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().any(|(k, v)| k == "menu.title" && v == "Start Game"));
    }

    #[test]
    fn test_parse_stringtable_empty_is_ok() {
        let yaml = r#"
MonoBehaviour:
  m_TableEntries: []
"#;
        let rows = parse("empty.asset", yaml.as_bytes()).unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn test_parse_stringtable_bad_yaml_errors() {
        let bad = b"\x00\x01\x02 not yaml at all \xff";
        // Should not panic; may return empty or error.
        let _ = parse("bad.asset", bad);
    }
}
