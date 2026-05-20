// cache_key.rs — deterministic project identifier and source-hash utilities.

use std::path::Path;

use sha2::{Digest, Sha256};

use crate::domain::{error::AppError, project::ProjectId};

/// Compute a stable, opaque project identifier from a canonical root path.
///
/// Algorithm: SHA-256(canonical UTF-8 path bytes), hex-encoded, first 16 chars.
/// Deterministic: same absolute path always returns the same id on the same OS.
pub fn compute_project_id(path: &Path) -> ProjectId {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let bytes = canonical.to_string_lossy().as_bytes().to_vec();
    let hex = hex::encode(Sha256::digest(&bytes));
    ProjectId(hex[..16].to_string())
}

/// Compute a source-change-detection hash for a Unity project root.
///
/// Primary: SHA-256 of the first 16 KiB of `*_Data/globalgamemanagers`.
/// Fallback: SHA-256 of root directory name + metadata mtime (seconds).
///
/// The 16 KiB cap is intentional — the header bytes change when a game patches,
/// while keeping the hash cheap even on large (multi-GB) game directories.
pub async fn source_hash(path: &Path) -> Result<String, AppError> {
    // Look for *_Data/globalgamemanagers
    if let Some(ggm_path) = find_ggm(path) {
        let bytes = tokio::task::spawn_blocking(move || {
            use std::io::Read;
            let mut f = std::fs::File::open(&ggm_path)
                .map_err(|e| AppError::Io(e.to_string()))?;
            let mut buf = vec![0u8; 16 * 1024];
            let n = f.read(&mut buf).map_err(|e| AppError::Io(e.to_string()))?;
            buf.truncate(n);
            Ok::<Vec<u8>, AppError>(buf)
        })
        .await
        .map_err(|e| AppError::Io(format!("spawn_blocking: {e}")))??;

        let hex = hex::encode(Sha256::digest(&bytes));
        return Ok(hex);
    }

    // Fallback: root dir name + mtime
    let path_owned = path.to_path_buf();
    let hex = tokio::task::spawn_blocking(move || {
        let name = path_owned
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mtime = std::fs::metadata(&path_owned)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH).ok()
            })
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let input = format!("{name}:{mtime}");
        hex::encode(Sha256::digest(input.as_bytes()))
    })
    .await
    .map_err(|e| AppError::Io(format!("spawn_blocking: {e}")))?;

    Ok(hex)
}

/// Find `*_Data/globalgamemanagers` under `root`.
pub(super) fn find_ggm(root: &Path) -> Option<std::path::PathBuf> {
    let data_dir = super::detect::find_data_dir(root)?;
    let ggm = data_dir.join("globalgamemanagers");
    if ggm.is_file() { Some(ggm) } else { None }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn project_id_deterministic() {
        let dir = TempDir::new().unwrap();
        let id1 = compute_project_id(dir.path());
        let id2 = compute_project_id(dir.path());
        assert_eq!(id1, id2);
        assert_eq!(id1.0.len(), 16);
        // Must be lowercase hex
        assert!(id1.0.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn project_id_different_for_different_paths() {
        let a = TempDir::new().unwrap();
        let b = TempDir::new().unwrap();
        // Two distinct temp dirs should have different IDs
        let id_a = compute_project_id(a.path());
        let id_b = compute_project_id(b.path());
        assert_ne!(id_a, id_b);
    }

    #[tokio::test]
    async fn source_hash_fallback_no_ggm() {
        let dir = TempDir::new().unwrap();
        let h = source_hash(dir.path()).await.unwrap();
        assert_eq!(h.len(), 64); // full sha256 hex
    }
}
