use std::sync::Arc;

use directories::ProjectDirs;
use tokio::sync::Mutex;
use tracing::info;

mod commands;
mod db;
mod detection;
pub mod domain;
mod events;
pub mod handlers;
mod sidecar;

use handlers::{unity::UnityHandler, FormatHandlerRegistry};
use sidecar::SidecarManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing (structured logging) before anything else.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "unwrap=debug,warn".into()),
        )
        .init();

    // Resolve the app data directory: %LOCALAPPDATA%\Unwrap\ on Windows.
    let app_dirs = ProjectDirs::from("com", "Unwrap", "Unwrap")
        .expect("cannot resolve application data directory");
    let data_dir = app_dirs.data_local_dir().to_path_buf();
    std::fs::create_dir_all(&data_dir).expect("cannot create app data directory");

    let db_path = data_dir.join("cache.db");
    info!(path = %db_path.display(), "opening SQLite database");

    // Connect and migrate the SQLite database.
    let conn = db::connect_and_migrate(&db_path).expect("failed to initialize SQLite database");
    let db_state: Arc<Mutex<rusqlite::Connection>> = Arc::new(Mutex::new(conn));

    // Sidecar process manager — shared across all commands.
    let sidecar: Arc<SidecarManager> = Arc::new(SidecarManager::new());

    // Format handler registry — register UnityHandler.
    let mut registry = FormatHandlerRegistry::new();
    registry.register(Arc::new(UnityHandler::new(db_state.clone())));
    let registry: Arc<FormatHandlerRegistry> = Arc::new(registry);

    let sidecar_clone = sidecar.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(db_state)
        .manage(sidecar)
        .manage(registry)
        .invoke_handler(tauri::generate_handler![
            commands::project::detect_project,
            commands::project::open_project,
            commands::project::get_project,
            commands::project::list_recents,
            commands::tree::tree,
            commands::preview::preview,
            commands::export::export,
            commands::ops::cancel,
            commands::read::read_file_chunk,
            commands::sidecar::ensure_il2cpp_installed,
            commands::decompile::decompile,
            commands::translate::list_translatable,
            commands::translate::save_translation,
            commands::translate::list_locales,
            commands::translate::export_translations,
            commands::search::search,
            commands::search::index_assets,
            commands::search::index_code,
            commands::search::index_strings,
        ])
        .setup(|app| {
            // Startup: verify bundled sidecar checksums (warn-only, non-fatal v1).
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Resolve the binaries directory next to the executable.
                let bin_dir = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|d| d.join("binaries")))
                    .unwrap_or_else(|| std::path::PathBuf::from("binaries"));
                sidecar::manifest::verify_bundled(&bin_dir).await;
                tracing::debug!(app = ?app_handle.package_info().name, "startup checksum verification complete");
            });
            Ok(())
        })
        .on_window_event(move |_window, event| {
            // Clean up any running sidecar processes on app exit.
            if let tauri::WindowEvent::Destroyed = event {
                sidecar_clone.cleanup_all();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
