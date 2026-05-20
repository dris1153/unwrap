use std::sync::Arc;

use tauri::State;

use crate::{
    domain::{error::AppError, preview::PreviewPayload, project::ProjectHandle, tree::NodeId},
    handlers::FormatHandlerRegistry,
};

/// Return a preview payload for a single asset node.
#[tauri::command]
pub async fn preview(
    handle: ProjectHandle,
    node_id: String,
    registry: State<'_, Arc<FormatHandlerRegistry>>,
) -> Result<PreviewPayload, AppError> {
    let handler = registry
        .handlers()
        .iter()
        .find(|h| h.id() == handle.format_id)
        .ok_or_else(|| {
            AppError::DetectFailed(format!("handler '{}' not registered", handle.format_id))
        })?
        .clone();

    handler.preview(&handle, &NodeId(node_id)).await
}
