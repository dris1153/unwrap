// commands/decompile.rs — `decompile` Tauri command.
//
// Validates the handle + node_id, resolves the project's scripting backend from
// SQLite, then dispatches to `decompile_dispatch::decompile()`.

use std::sync::Arc;

use directories::ProjectDirs;
use tauri::State;
use tracing::error;

use crate::{
    db,
    domain::{
        error::AppError,
        preview::DecompilePayload,
        project::{ProjectHandle, ScriptingBackend},
    },
    handlers::{
        unity::decompile_dispatch,
        HandlerCtx,
    },
    sidecar::SidecarManager,
};

use tokio::sync::Mutex;

/// On-demand decompile a script asset node.
///
/// Returns a `DecompilePayload` with the C# source, backend, confidence, and
/// parsed references.  Results are cached on disk; second call is <50 ms.
#[tauri::command]
pub async fn decompile(
    handle: ProjectHandle,
    node_id: String,
    db_state: State<'_, Arc<Mutex<rusqlite::Connection>>>,
    sidecar: State<'_, Arc<SidecarManager>>,
    app: tauri::AppHandle,
) -> Result<DecompilePayload, AppError> {
    // Resolve cache directory.
    let app_dirs = ProjectDirs::from("com", "Unwrap", "Unwrap")
        .ok_or_else(|| AppError::Io("cannot resolve app data directory".into()))?;
    let cache_dir = app_dirs.data_local_dir().to_path_buf();

    // Load project from DB to get scripting backend.
    let backend = {
        let conn = db_state.lock().await;
        let proj = tokio::task::block_in_place(|| {
            db::repo::projects::get(&conn, &handle.id.0)
        })?;
        proj.map(|p| p.scripting_backend)
            .unwrap_or(ScriptingBackend::Unknown)
    };

    // Load the asset tree to resolve the node.
    let tree = {
        let conn = db_state.lock().await;
        let project_id = handle.id.0.clone();
        tokio::task::block_in_place(|| {
            let mut stmt = conn
                .prepare("SELECT json_blob FROM asset_trees WHERE project_id = ?1")
                .map_err(|e| AppError::Db(e.to_string()))?;
            let mut rows = stmt
                .query(rusqlite::params![project_id])
                .map_err(|e| AppError::Db(e.to_string()))?;
            if let Some(row) = rows.next().map_err(|e| AppError::Db(e.to_string()))? {
                let blob: Vec<u8> = row.get(0).map_err(|e| AppError::Db(e.to_string()))?;
                let tree: crate::domain::tree::AssetTree = serde_json::from_slice(&blob)
                    .map_err(|e| AppError::Db(format!("deserialize tree: {e}")))?;
                Ok::<_, AppError>(tree)
            } else {
                Err(AppError::Db(format!("no cached tree for project {project_id}")))
            }
        })?
    };

    let node = tree
        .nodes
        .get(&node_id)
        .ok_or_else(|| AppError::InvalidPath(format!("node not found: {node_id}")))?
        .clone();

    let operation_id = format!("decompile:{node_id}");

    let ctx = HandlerCtx {
        app,
        operation_id,
        cache_dir,
        sidecar: sidecar.inner().clone(),
    };

    let payload = decompile_dispatch::decompile(&node, backend, &ctx, &handle.id.0).await?;

    // Fire-and-forget: index the decompiled class into FTS5 search.
    if let Some(ref class_fullname) = payload.class_fullname {
        let pid = handle.id.0.clone();
        let class = class_fullname.clone();
        let file_path = node.source_path.clone();
        let db_clone = db_state.inner().clone();
        tokio::task::spawn(async move {
            let conn = db_clone.lock().await;
            if let Err(e) = tokio::task::block_in_place(|| {
                crate::db::repo::search::insert_code(&conn, &pid, &class, &file_path)
            }) {
                error!(project_id = %pid, class = %class, error = %e, "search index: code indexing failed");
            }
        });
    }

    Ok(payload)
}
