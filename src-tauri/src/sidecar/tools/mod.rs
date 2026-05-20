// tools/mod.rs — typed wrappers for each external sidecar tool.
// Re-exports the three wrappers and houses the shared `pump_progress` helper.
#![allow(dead_code)]

pub mod asset_ripper;
pub mod il2cpp_dumper;
pub mod ilspy;

use tauri::AppHandle;
use tracing::debug;

use crate::domain::error::AppError;
use crate::events::{emit_progress, ProgressPayload};
use crate::sidecar::{
    progress::SidecarMsg,
    spawn::SidecarHandle,
};

// ---------------------------------------------------------------------------
// Shared progress pump
// ---------------------------------------------------------------------------

/// Drain `handle.stdout_rx`, forwarding progress messages to the UI.
///
/// Returns `Ok(collected_log)` on successful process exit (code 0), or
/// `Err(AppError::SidecarFailed)` if the process exits non-zero or errors.
///
/// The `phase` parameter labels ALL progress events emitted by this pump
/// (individual tools wrap this with their own phase names before calling).
pub(crate) async fn pump_progress(
    mut handle: SidecarHandle,
    app: AppHandle,
    operation_id: String,
    default_phase: &str,
) -> Result<Vec<String>, AppError> {
    let mut logs = Vec::new();

    loop {
        match handle.stdout_rx.recv().await {
            None => break, // channel closed
            Some(SidecarMsg::Progress { percent, message }) => {
                debug!(op_id = %operation_id, percent = percent, "sidecar progress");
                emit_progress(
                    &app,
                    ProgressPayload {
                        operation_id: operation_id.clone(),
                        phase: default_phase.to_string(),
                        percent: Some(percent),
                        message: message.clone(),
                    },
                );
                logs.push(message);
            }
            Some(SidecarMsg::Log(line)) => {
                debug!(op_id = %operation_id, line = %line, "sidecar log");
                logs.push(line);
            }
            Some(SidecarMsg::Done) => {
                debug!(op_id = %operation_id, "sidecar done");
                break;
            }
            Some(SidecarMsg::Error(msg)) => {
                return Err(AppError::SidecarFailed(format!(
                    "sidecar error: {}",
                    msg
                )));
            }
        }
    }

    Ok(logs)
}
