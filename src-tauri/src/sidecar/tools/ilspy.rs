// ilspy.rs — typed wrapper for the ILSpyCmd sidecar (MIT license).
//
// ILSpyCmd is invoked as: ilspycmd <assembly.dll> -o <output_dir> [--nested-directories]
//
// Progress is sparse — ILSpy emits nothing to stdout while running; it
// finishes with a single exit code. We emit 50% on spawn, 100% on done.
#![allow(dead_code)]

use std::{path::Path, time::Duration};

use tracing::debug;

use crate::domain::error::AppError;
use crate::events::{emit_progress, ProgressPayload};
use crate::handlers::HandlerCtx;
use crate::sidecar::installer::ensure_bundled_cached;
use crate::sidecar::spawn::{ProgressParser, SidecarSpec};

use super::pump_progress;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Decompile `dll` to C# source files in `output_cs` directory.
///
/// Emits 50% progress on spawn, 100% on completion. ILSpyCmd does not
/// stream progress — the bar will appear "stuck" at 50% during long runs.
pub async fn decompile(dll: &Path, output_cs: &Path, ctx: &HandlerCtx) -> Result<(), AppError> {
    // Validate inputs before spawning.
    if !dll.exists() {
        return Err(AppError::InvalidPath(format!(
            "DLL not found: {}",
            dll.display()
        )));
    }

    // Ensure output directory exists.
    tokio::fs::create_dir_all(output_cs)
        .await
        .map_err(|e| AppError::Io(format!("create output dir: {e}")))?;

    emit_progress(
        &ctx.app,
        ProgressPayload {
            operation_id: ctx.operation_id.clone(),
            phase: "decompiling".into(),
            percent: Some(0.0),
            message: format!("Starting ILSpy on {}…", dll.file_name().unwrap_or_default().to_string_lossy()),
        },
    );

    // Run ILSpyCmd from the per-user cache dir, NOT the source-tree binary.
    // Same rationale as AssetRipper (see installer::ensure_bundled_cached):
    // %PROGRAMFILES%\Unwrap\binaries is read-only in production, and writes
    // adjacent to the exe in dev trigger Tauri's file-watcher rebuild loop.
    let bin = ensure_bundled_cached("ilspycmd", ctx).await?;

    let spec = SidecarSpec {
        operation_id: ctx.operation_id.clone(),
        bin: bin.to_string_lossy().into_owned(),
        args: vec![
            dll.to_string_lossy().into_owned(),
            "-o".into(),
            output_cs.to_string_lossy().into_owned(),
            "--nested-directories".into(),
        ],
        workdir: ctx.cache_dir.clone(),
        timeout_duration: Duration::from_secs(300), // 5 min; decompile is CPU-bound
        progress_parser: ProgressParser::Stderr,    // ILSpy writes to stderr
    };

    let handle = ctx.sidecar.spawn(spec).await?;

    // Emit 50% immediately to signal "running".
    emit_progress(
        &ctx.app,
        ProgressPayload {
            operation_id: ctx.operation_id.clone(),
            phase: "decompiling".into(),
            percent: Some(50.0),
            message: "Decompiling C# sources…".into(),
        },
    );

    let logs = pump_progress(handle, ctx.app.clone(), ctx.operation_id.clone(), "decompiling").await?;

    debug!(
        op_id = %ctx.operation_id,
        log_lines = logs.len(),
        "ILSpy decompile finished"
    );

    emit_progress(
        &ctx.app,
        ProgressPayload {
            operation_id: ctx.operation_id.clone(),
            phase: "done".into(),
            percent: Some(100.0),
            message: "Decompilation complete".into(),
        },
    );

    Ok(())
}
