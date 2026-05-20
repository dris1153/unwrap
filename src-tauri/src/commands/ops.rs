use std::sync::Arc;

use tauri::State;

use crate::{domain::error::AppError, sidecar::SidecarManager};

/// Cancel a running operation by `operation_id`.
/// Delegates to `SidecarManager::kill` which sends `taskkill /T /F` on Windows.
#[tauri::command]
pub async fn cancel(
    operation_id: String,
    sidecar: State<'_, Arc<SidecarManager>>,
) -> Result<(), AppError> {
    sidecar.kill(&operation_id)
}
