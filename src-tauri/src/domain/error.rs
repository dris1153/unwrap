use serde::ser::{Serialize, Serializer, SerializeStruct};
use thiserror::Error;

/// Typed application error returned from all Tauri commands.
/// Manually implements Serialize because thiserror does not auto-derive it.
/// Serialized shape: `{ "code": "NOT_IMPLEMENTED", "message": "..." }`
// Variants like DetectFailed/Cancelled/Timeout are used by phase-05+ handlers.
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum AppError {
    #[error("not implemented")]
    NotImplemented,

    #[error("I/O error: {0}")]
    Io(String),

    #[error("database error: {0}")]
    Db(String),

    #[error("detection failed: {0}")]
    DetectFailed(String),

    #[error("sidecar failed: {0}")]
    SidecarFailed(String),

    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("operation cancelled")]
    Cancelled,

    #[error("operation timed out")]
    Timeout,

    #[error("unsupported decompile target")]
    UnsupportedDecompileTarget,
}

impl AppError {
    /// Machine-readable code for the frontend.
    fn code(&self) -> &'static str {
        match self {
            AppError::NotImplemented => "NOT_IMPLEMENTED",
            AppError::Io(_) => "IO_ERROR",
            AppError::Db(_) => "DB_ERROR",
            AppError::DetectFailed(_) => "DETECT_FAILED",
            AppError::SidecarFailed(_) => "SIDECAR_FAILED",
            AppError::InvalidPath(_) => "INVALID_PATH",
            AppError::Cancelled => "CANCELLED",
            AppError::Timeout => "TIMEOUT",
            AppError::UnsupportedDecompileTarget => "UNSUPPORTED_DECOMPILE_TARGET",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("AppError", 2)?;
        s.serialize_field("code", self.code())?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Db(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Db(format!("json serialization: {e}"))
    }
}
