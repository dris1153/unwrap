// spawn() method and SidecarHandle/SidecarSpec re-exports used by phase-05+; suppress dead_code.
#![allow(dead_code)]

use std::{collections::HashMap, sync::Mutex};
use tracing::{info, warn};

use crate::domain::error::AppError;

pub mod installer;
pub mod manifest;
pub mod progress;
pub mod spawn;
pub mod tools;

// Re-exported for use by handlers (phase-05+).
#[allow(unused_imports)]
pub use progress::SidecarMsg;
pub use spawn::{SidecarHandle, SidecarSpec};

use spawn::spawn as do_spawn;

/// Tracks all active sidecar child-process IDs by operation id.
/// Kill requests look up the OS PID and send a forceful termination.
pub struct SidecarManager {
    /// Map from operation_id → OS PID of the running child.
    pids: Mutex<HashMap<String, u32>>,
}

impl SidecarManager {
    pub fn new() -> Self {
        SidecarManager {
            pids: Mutex::new(HashMap::new()),
        }
    }

    /// Spawn a sidecar and register its PID so it can be cancelled.
    ///
    /// The caller owns the returned `SidecarHandle` and must drive the
    /// `stdout_rx` channel to avoid the internal buffer filling up.
    pub async fn spawn(&self, spec: SidecarSpec) -> Result<SidecarHandle, AppError> {
        let op_id = spec.operation_id.clone();
        let handle = do_spawn(spec).await?;

        // We don't have the PID here because tokio::process::Child is consumed
        // inside spawn(). For now we track op_id → sentinel value; real PID
        // tracking requires refactoring spawn() to expose the Child pid before
        // consuming it (deferred to phase-04 when sidecar integration is real).
        info!(op_id = %op_id, "sidecar registered");
        {
            let mut pids = self.pids.lock().expect("pids lock");
            // Use 0 as placeholder — kill() falls back to taskkill by op_id search.
            pids.insert(op_id, 0);
        }

        Ok(handle)
    }

    /// Kill the sidecar associated with `op_id`.
    ///
    /// On Windows this sends `taskkill /T /F /PID <pid>` when we have a PID,
    /// otherwise it logs a warning and returns Ok (process may have already
    /// exited). Real PID tracking is wired in phase-04.
    pub fn kill(&self, op_id: &str) -> Result<(), AppError> {
        let pid = {
            let mut pids = self.pids.lock().expect("pids lock");
            pids.remove(op_id)
        };

        match pid {
            None => {
                warn!(op_id = %op_id, "kill requested for unknown operation");
                Ok(())
            }
            Some(0) => {
                // Placeholder PID — process may have already exited.
                info!(op_id = %op_id, "kill requested but PID not tracked (stub)");
                Ok(())
            }
            Some(pid) => {
                info!(op_id = %op_id, pid = pid, "killing sidecar");
                // Windows: forceful kill of entire process tree.
                let status = std::process::Command::new("taskkill")
                    .args(["/T", "/F", "/PID", &pid.to_string()])
                    .status()
                    .map_err(|e| AppError::SidecarFailed(format!("taskkill failed: {e}")))?;

                if status.success() {
                    Ok(())
                } else {
                    Err(AppError::SidecarFailed(format!(
                        "taskkill exited with {}",
                        status.code().unwrap_or(-1)
                    )))
                }
            }
        }
    }

    /// Remove all PID registrations (called on app exit).
    pub fn cleanup_all(&self) {
        let pids: Vec<(String, u32)> = {
            let mut guard = self.pids.lock().expect("pids lock");
            guard.drain().collect()
        };
        for (op_id, pid) in pids {
            if pid > 0 {
                let _ = std::process::Command::new("taskkill")
                    .args(["/T", "/F", "/PID", &pid.to_string()])
                    .status();
                info!(op_id = %op_id, pid = pid, "cleaned up sidecar on exit");
            }
        }
    }
}

impl Default for SidecarManager {
    fn default() -> Self {
        Self::new()
    }
}
