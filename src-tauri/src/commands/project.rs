use std::sync::Arc;

use tauri::State;
use tokio::sync::Mutex;

use crate::{
    db,
    detection,
    domain::{
        detection::DetectionResult,
        error::AppError,
        project::{Project, ProjectHandle, RecentEntry},
    },
    handlers::FormatHandlerRegistry,
    sidecar::SidecarManager,
};

/// Detect which format handler can open the project at `path`.
///
/// Runs the full magic-byte → extension → structural pipeline in parallel
/// across all registered handlers.
#[tauri::command]
pub async fn detect_project(
    path: String,
    registry: State<'_, Arc<FormatHandlerRegistry>>,
) -> Result<DetectionResult, AppError> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(AppError::InvalidPath(format!("path does not exist: {path}")));
    }
    let result = detection::run_pipeline(p, &registry).await;
    Ok(result)
}

/// Open a project — invokes the winning format handler and returns a `ProjectHandle`.
///
/// Emits `progress` events on the Tauri event bus as the operation progresses.
/// Operation id is derived from the path hash so duplicate opens are idempotent.
#[tauri::command]
pub async fn open_project(
    path: String,
    registry: State<'_, Arc<FormatHandlerRegistry>>,
    _db: State<'_, Arc<Mutex<rusqlite::Connection>>>,
    sidecar: State<'_, Arc<SidecarManager>>,
    app: tauri::AppHandle,
) -> Result<ProjectHandle, AppError> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(AppError::InvalidPath(format!("path does not exist: {path}")));
    }

    // Detect which handler to use.
    let detection = detection::run_pipeline(p, &registry).await;

    let handler_id = match &detection {
        DetectionResult::Auto { candidate } => candidate.handler_id.clone(),
        DetectionResult::Chooser { candidates } => candidates
            .first()
            .map(|c| c.handler_id.clone())
            .ok_or_else(|| AppError::DetectFailed("no handler recognised this path".into()))?,
    };

    let handler = registry
        .handlers()
        .iter()
        .find(|h| h.id() == handler_id)
        .ok_or_else(|| AppError::DetectFailed(format!("handler '{handler_id}' not registered")))?
        .clone();

    // Derive a stable operation id from the path (first 16 chars of sha256).
    let operation_id = {
        use sha2::{Digest, Sha256};
        let hex = hex::encode(Sha256::digest(path.as_bytes()));
        hex[..16].to_string()
    };

    // Resolve cache directory: %LOCALAPPDATA%\Unwrap\cache\
    let cache_dir = directories::ProjectDirs::from("com", "Unwrap", "Unwrap")
        .map(|d| d.data_local_dir().join("cache"))
        .unwrap_or_else(|| std::path::PathBuf::from("cache"));
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| AppError::Io(format!("create cache dir: {e}")))?;

    let ctx = crate::handlers::HandlerCtx {
        app,
        operation_id,
        cache_dir,
        sidecar: sidecar.inner().clone(),
    };

    handler.open(p, ctx).await
}

/// Fetch a cached project by id.
#[tauri::command]
pub async fn get_project(
    id: String,
    db: State<'_, Arc<Mutex<rusqlite::Connection>>>,
) -> Result<Project, AppError> {
    let conn = db.lock().await;
    tokio::task::block_in_place(|| db::repo::projects::get(&conn, &id))?
        .ok_or_else(|| AppError::InvalidPath(format!("project not found: {id}")))
}

/// Return all recent projects from the local SQLite cache.
#[tauri::command]
pub async fn list_recents(
    db: State<'_, Arc<Mutex<rusqlite::Connection>>>,
) -> Result<Vec<RecentEntry>, AppError> {
    let conn = db.lock().await;
    tokio::task::block_in_place(|| db::repo::recents::list_all(&conn))
}
