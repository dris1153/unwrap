// commands/sidecar.rs — IPC commands for sidecar management.
// Exposed to the frontend via tauri::generate_handler!.
#![allow(dead_code)]

use std::sync::Arc;

use tauri::State;

use crate::domain::error::AppError;
use crate::handlers::HandlerCtx;
use crate::sidecar::{installer::ensure_installed, SidecarManager};

// ---------------------------------------------------------------------------
// State aliases
// ---------------------------------------------------------------------------

type DbState<'a> = State<'a, Arc<tokio::sync::Mutex<rusqlite::Connection>>>;

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Ensure Il2CppDumper is downloaded and installed.
///
/// The UI calls this when the first IL2CPP game is opened, presenting a
/// download progress modal. Emits `progress` events during download.
///
/// Returns `Ok(())` on success. Returns an error string if the download
/// or checksum verification fails.
#[tauri::command]
pub async fn ensure_il2cpp_installed(
    app: tauri::AppHandle,
    sidecar: State<'_, Arc<SidecarManager>>,
) -> Result<(), AppError> {
    // Build a temporary HandlerCtx for the installer.
    let app_dirs = directories::ProjectDirs::from("com", "Unwrap", "Unwrap")
        .ok_or_else(|| AppError::Io("cannot resolve app data directory".into()))?;

    let cache_dir = app_dirs.data_local_dir().to_path_buf();
    tokio::fs::create_dir_all(&cache_dir)
        .await
        .map_err(|e| AppError::Io(format!("create cache dir: {e}")))?;

    let ctx = HandlerCtx {
        app: app.clone(),
        operation_id: format!("install-il2cpp-{}", uuid_v4()),
        cache_dir,
        sidecar: sidecar.inner().clone(),
    };

    ensure_installed("il2cpp-dumper", &ctx).await?;
    Ok(())
}

/// Generate a simple pseudo-UUID v4 (no external dependency needed).
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:016x}-{:08x}", ts, (ts >> 32) as u32 ^ (ts as u32))
}
