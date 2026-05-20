// upsert/remove used by phase-05+ handlers; suppress dead_code until then.
#![allow(dead_code)]

use rusqlite::{params, Connection};

use crate::domain::{
    error::AppError,
    project::{Project, ProjectId, RecentEntry},
};

/// Add or update a project in the recents list.
pub fn upsert(conn: &Connection, project_id: &str, pinned: bool) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO recents (project_id, pinned) VALUES (?1, ?2)
         ON CONFLICT(project_id) DO UPDATE SET pinned = excluded.pinned",
        params![project_id, pinned as i64],
    )?;
    Ok(())
}

/// Remove a project from the recents list.
pub fn remove(conn: &Connection, project_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM recents WHERE project_id = ?1",
        params![project_id],
    )?;
    Ok(())
}

/// Return all recent entries, ordered: pinned first, then by last_opened DESC.
pub fn list_all(conn: &Connection) -> Result<Vec<RecentEntry>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.root_path, p.format_id, p.engine_version, p.scripting_backend,
                p.created_at, p.last_opened, r.pinned
         FROM recents r
         JOIN projects p ON p.id = r.project_id
         ORDER BY r.pinned DESC, p.last_opened DESC",
    )?;

    let entries = stmt
        .query_map([], |row| {
            let backend_str: String = row.get(4)?;
            let project = Project {
                id: ProjectId(row.get(0)?),
                root_path: row.get(1)?,
                format_id: row.get(2)?,
                engine_version: row.get(3)?,
                scripting_backend: backend_str.parse().unwrap_or_default(),
                created_at: row.get(5)?,
                last_opened: row.get(6)?,
            };
            let pinned: i64 = row.get(7)?;
            Ok(RecentEntry {
                project,
                pinned: pinned != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(entries)
}
