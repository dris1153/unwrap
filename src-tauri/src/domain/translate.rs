// domain/translate.rs — types for the translatable text feature.

use serde::{Deserialize, Serialize};

/// A single translatable string row, sourced from an asset file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatableRow {
    /// Stable ID = hex(sha256(source_path + key)[..16]).
    pub id: String,
    pub project_id: String,
    /// Relative path from extracted root, e.g. "Assets/Localization/en/strings.json".
    pub source_path: String,
    /// Key within the file, e.g. "menu.title".
    pub key: String,
    pub comment: Option<String>,
    pub source_locale: String,
    pub source_text: String,
    /// Unix timestamp (seconds) when this row was detected.
    pub detected_at: i64,
}

/// A user-entered translation for a specific row and target locale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationEntry {
    pub row_id: String,
    pub target_locale: String,
    pub text: String,
    pub status: TranslationStatus,
    /// Unix timestamp (seconds) of last update.
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TranslationStatus {
    Pending,
    Translated,
    Review,
}

impl TranslationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TranslationStatus::Pending => "Pending",
            TranslationStatus::Translated => "Translated",
            TranslationStatus::Review => "Review",
        }
    }

    /// Parse from DB string; unknown values fall back to Pending.
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "Translated" => TranslationStatus::Translated,
            "Review" => TranslationStatus::Review,
            _ => TranslationStatus::Pending,
        }
    }
}

/// Row combined with its translation (if any) — returned by list_translatable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatableRowWithDraft {
    #[serde(flatten)]
    pub row: TranslatableRow,
    pub translation: Option<TranslationEntry>,
}

/// Locale entry returned by list_locales.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleEntry {
    pub code: String,
    pub count: u64,
    /// "source" = detected from file paths; "target" = ISO-639 fallback list.
    pub kind: LocaleKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LocaleKind {
    Source,
    Target,
}
