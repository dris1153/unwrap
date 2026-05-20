/// SQLite schema version tracked via `PRAGMA user_version`.
pub const SCHEMA_VERSION: u32 = 3;

/// DDL for schema v1.
/// WAL mode is set at connection time so it persists across sessions.
pub const DDL_V1: &str = r#"
CREATE TABLE IF NOT EXISTS projects (
  id TEXT PRIMARY KEY,
  root_path TEXT NOT NULL,
  format_id TEXT NOT NULL,
  engine_version TEXT,
  scripting_backend TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  last_opened INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS recents (
  project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
  pinned INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS asset_trees (
  project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
  source_hash TEXT NOT NULL,
  json_blob BLOB NOT NULL,
  built_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_recents_pinned ON recents(pinned DESC);
"#;

/// Incremental DDL applied on top of v2 to reach v3.
/// Adds FTS5 search index + status tracking table.
pub const DDL_V2_TO_V3: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(
  project_id UNINDEXED,
  source,
  kind UNINDEXED,
  ref_id UNINDEXED,
  label,
  sublabel,
  tokenize = 'porter unicode61'
);

CREATE TABLE IF NOT EXISTS search_index_status (
  project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
  built_at INTEGER,
  asset_count INTEGER DEFAULT 0,
  code_count INTEGER DEFAULT 0,
  string_count INTEGER DEFAULT 0
);
"#;

/// Incremental DDL applied on top of v1 to reach v2.
/// Adds translatable_rows, translations tables, and translatable_scanned flag on projects.
pub const DDL_V1_TO_V2: &str = r#"
ALTER TABLE projects ADD COLUMN translatable_scanned INTEGER NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS translatable_rows (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  source_path TEXT NOT NULL,
  key TEXT NOT NULL,
  comment TEXT,
  source_locale TEXT NOT NULL,
  source_text TEXT NOT NULL,
  detected_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_tr_project ON translatable_rows(project_id);

CREATE TABLE IF NOT EXISTS translations (
  row_id TEXT NOT NULL REFERENCES translatable_rows(id) ON DELETE CASCADE,
  target_locale TEXT NOT NULL,
  text TEXT NOT NULL,
  status TEXT NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (row_id, target_locale)
);
"#;
