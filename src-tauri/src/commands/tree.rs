use std::sync::Arc;

use tauri::State;

use crate::{
    domain::{error::AppError, project::ProjectHandle, tree::AssetTree},
    handlers::FormatHandlerRegistry,
};

/// Return the asset tree for an open project handle.
#[tauri::command]
pub async fn tree(
    handle: ProjectHandle,
    registry: State<'_, Arc<FormatHandlerRegistry>>,
) -> Result<AssetTree, AppError> {
    let handler = registry
        .handlers()
        .iter()
        .find(|h| h.id() == handle.format_id)
        .ok_or_else(|| {
            AppError::DetectFailed(format!("handler '{}' not registered", handle.format_id))
        })?
        .clone();

    handler.tree(&handle).await
}
