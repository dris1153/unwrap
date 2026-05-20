pub mod unity;

use std::{path::Path, sync::Arc};

use async_trait::async_trait;

use crate::domain::{
    detection::DetectionResult,
    error::AppError,
    preview::PreviewPayload,
    project::ProjectHandle,
    tree::{AssetTree, NodeId},
};
use crate::sidecar::SidecarManager;

/// Context passed into `FormatHandler::open()` so handlers can emit progress
/// events and spawn sidecars without needing global state.
pub struct HandlerCtx {
    pub app: tauri::AppHandle,
    pub operation_id: String,
    pub cache_dir: std::path::PathBuf,
    pub sidecar: Arc<SidecarManager>,
}

/// Core trait every format handler must implement.
/// Boxed behind `Arc<dyn FormatHandler>` in the registry.
#[async_trait]
pub trait FormatHandler: Send + Sync {
    /// Unique machine-readable identifier (e.g. "unity").
    fn id(&self) -> &str;
    /// Human-readable label shown in the chooser UI.
    fn display_name(&self) -> &str;

    /// Return a confidence score for the given path without opening it.
    async fn detect(&self, path: &Path) -> DetectionResult;

    /// Open the project at `path`, returning a lightweight handle.
    async fn open(&self, path: &Path, ctx: HandlerCtx) -> Result<ProjectHandle, AppError>;

    /// Build the full asset tree from an open handle.
    async fn tree(&self, handle: &ProjectHandle) -> Result<AssetTree, AppError>;

    /// Return a preview payload for a single node.
    async fn preview(
        &self,
        handle: &ProjectHandle,
        node: &NodeId,
    ) -> Result<PreviewPayload, AppError>;

    /// Export a single node to `dest`.
    async fn export(
        &self,
        handle: &ProjectHandle,
        node: &NodeId,
        dest: &Path,
    ) -> Result<(), AppError>;
}

/// Central registry of all registered format handlers.
/// Handlers are registered at startup (currently empty until phase-05).
pub struct FormatHandlerRegistry {
    handlers: Vec<Arc<dyn FormatHandler>>,
}

impl FormatHandlerRegistry {
    pub fn new() -> Self {
        FormatHandlerRegistry {
            handlers: Vec::new(),
        }
    }

    /// Register a new handler. Call during app initialization.
    pub fn register(&mut self, handler: Arc<dyn FormatHandler>) {
        self.handlers.push(handler);
    }

    /// Read-only slice of all registered handlers.
    pub fn handlers(&self) -> &[Arc<dyn FormatHandler>] {
        &self.handlers
    }
}

impl Default for FormatHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
