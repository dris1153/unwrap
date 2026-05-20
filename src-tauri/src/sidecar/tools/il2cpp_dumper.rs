// il2cpp_dumper.rs — typed wrapper for Il2CppDumper (MIT license, NOT bundled).
//
// Il2CppDumper is large (~15MB) and only needed for IL2CPP games — NOT included
// in the installer. It is lazy-downloaded to %LOCALAPPDATA%\Unwrap\bin\ on
// first IL2CPP detection, via `installer::ensure_installed`.
//
// Invocation: Il2CppDumper.exe <GameAssembly.dll> <global-metadata.dat> <output_dir>
// Output: dummy DLLs + dump.cs in <output_dir>
#![allow(dead_code)]

use std::{path::Path, path::PathBuf, time::Duration};

use tracing::debug;

use crate::domain::error::AppError;
use crate::events::{emit_progress, ProgressPayload};
use crate::handlers::HandlerCtx;
use crate::sidecar::spawn::{ProgressParser, SidecarSpec};
use crate::sidecar::installer::ensure_installed;

use super::pump_progress;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Report returned after a successful Il2CppDumper run.
#[derive(Debug, Clone)]
pub struct DumpReport {
    /// Directory containing the generated dummy DLLs.
    pub dummy_dlls_dir: PathBuf,
    /// Path to the generated dump.cs file.
    pub dump_cs_path: PathBuf,
}

// ---------------------------------------------------------------------------
// Error sentinel
// ---------------------------------------------------------------------------

/// Returned when Il2CppDumper is not installed and must be fetched first.
///
/// The UI layer should call `ensure_il2cpp_installed` IPC command and retry.
pub const ERR_MISSING_SIDECAR: &str = "il2cpp-dumper not installed";

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Dump IL2CPP metadata for `game_assembly` + `global_metadata` into `out`.
///
/// Calls `installer::ensure_installed("il2cpp-dumper", ctx)` first.
/// If the binary is not yet present and cannot be downloaded, returns
/// `Err(AppError::SidecarFailed(ERR_MISSING_SIDECAR))`.
///
/// On success, returns paths to the dummy DLL directory and dump.cs.
pub async fn dump(
    game_assembly: &Path,
    global_metadata: &Path,
    out: &Path,
    ctx: &HandlerCtx,
) -> Result<DumpReport, AppError> {
    // Validate inputs.
    if !game_assembly.exists() {
        return Err(AppError::InvalidPath(format!(
            "GameAssembly.dll not found: {}",
            game_assembly.display()
        )));
    }
    if !global_metadata.exists() {
        return Err(AppError::InvalidPath(format!(
            "global-metadata.dat not found: {}",
            global_metadata.display()
        )));
    }

    // Ensure binary is installed (downloads if missing).
    let bin_path = match ensure_installed("il2cpp-dumper", ctx).await {
        Ok(p) => p,
        Err(e) => {
            return Err(AppError::SidecarFailed(format!(
                "{}: {}",
                ERR_MISSING_SIDECAR, e
            )));
        }
    };

    tokio::fs::create_dir_all(out)
        .await
        .map_err(|e| AppError::Io(format!("create output dir: {e}")))?;

    emit_progress(
        &ctx.app,
        ProgressPayload {
            operation_id: ctx.operation_id.clone(),
            phase: "dumping".into(),
            percent: Some(0.0),
            message: "Starting Il2CppDumper…".into(),
        },
    );

    let spec = SidecarSpec {
        operation_id: ctx.operation_id.clone(),
        // Use the downloaded binary path directly (not a bundled Tauri sidecar name).
        bin: bin_path.to_string_lossy().into_owned(),
        args: vec![
            game_assembly.to_string_lossy().into_owned(),
            global_metadata.to_string_lossy().into_owned(),
            out.to_string_lossy().into_owned(),
        ],
        workdir: out.to_path_buf(),
        timeout_duration: Duration::from_secs(300),
        progress_parser: ProgressParser::JsonLines,
    };

    emit_progress(
        &ctx.app,
        ProgressPayload {
            operation_id: ctx.operation_id.clone(),
            phase: "dumping".into(),
            percent: Some(10.0),
            message: "Il2CppDumper running…".into(),
        },
    );

    let handle = ctx.sidecar.spawn(spec).await?;
    let logs = pump_progress(handle, ctx.app.clone(), ctx.operation_id.clone(), "dumping").await?;

    debug!(
        op_id = %ctx.operation_id,
        log_lines = logs.len(),
        "Il2CppDumper finished"
    );

    // Il2CppDumper writes dummy DLLs into the output dir and a dump.cs at out/dump.cs.
    let dump_cs = out.join("dump.cs");
    let dummy_dlls = out.to_path_buf();

    emit_progress(
        &ctx.app,
        ProgressPayload {
            operation_id: ctx.operation_id.clone(),
            phase: "done".into(),
            percent: Some(100.0),
            message: "IL2CPP dump complete".into(),
        },
    );

    Ok(DumpReport {
        dummy_dlls_dir: dummy_dlls,
        dump_cs_path: dump_cs,
    })
}
