use std::sync::Arc;

use tauri::State;

use crate::{
    domain::{error::AppError, project::ProjectHandle, tree::NodeId},
    handlers::FormatHandlerRegistry,
};

/// Export a single asset node to `dest` on disk.
#[tauri::command]
pub async fn export(
    handle: ProjectHandle,
    node_id: String,
    dest: String,
    registry: State<'_, Arc<FormatHandlerRegistry>>,
) -> Result<(), AppError> {
    let handler = registry
        .handlers()
        .iter()
        .find(|h| h.id() == handle.format_id)
        .ok_or_else(|| {
            AppError::DetectFailed(format!("handler '{}' not registered", handle.format_id))
        })?
        .clone();

    handler
        .export(&handle, &NodeId(node_id), std::path::Path::new(&dest))
        .await
}
