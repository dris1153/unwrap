// handlers/unity/mod.rs — concrete UnityHandler implementing FormatHandler.
//
// Wires: detect → cache_key → db lookup → (il2cpp_dump) → asset_ripper → tree_builder → db persist
// All progress events are emitted via ctx.app with the shared ProgressPayload shape.

pub mod cache_key;
pub mod decompile_dispatch;
pub mod detect;
pub mod dummy_dlls;
pub mod kind_table;
pub mod preview_dispatch;
pub mod strings;
pub mod tree_builder;
pub mod version;

use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::{
    commands::search::index_assets_sync,
    db,
    domain::{
        detection::DetectionResult,
        error::AppError,
        preview::PreviewPayload,
        project::{Project, ProjectHandle, ScriptingBackend},
        tree::{AssetTree, NodeId},
    },
    events::{emit_progress, ProgressPayload},
    handlers::{FormatHandler, HandlerCtx},
    sidecar::tools::{asset_ripper, il2cpp_dumper},
};

use detect::{detect_unity, extract_hints, find_data_dir, find_game_assembly_dll};

// ---------------------------------------------------------------------------
// UnityHandler
// ---------------------------------------------------------------------------

/// Concrete handler for Unity game builds (Mono + IL2CPP).
pub struct UnityHandler {
    /// Shared SQLite connection — must be the same Arc as app state.
    db: Arc<Mutex<rusqlite::Connection>>,
}

impl UnityHandler {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        UnityHandler { db }
    }
}

// ---------------------------------------------------------------------------
// FormatHandler impl
// ---------------------------------------------------------------------------

#[async_trait]
impl FormatHandler for UnityHandler {
    fn id(&self) -> &str {
        "unity"
    }

    fn display_name(&self) -> &str {
        "Unity"
    }

    async fn detect(&self, path: &Path) -> DetectionResult {
        detect_unity(path).await
    }

    async fn open(&self, path: &Path, ctx: HandlerCtx) -> Result<ProjectHandle, AppError> {
        // Reject paths that escape after canonicalization.
        let canonical = std::fs::canonicalize(path)
            .map_err(|e| AppError::InvalidPath(format!("canonicalize: {e}")))?;

        let project_id = cache_key::compute_project_id(&canonical);

        emit_progress(
            &ctx.app,
            ProgressPayload {
                operation_id: ctx.operation_id.clone(),
                phase: "detecting".into(),
                percent: Some(5.0),
                message: "Detecting Unity build…".into(),
            },
        );

        let source_hash = cache_key::source_hash(&canonical).await?;

        // --- Cache hit path ---
        // Self-heal: a cached tree with <= 1 nodes (just the root) means a
        // previous extraction failed silently and poisoned the cache. Drop
        // the bad row and fall through to re-extract.
        {
            let conn = self.db.lock().await;
            let cached = tokio::task::block_in_place(|| {
                db::repo::asset_trees::get(&conn, &project_id.0, &source_hash)
            })?;

            if let Some(cached_tree) = cached {
                if cached_tree.nodes.len() > 1 {
                    debug!(project_id = %project_id, "cache hit — returning fast");
                    let project = tokio::task::block_in_place(|| {
                        db::repo::projects::get(&conn, &project_id.0)
                    })?;

                    if let Some(proj) = project {
                        let handle = ProjectHandle {
                            id: proj.id,
                            root_path: proj.root_path,
                            format_id: proj.format_id,
                        };
                        emit_progress(
                            &ctx.app,
                            ProgressPayload {
                                operation_id: ctx.operation_id.clone(),
                                phase: "done".into(),
                                percent: Some(100.0),
                                message: "Loaded from cache".into(),
                            },
                        );
                        return Ok(handle);
                    }
                } else {
                    tracing::warn!(
                        project_id = %project_id,
                        nodes = cached_tree.nodes.len(),
                        "cached tree empty — discarding and re-extracting"
                    );
                    let _ = tokio::task::block_in_place(|| {
                        db::repo::asset_trees::delete(&conn, &project_id.0)
                    });
                    // fall through to extraction below
                }
            }
        }

        // --- Cache miss path ---

        let detection = detect_unity(&canonical).await;
        let (backend, engine_version) = extract_hints(&detection);

        let extracted_dir = ctx.cache_dir.join(&project_id.0).join("extracted");

        // IL2CPP: run Il2CppDumper first to produce dummy DLLs.
        if backend == ScriptingBackend::Il2Cpp {
            emit_progress(
                &ctx.app,
                ProgressPayload {
                    operation_id: ctx.operation_id.clone(),
                    phase: "dumping_il2cpp".into(),
                    percent: Some(10.0),
                    message: "Dumping IL2CPP metadata…".into(),
                },
            );

            let data_dir = find_data_dir(&canonical)
                .ok_or_else(|| AppError::DetectFailed("*_Data/ not found".into()))?;

            let game_assembly = find_game_assembly_dll(&canonical)
                .ok_or_else(|| AppError::DetectFailed("GameAssembly.dll not found".into()))?;

            let global_metadata = data_dir
                .join("il2cpp_data")
                .join("Metadata")
                .join("global-metadata.dat");

            let dummy_dir = ctx.cache_dir.join(&project_id.0).join("dummy_dlls");

            il2cpp_dumper::dump(&game_assembly, &global_metadata, &dummy_dir, &ctx).await?;
        }

        // Run AssetRipper extraction.
        emit_progress(
            &ctx.app,
            ProgressPayload {
                operation_id: ctx.operation_id.clone(),
                phase: "extracting".into(),
                percent: Some(30.0),
                message: "Extracting assets…".into(),
            },
        );

        let report = asset_ripper::extract(&canonical, &extracted_dir, &ctx).await?;

        // Build in-memory asset tree.
        emit_progress(
            &ctx.app,
            ProgressPayload {
                operation_id: ctx.operation_id.clone(),
                phase: "indexing".into(),
                percent: Some(95.0),
                message: format!(
                    "Indexing {} files…",
                    report.assets_count
                ),
            },
        );

        // Walk the resolved output dir from the extract report (typically
        // `extracted_dir/ExportedProject`) rather than the export root —
        // AssetRipper writes assets into the ExportedProject subfolder.
        let tree = tree_builder::walk(&report.output_dir).await?;

        // Prefer engine version from AssetRipper report; fall back to ggm parse.
        let final_engine_version = report
            .engine_version
            .clone()
            .or(engine_version);

        // Persist project + tree.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let project = Project {
            id: project_id.clone(),
            root_path: canonical.to_string_lossy().into_owned(),
            format_id: "unity".into(),
            engine_version: final_engine_version,
            scripting_backend: backend,
            created_at: now,
            last_opened: now,
        };

        {
            let conn = self.db.lock().await;
            let nodes_len = tree.nodes.len();
            tokio::task::block_in_place(|| -> Result<(), AppError> {
                db::repo::projects::upsert(&conn, &project)?;
                // Don't poison the cache with empty trees: an extraction that
                // produced no assets (root-only tree) re-runs on next open
                // instead of being remembered forever.
                if nodes_len > 1 {
                    db::repo::asset_trees::put(&conn, &project_id.0, &source_hash, &tree)?;
                } else {
                    tracing::warn!(
                        project_id = %project_id,
                        "extraction produced empty tree — NOT caching"
                    );
                }
                db::repo::recents::upsert(&conn, &project_id.0, false)?;
                Ok(())
            })?;
        }

        // Fire-and-forget: build FTS5 search index in background after tree is persisted.
        {
            let db_clone = self.db.clone();
            let pid = project_id.0.clone();
            tokio::task::spawn(async move {
                let conn = db_clone.lock().await;
                if let Err(e) = tokio::task::block_in_place(|| index_assets_sync(&conn, &pid)) {
                    tracing::error!(project_id = %pid, error = %e, "search index: asset indexing failed");
                }
            });
        }

        emit_progress(
            &ctx.app,
            ProgressPayload {
                operation_id: ctx.operation_id.clone(),
                phase: "done".into(),
                percent: Some(100.0),
                message: "Ready".into(),
            },
        );

        info!(project_id = %project_id, "project opened");

        Ok(ProjectHandle {
            id: project.id,
            root_path: project.root_path,
            format_id: "unity".into(),
        })
    }

    async fn tree(&self, handle: &ProjectHandle) -> Result<AssetTree, AppError> {
        let conn = self.db.lock().await;
        let project_id = handle.id.0.clone();

        // Retrieve the cached tree (hash check skipped here — handle already validated at open).
        // We fetch any cached entry regardless of source_hash by querying directly.
        let tree = tokio::task::block_in_place(|| {
            let mut stmt = conn
                .prepare("SELECT source_hash, json_blob FROM asset_trees WHERE project_id = ?1")
                .map_err(|e| AppError::Db(e.to_string()))?;
            let mut rows = stmt
                .query(rusqlite::params![project_id])
                .map_err(|e| AppError::Db(e.to_string()))?;
            if let Some(row) = rows.next().map_err(|e| AppError::Db(e.to_string()))? {
                let blob: Vec<u8> = row.get(1).map_err(|e| AppError::Db(e.to_string()))?;
                let tree: AssetTree = serde_json::from_slice(&blob)
                    .map_err(|e| AppError::Db(format!("deserialize tree: {e}")))?;
                Ok::<Option<AssetTree>, AppError>(Some(tree))
            } else {
                Ok::<Option<AssetTree>, AppError>(None)
            }
        })?;

        tree.ok_or_else(|| AppError::Db(format!("no cached tree for project {project_id}")))
    }

    async fn preview(
        &self,
        handle: &ProjectHandle,
        node_id: &NodeId,
    ) -> Result<PreviewPayload, AppError> {
        let tree = self.tree(handle).await?;
        let node = tree
            .nodes
            .get(&node_id.0)
            .ok_or_else(|| AppError::InvalidPath(format!("node not found: {node_id}")))?;
        preview_dispatch::dispatch(node).await
    }

    async fn export(
        &self,
        handle: &ProjectHandle,
        node_id: &NodeId,
        dest: &Path,
    ) -> Result<(), AppError> {
        let tree = self.tree(handle).await?;
        let node = tree
            .nodes
            .get(&node_id.0)
            .ok_or_else(|| AppError::InvalidPath(format!("node not found: {node_id}")))?;

        let src = Path::new(&node.source_path);
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::Io(format!("create dest dir: {e}")))?;
        }
        tokio::fs::copy(src, dest)
            .await
            .map_err(|e| AppError::Io(format!("copy asset: {e}")))?;
        Ok(())
    }
}
