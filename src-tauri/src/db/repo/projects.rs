// Functions used by phase-05+ handlers; suppress dead_code until then.
#![allow(dead_code)]

use rusqlite::{params, Connection};

use crate::domain::{
    error::AppError,
    project::{Project, ProjectId},
};

/// Insert or update a project record.
pub fn upsert(conn: &Connection, project: &Project) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO projects (id, root_path, format_id, engine_version, scripting_backend, created_at, last_opened)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
           root_path = excluded.root_path,
           format_id = excluded.format_id,
           engine_version = excluded.engine_version,
           scripting_backend = excluded.scripting_backend,
           last_opened = excluded.last_opened",
        params![
            project.id.0,
            project.root_path,
            project.format_id,
            project.engine_version,
            project.scripting_backend.to_string(),
            project.created_at,
            project.last_opened,
        ],
    )?;
    Ok(())
}

/// Fetch a single project by id.
pub fn get(conn: &Connection, id: &str) -> Result<Option<Project>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, root_path, format_id, engine_version, scripting_backend, created_at, last_opened
         FROM projects WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_to_project(row)?))
    } else {
        Ok(None)
    }
}

/// Delete a project by id (cascades to recents + asset_trees).
pub fn delete(conn: &Connection, id: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(())
}

fn row_to_project(row: &rusqlite::Row<'_>) -> Result<Project, rusqlite::Error> {
    let backend_str: String = row.get(4)?;
    Ok(Project {
        id: ProjectId(row.get(0)?),
        root_path: row.get(1)?,
        format_id: row.get(2)?,
        engine_version: row.get(3)?,
        scripting_backend: backend_str.parse().unwrap_or_default(),
        created_at: row.get(5)?,
        last_opened: row.get(6)?,
    })
}
