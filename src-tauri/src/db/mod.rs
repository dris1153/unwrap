use std::path::Path;

use rusqlite::Connection;

use crate::domain::error::AppError;

pub mod repo;
pub mod schema;

/// Open (or create) the SQLite database at `db_path` and run migrations.
///
/// WAL mode is enabled for better concurrent-read performance.
/// Migration strategy for v0.1: drop-and-recreate if schema version < expected.
pub fn connect_and_migrate(db_path: &Path) -> Result<Connection, AppError> {
    let conn = Connection::open(db_path)?;

    // Enable WAL for better read concurrency.
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    // Enable foreign-key enforcement.
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    migrate(&conn)?;

    Ok(conn)
}

/// Apply schema migrations.
///
/// v0→v1: drop-and-recreate (early dev, no data to preserve).
/// v1→v2: incremental (ALTER TABLE + new tables) — preserves existing data.
/// v2→v3: add FTS5 search_index + search_index_status tables.
fn migrate(conn: &Connection) -> Result<(), AppError> {
    let current: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    if current == 0 {
        // Fresh install — apply full v1 DDL then upgrade incrementally.
        conn.execute_batch(schema::DDL_V1)?;
        conn.execute_batch(schema::DDL_V1_TO_V2)?;
        conn.execute_batch(schema::DDL_V2_TO_V3)?;
        conn.pragma_update(None, "user_version", schema::SCHEMA_VERSION)?;
    } else if current == 1 {
        // Existing v1 DB — apply v2 then v3.
        conn.execute_batch(schema::DDL_V1_TO_V2)?;
        conn.execute_batch(schema::DDL_V2_TO_V3)?;
        conn.pragma_update(None, "user_version", schema::SCHEMA_VERSION)?;
    } else if current == 2 {
        // Existing v2 DB — apply v3 only.
        conn.execute_batch(schema::DDL_V2_TO_V3)?;
        conn.pragma_update(None, "user_version", schema::SCHEMA_VERSION)?;
    }
    // current == SCHEMA_VERSION (3): nothing to do.

    Ok(())
}
