// asset_ripper.rs — HTTP client driver for AssetRipper.GUI.Web 1.3.14.
//
// AssetRipper (GPL-3.0) is invoked as a subprocess only — never linked.
// License notice: see src-tauri/NOTICE.txt.
//
// Architecture
// ------------
// Bundled binary is `AssetRipper.GUI.Web` — an ASP.NET web app, NOT a CLI
// extractor. We spawn it in headless mode on a random local port and drive
// its REST endpoints to perform extraction.
//
// Endpoints used (per /openapi.json on the binary itself):
//   POST /LoadFolder            Path=<dir>      → 302 (blocking, sync)
//   GET  /Collections/Count                     → int (verify load success)
//   GET  /FailedFiles/View                      → HTML (informational)
//   POST /Export/UnityProject   Path=<dir>      → 302 (blocking, sync)
//   POST /Reset                                 → 200
//
// Output structure
// ----------------
// /Export/UnityProject writes to `<output>/ExportedProject/Assets/...` and
// `<output>/AuxiliaryFiles/...`. We walk the ExportedProject subfolder if
// present; otherwise fall back to the export root.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use reqwest::Client;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{ChildStdout, Command},
    time::timeout,
};
use tracing::{debug, info, warn};

use crate::domain::error::AppError;
use crate::events::{emit_progress, ProgressPayload};
use crate::handlers::HandlerCtx;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const READY_LINE: &str = "Now listening on:";
const STARTUP_TIMEOUT_SECS: u64 = 30;
const HTTP_OP_TIMEOUT_SECS: u64 = 600;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Report returned after a successful AssetRipper extraction run.
#[derive(Debug, Clone)]
pub struct ExtractReport {
    /// Resolved output directory — `<output>/ExportedProject` if it exists,
    /// else the export root. This is the path the asset tree walker should use.
    pub output_dir: PathBuf,
    /// Best-effort count of files under `output_dir`.
    pub assets_count: u32,
    /// Always None from this layer — handler reads it from `globalgamemanagers`.
    pub engine_version: Option<String>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run AssetRipper on `input` (a Unity game directory), writing extracted
/// assets to `output`.
///
/// Spawns `AssetRipper-...exe --headless --port <random>` and drives its
/// HTTP API. Progress events are emitted via `ctx.app`. Child process is
/// killed via `kill_on_drop(true)` if this future is dropped (e.g. cancel).
pub async fn extract(
    input: &Path,
    output: &Path,
    ctx: &HandlerCtx,
) -> Result<ExtractReport, AppError> {
    // ----------------------------------------------------------------------
    // 1. Setup: pick a free port, resolve log path, ensure output dir exists.
    // ----------------------------------------------------------------------
    let port = pick_free_port().await?;
    let log_path = log_path_for_op(&ctx.operation_id);
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Io(format!("create log dir: {e}")))?;
    }
    std::fs::create_dir_all(output)
        .map_err(|e| AppError::Io(format!("create output dir: {e}")))?;

    let bin = resolve_binary_path();
    info!(
        port = port,
        bin = %bin.display(),
        log = %log_path.display(),
        "spawning AssetRipper.GUI.Web"
    );

    // ----------------------------------------------------------------------
    // 2. Spawn binary, wait for "Now listening on:" readiness signal.
    // ----------------------------------------------------------------------
    emit_progress(
        &ctx.app,
        ProgressPayload {
            operation_id: ctx.operation_id.clone(),
            phase: "starting_sidecar".into(),
            percent: Some(5.0),
            message: "Starting AssetRipper…".into(),
        },
    );

    let mut child = Command::new(&bin)
        .args([
            "--headless",
            "--port",
            &port.to_string(),
            "--log",
            "--log-path",
            log_path.to_string_lossy().as_ref(),
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| AppError::SidecarFailed(format!("spawn AssetRipper: {e}")))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::SidecarFailed("AssetRipper stdout not captured".into()))?;
    wait_for_ready(stdout, Duration::from_secs(STARTUP_TIMEOUT_SECS)).await?;

    // ----------------------------------------------------------------------
    // 3. Drive HTTP API.
    // ----------------------------------------------------------------------
    let client = Client::builder()
        .timeout(Duration::from_secs(HTTP_OP_TIMEOUT_SECS))
        .build()
        .map_err(|e| AppError::SidecarFailed(format!("build http client: {e}")))?;
    let base = format!("http://127.0.0.1:{port}");

    // 3a. Reset (idempotent cleanup of any previous state).
    let _ = client.post(format!("{base}/Reset")).send().await;

    // 3b. Load folder.
    emit_progress(
        &ctx.app,
        ProgressPayload {
            operation_id: ctx.operation_id.clone(),
            phase: "loading".into(),
            percent: Some(20.0),
            message: format!("Loading {}", input.display()),
        },
    );
    post_path(&client, &format!("{base}/LoadFolder"), input)
        .await
        .map_err(|e| {
            AppError::SidecarFailed(format!(
                "LoadFolder failed: {e}; see {}",
                log_path.display()
            ))
        })?;

    // 3c. Verify load via /Collections/Count.
    emit_progress(
        &ctx.app,
        ProgressPayload {
            operation_id: ctx.operation_id.clone(),
            phase: "verifying".into(),
            percent: Some(30.0),
            message: "Verifying load…".into(),
        },
    );
    let count = get_collections_count(&client, &base).await?;
    if count == 0 {
        let _ = client.post(format!("{base}/Reset")).send().await;
        let _ = child.kill().await;
        return Err(AppError::SidecarFailed(format!(
            "AssetRipper loaded no collections from {}; see {}",
            input.display(),
            log_path.display()
        )));
    }
    debug!(count, "AssetRipper loaded collections");

    // 3d. Export (blocking — server stays on this request until done).
    emit_progress(
        &ctx.app,
        ProgressPayload {
            operation_id: ctx.operation_id.clone(),
            phase: "extracting".into(),
            percent: Some(40.0),
            message: format!("Exporting {count} collections…"),
        },
    );
    post_path(
        &client,
        &format!("{base}/Export/UnityProject"),
        output,
    )
    .await
    .map_err(|e| {
        AppError::SidecarFailed(format!(
            "Export/UnityProject failed: {e}; see {}",
            log_path.display()
        ))
    })?;

    emit_progress(
        &ctx.app,
        ProgressPayload {
            operation_id: ctx.operation_id.clone(),
            phase: "extracting".into(),
            percent: Some(90.0),
            message: "Export complete, indexing…".into(),
        },
    );

    // 3e. Cleanup server state, then kill the process.
    let _ = client.post(format!("{base}/Reset")).send().await;
    let _ = child.kill().await;

    // ----------------------------------------------------------------------
    // 4. Resolve output dir and report.
    // ----------------------------------------------------------------------
    let output_dir = resolve_output_dir(output).ok_or_else(|| {
        AppError::SidecarFailed(format!(
            "AssetRipper export produced no assets at {}; see {}",
            output.display(),
            log_path.display()
        ))
    })?;

    let assets_count = count_files_recursive(&output_dir).unwrap_or(0) as u32;

    info!(
        path = %output_dir.display(),
        count = assets_count,
        "AssetRipper extraction complete"
    );

    Ok(ExtractReport {
        output_dir,
        assets_count,
        engine_version: None,
    })
}

// ---------------------------------------------------------------------------
// Path / binary resolution
// ---------------------------------------------------------------------------

/// Resolve the AssetRipper binary path next to the running executable.
///
/// Tauri's sidecar resolution adds the target-triple suffix at build time,
/// so production bundles ship `AssetRipper-x86_64-pc-windows-msvc.exe`
/// next to the main executable. In dev runs we also place it under
/// `src-tauri/binaries/`.
fn resolve_binary_path() -> PathBuf {
    let target_name = "AssetRipper-x86_64-pc-windows-msvc.exe";

    // 1. Next to the running executable (production + tauri dev).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let cand = dir.join(target_name);
            if cand.is_file() {
                return cand;
            }
            let cand = dir.join("binaries").join(target_name);
            if cand.is_file() {
                return cand;
            }
        }
    }

    // 2. cargo run / cargo test (working dir = workspace root).
    if let Ok(cwd) = std::env::current_dir() {
        let cand = cwd.join("src-tauri").join("binaries").join(target_name);
        if cand.is_file() {
            return cand;
        }
        let cand = cwd.join("binaries").join(target_name);
        if cand.is_file() {
            return cand;
        }
    }

    // 3. Last resort — let the OS path-resolve. Will fail at spawn if missing.
    PathBuf::from(target_name)
}

fn log_path_for_op(operation_id: &str) -> PathBuf {
    directories::ProjectDirs::from("com", "Unwrap", "Unwrap")
        .map(|d| d.data_local_dir().join("logs"))
        .unwrap_or_else(|| PathBuf::from("logs"))
        .join(format!("{operation_id}-asset-ripper.log"))
}

// ---------------------------------------------------------------------------
// Output dir resolution
// ---------------------------------------------------------------------------

/// Return the directory that contains the actual extracted assets.
///
/// AssetRipper writes to `<output>/ExportedProject/` (with `Assets/` inside).
/// If that subdir exists and is non-empty, prefer it. Otherwise fall back to
/// the export root if it contains more than a couple of entries.
/// Returns `None` when nothing usable was produced.
pub(super) fn resolve_output_dir(base: &Path) -> Option<PathBuf> {
    let exported = base.join("ExportedProject");
    if exported.is_dir() && count_dir_entries(&exported) > 0 {
        return Some(exported);
    }
    if count_dir_entries(base) > 5 {
        return Some(base.to_path_buf());
    }
    None
}

fn count_dir_entries(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|rd| rd.flatten().count())
        .unwrap_or(0)
}

fn count_files_recursive(dir: &Path) -> std::io::Result<usize> {
    let mut total = 0usize;
    for entry in walkdir::WalkDir::new(dir).into_iter().flatten() {
        if entry.file_type().is_file() {
            total += 1;
        }
    }
    Ok(total)
}

// ---------------------------------------------------------------------------
// Port allocation
// ---------------------------------------------------------------------------

/// Bind to `127.0.0.1:0` to let the OS pick a free port, then drop the
/// listener so AssetRipper can reuse it. The race window between drop and
/// spawn is small but non-zero — if it loses, AssetRipper errors at startup
/// and we surface that via the readiness timeout.
async fn pick_free_port() -> Result<u16, AppError> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| AppError::Io(format!("port pick bind: {e}")))?;
    let port = listener
        .local_addr()
        .map_err(|e| AppError::Io(format!("port pick addr: {e}")))?
        .port();
    drop(listener);
    Ok(port)
}

// ---------------------------------------------------------------------------
// Readiness wait
// ---------------------------------------------------------------------------

/// Read AssetRipper stdout until it announces "Now listening on:".
/// Times out after `dur` if the binary never reaches that state.
async fn wait_for_ready(stdout: ChildStdout, dur: Duration) -> Result<(), AppError> {
    let result = timeout(dur, async {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            debug!(line = %line, "AssetRipper startup");
            if line.contains(READY_LINE) {
                return Ok::<(), AppError>(());
            }
        }
        Err(AppError::SidecarFailed(
            "AssetRipper exited before becoming ready".into(),
        ))
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) => {
            warn!("AssetRipper readiness timeout ({}s)", dur.as_secs());
            Err(AppError::Timeout)
        }
    }
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

/// POST `Path=<path>` to `url` as form-urlencoded.
///
/// AssetRipper's load/export endpoints reply with `302 Found` on success.
/// `reqwest` by default follows redirects, so a success appears as the final
/// page (200). Either status is acceptable; we only error on 4xx/5xx.
async fn post_path(client: &Client, url: &str, path: &Path) -> Result<(), reqwest::Error> {
    let lossy = path.to_string_lossy();
    let cleaned = strip_unc_prefix(&lossy);
    let body = [("Path", cleaned.as_ref())];
    let resp = client.post(url).form(&body).send().await?;
    resp.error_for_status()?;
    Ok(())
}

/// Strip the Windows UNC `\\?\` prefix from a path string. Long paths come
/// back from `canonicalize` with this prefix on Windows; passing them to
/// AssetRipper's path parser triggers ".NET file-not-found" errors.
fn strip_unc_prefix(s: &str) -> std::borrow::Cow<'_, str> {
    if let Some(rest) = s.strip_prefix(r"\\?\") {
        std::borrow::Cow::Owned(rest.to_string())
    } else {
        std::borrow::Cow::Borrowed(s)
    }
}

async fn get_collections_count(client: &Client, base: &str) -> Result<i64, AppError> {
    let resp = client
        .get(format!("{base}/Collections/Count"))
        .send()
        .await
        .map_err(|e| AppError::SidecarFailed(format!("GET Collections/Count: {e}")))?;
    let resp = resp
        .error_for_status()
        .map_err(|e| AppError::SidecarFailed(format!("Collections/Count status: {e}")))?;
    let body = resp
        .text()
        .await
        .map_err(|e| AppError::SidecarFailed(format!("read Collections/Count body: {e}")))?;
    // Body may be JSON int or quoted string per OpenAPI ("integer|string").
    let trimmed = body.trim().trim_matches('"');
    trimmed
        .parse::<i64>()
        .map_err(|e| AppError::SidecarFailed(format!("parse Collections/Count '{body}': {e}")))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pick_free_port_returns_in_ephemeral_range() {
        let port = pick_free_port().await.unwrap();
        assert!(port > 1024, "got port {port}");
    }

    #[tokio::test]
    async fn pick_free_port_two_calls_succeed() {
        // Both calls should succeed; ports may or may not differ depending on
        // OS reuse policy. We only care that both return.
        let a = pick_free_port().await.unwrap();
        let b = pick_free_port().await.unwrap();
        assert!(a > 1024 && b > 1024);
    }

    #[test]
    fn strip_unc_prefix_strips_when_present() {
        assert_eq!(
            strip_unc_prefix(r"\\?\D:\games\unity_build"),
            "D:\\games\\unity_build"
        );
    }

    #[test]
    fn strip_unc_prefix_passes_through_normal_path() {
        assert_eq!(strip_unc_prefix(r"D:\games\unity_build"), "D:\\games\\unity_build");
        assert_eq!(strip_unc_prefix("/usr/local/bin"), "/usr/local/bin");
    }

    #[test]
    fn resolve_output_dir_prefers_exported_project() {
        let dir = tempfile::TempDir::new().unwrap();
        let exported = dir.path().join("ExportedProject");
        std::fs::create_dir(&exported).unwrap();
        std::fs::write(exported.join("dummy.txt"), "x").unwrap();
        assert_eq!(resolve_output_dir(dir.path()), Some(exported));
    }

    #[test]
    fn resolve_output_dir_falls_back_to_root_when_many_entries() {
        let dir = tempfile::TempDir::new().unwrap();
        for i in 0..10 {
            std::fs::write(dir.path().join(format!("f{i}.txt")), "x").unwrap();
        }
        assert_eq!(resolve_output_dir(dir.path()), Some(dir.path().to_path_buf()));
    }

    #[test]
    fn resolve_output_dir_errors_on_empty() {
        let dir = tempfile::TempDir::new().unwrap();
        assert_eq!(resolve_output_dir(dir.path()), None);
    }

    #[test]
    fn resolve_output_dir_errors_on_empty_exported_project() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir(dir.path().join("ExportedProject")).unwrap();
        // ExportedProject exists but empty AND parent has only one child (the dir).
        assert_eq!(resolve_output_dir(dir.path()), None);
    }

    #[tokio::test]
    async fn wait_for_ready_errors_when_process_exits_silently() {
        // Spawn a process that exits immediately without printing the ready line.
        // Use `cmd /c exit 0` on Windows, `true` elsewhere.
        #[cfg(target_os = "windows")]
        let mut child = tokio::process::Command::new("cmd")
            .args(["/c", "exit", "0"])
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("spawn cmd");
        #[cfg(not(target_os = "windows"))]
        let mut child = tokio::process::Command::new("true")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("spawn true");

        let stdout = child.stdout.take().expect("stdout");
        let result = wait_for_ready(stdout, Duration::from_secs(5)).await;
        assert!(result.is_err(), "expected error, got {result:?}");
        let _ = child.wait().await;
    }
}
