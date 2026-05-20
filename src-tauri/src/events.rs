// emit_progress used by phase-05+ handlers; suppress dead_code until then.
#![allow(dead_code)]

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tracing::warn;

/// Progress event payload emitted from Rust to the React frontend.
/// Frontend listens via: `listen("progress", (event) => ...)`.
#[derive(Debug, Clone, Serialize)]
pub struct ProgressPayload {
    pub operation_id: String,
    /// Phase label, e.g. "extracting" | "indexing" | "decompiling".
    pub phase: String,
    /// 0–100 percent; `None` if indeterminate.
    pub percent: Option<f32>,
    pub message: String,
}

/// Emit a progress event to all frontend windows.
///
/// Failures are logged but not propagated — a broken event channel
/// must not crash the backend operation.
pub fn emit_progress(app: &AppHandle, payload: ProgressPayload) {
    if let Err(e) = app.emit("progress", payload) {
        warn!("failed to emit progress event: {e}");
    }
}
