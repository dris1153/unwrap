use std::io::SeekFrom;
use std::path::PathBuf;

use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::domain::error::AppError;

/// Maximum bytes returnable per call — 1 MiB.
const MAX_LEN: u32 = 1024 * 1024;

/// Read a raw slice of bytes from an arbitrary file path.
/// Used by the frontend HexPreview for chunked lazy loading.
///
/// * `offset` — byte offset from start of file
/// * `len`    — number of bytes to read; clamped to MAX_LEN server-side
///
/// Returns the bytes as a `Vec<u8>` (Tauri 2 serialises this as a JSON number array).
#[tauri::command]
pub async fn read_file_chunk(
    path: String,
    offset: u64,
    len: u32,
) -> Result<Vec<u8>, AppError> {
    let len = len.min(MAX_LEN);
    let p = PathBuf::from(&path);

    let canonical = tokio::fs::canonicalize(&p)
        .await
        .map_err(|e| AppError::InvalidPath(format!("cannot canonicalize '{}': {}", path, e)))?;

    let mut file = tokio::fs::File::open(&canonical)
        .await
        .map_err(|e| AppError::Io(e.to_string()))?;

    file.seek(SeekFrom::Start(offset))
        .await
        .map_err(|e| AppError::Io(e.to_string()))?;

    let mut buf = vec![0u8; len as usize];
    let n = file
        .read(&mut buf)
        .await
        .map_err(|e| AppError::Io(e.to_string()))?;

    buf.truncate(n);
    Ok(buf)
}
