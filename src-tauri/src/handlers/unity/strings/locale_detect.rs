// handlers/unity/strings/locale_detect.rs — extract locale code from a file path or filename.

use regex::Regex;
use std::sync::OnceLock;

/// Regex matching a locale segment in a directory path, e.g. "Locales/en/", "Localization/zh-CN/".
static DIR_LOCALE_RE: OnceLock<Regex> = OnceLock::new();

/// Regex matching a locale suffix in a filename, e.g. "strings_vi.json", "items_zh-CN.csv".
static FILE_LOCALE_RE: OnceLock<Regex> = OnceLock::new();

fn dir_re() -> &'static Regex {
    DIR_LOCALE_RE.get_or_init(|| {
        // Matches: Localization/, Localisation/, Localizations/, Locales/, Locale/
        Regex::new(r"[/\\](?:Locali[sz]ations?|Locales?)[/\\]([a-z]{2,3}(?:-[A-Z]{2})?)[/\\]")
            .expect("dir locale regex")
    })
}

fn file_re() -> &'static Regex {
    FILE_LOCALE_RE.get_or_init(|| {
        Regex::new(r"_([a-z]{2,3}(?:-[A-Z]{2,3})?)\.[a-zA-Z]+$").expect("file locale regex")
    })
}

/// Extract a BCP-47-ish locale code from an asset path.
///
/// Priority:
/// 1. Directory component matching `Locales?/` or `Localization?/` + locale segment.
/// 2. Filename suffix `_<locale>.<ext>`.
/// 3. Fallback: `"en"`.
pub fn extract_locale(path: &str) -> String {
    // 1. Directory match
    if let Some(cap) = dir_re().captures(path) {
        return cap[1].to_string();
    }

    // 2. Filename suffix match — only keep if it looks like a real locale code
    if let Some(cap) = file_re().captures(path) {
        let candidate = &cap[1];
        if is_plausible_locale(candidate) {
            return candidate.to_string();
        }
    }

    // 3. Fallback
    "en".to_string()
}

/// Loose check — 2-3 letter language code, optionally with region suffix.
fn is_plausible_locale(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    let lang_ok = parts[0].len() == 2 || parts[0].len() == 3;
    let region_ok = parts.len() == 1 || (parts.len() == 2 && parts[1].len() == 2);
    lang_ok && region_ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dir_locale_en() {
        assert_eq!(
            extract_locale("Assets/Localization/en/strings.json"),
            "en"
        );
    }

    #[test]
    fn test_dir_locale_zh_cn() {
        assert_eq!(
            extract_locale("Assets/Localizations/zh-CN/ui.txt"),
            "zh-CN"
        );
    }

    #[test]
    fn test_file_suffix_vi() {
        assert_eq!(extract_locale("Assets/Text/strings_vi.json"), "vi");
    }

    #[test]
    fn test_file_suffix_fr() {
        assert_eq!(extract_locale("data/items_fr.csv"), "fr");
    }

    #[test]
    fn test_fallback_en() {
        assert_eq!(extract_locale("Assets/Text/dialogue.json"), "en");
    }

    #[test]
    fn test_dir_locale_locales_singular() {
        assert_eq!(
            extract_locale("Assets/Locales/ja/npc.txt"),
            "ja"
        );
    }
}
