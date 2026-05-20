// manifest.rs — reads sidecar-manifest.json and verifies SHA-256 checksums.
// Used at startup (warn-only) and by installer before executing a downloaded binary.
#![allow(dead_code)]

use std::path::Path;

use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::io::AsyncReadExt;
use tracing::debug;

use crate::domain::error::AppError;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Top-level shape of `src-tauri/binaries/sidecar-manifest.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub tools: Vec<ToolEntry>,
}

/// One entry in the manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct ToolEntry {
    pub id: String,
    pub version: String,
    pub bundled: bool,
    pub binary: String,
    pub sha256: String,
    pub license: String,
    pub source: String,
    /// Only present for non-bundled tools.
    pub download_url: Option<String>,
    /// Path inside the zip to extract (non-bundled only).
    pub install_subpath: Option<String>,
    /// Whether NOTICE.txt is legally required for this tool.
    #[serde(default)]
    pub notice_required: bool,
    /// Invocation mode metadata (informational only).
    pub invocation_mode: Option<String>,
}

impl Manifest {
    /// Find a tool entry by id, returning a reference.
    pub fn find(&self, id: &str) -> Option<&ToolEntry> {
        self.tools.iter().find(|t| t.id == id)
    }
}

// ---------------------------------------------------------------------------
// I/O
// ---------------------------------------------------------------------------

/// Parse the bundled manifest from the binary's resource directory.
///
/// In development Tauri resolves resources relative to the cargo workspace;
/// in production the file is bundled alongside the executable.
pub fn read_manifest() -> Result<Manifest, AppError> {
    // Try paths: next to the exe first, then common dev locations.
    let candidates = manifest_search_paths();

    for path in &candidates {
        if path.exists() {
            debug!(path = %path.display(), "loading sidecar manifest");
            let raw = std::fs::read_to_string(path)
                .map_err(|e| AppError::Io(format!("manifest read: {e}")))?;
            let manifest: Manifest = serde_json::from_str(&raw)
                .map_err(|e| AppError::Io(format!("manifest parse: {e}")))?;
            return Ok(manifest);
        }
    }

    Err(AppError::Io(format!(
        "sidecar-manifest.json not found; searched: {}",
        candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )))
}

fn manifest_search_paths() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();

    // 1. Next to the running executable (production bundle + dev tauri run).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join("binaries").join("sidecar-manifest.json"));
            paths.push(dir.join("sidecar-manifest.json"));
        }
    }

    // 2. Cargo workspace root (cargo test / cargo run in development).
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(
            cwd.join("src-tauri")
                .join("binaries")
                .join("sidecar-manifest.json"),
        );
        paths.push(cwd.join("binaries").join("sidecar-manifest.json"));
    }

    paths
}

// ---------------------------------------------------------------------------
// Checksum
// ---------------------------------------------------------------------------

/// Verify a file's SHA-256 against `expected_hex` (lowercase hex string).
///
/// Streams the file asynchronously to avoid loading large binaries into RAM.
/// Returns `Ok(true)` on match, `Ok(false)` on mismatch.
pub async fn verify_sha256(path: &Path, expected_hex: &str) -> Result<bool, AppError> {
    if expected_hex.is_empty() || expected_hex == "TBD" {
        // SECURITY: refuse to verify a placeholder checksum. The previous
        // implementation returned Ok(true) which made every "TBD"-marked tool
        // bypass integrity checking entirely (code-review P1 finding).
        return Err(AppError::SidecarFailed(format!(
            "refusing to verify {} with placeholder sha256 ('{}'); set a real checksum in sidecar-manifest.json before installing",
            path.display(),
            expected_hex
        )));
    }

    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|e| AppError::Io(format!("sha256 open {}: {e}", path.display())))?;

    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .await
            .map_err(|e| AppError::Io(format!("sha256 read: {e}")))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let computed = hex::encode(hasher.finalize());
    let matches = computed.eq_ignore_ascii_case(expected_hex);
    if !matches {
        debug!(
            path = %path.display(),
            expected = %expected_hex,
            computed = %computed,
            "sha256 mismatch"
        );
    }
    Ok(matches)
}

/// Synchronous variant used in startup verification path (non-async context).
pub fn verify_sha256_bytes(data: &[u8], expected_hex: &str) -> Result<bool, AppError> {
    if expected_hex.is_empty() || expected_hex == "TBD" {
        return Err(AppError::SidecarFailed(format!(
            "refusing to verify downloaded blob with placeholder sha256 ('{}')",
            expected_hex
        )));
    }
    let mut hasher = Sha256::new();
    hasher.update(data);
    let computed = hex::encode(hasher.finalize());
    Ok(computed.eq_ignore_ascii_case(expected_hex))
}

/// Verify all bundled sidecars listed in the manifest.
///
/// Logs a `warn!` for any mismatch — non-fatal in v1.
pub async fn verify_bundled(base_dir: &Path) {
    let manifest = match read_manifest() {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!("startup: could not read sidecar manifest: {e}");
            return;
        }
    };

    for tool in manifest.tools.iter().filter(|t| t.bundled) {
        let bin_path = base_dir.join(&tool.binary);
        if !bin_path.exists() {
            tracing::warn!(
                tool = %tool.id,
                path = %bin_path.display(),
                "bundled sidecar binary is missing"
            );
            continue;
        }
        match verify_sha256(&bin_path, &tool.sha256).await {
            Ok(true) => debug!(tool = %tool.id, "checksum OK"),
            Ok(false) => tracing::warn!(
                tool = %tool.id,
                "bundled sidecar checksum mismatch — binary may be corrupt or tampered"
            ),
            Err(e) => tracing::warn!(tool = %tool.id, "checksum verify error: {e}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_MANIFEST: &str = r#"{
        "version": 1,
        "tools": [
            {
                "id": "asset-ripper",
                "version": "1.3.14",
                "bundled": true,
                "binary": "AssetRipper-x86_64-pc-windows-msvc.exe",
                "sha256": "11ec892dcd70b1b86f2e52db631e83e007f6103daa20a9616ecaa7a95baa9f21",
                "license": "GPL-3.0",
                "source": "https://github.com/AssetRipper/AssetRipper/releases/tag/1.3.14",
                "notice_required": true
            },
            {
                "id": "ilspycmd",
                "version": "9.1.0.7988",
                "bundled": true,
                "binary": "ilspycmd-x86_64-pc-windows-msvc.exe",
                "sha256": "a7051adbef4494774fa6e982c0d016b2441dccf8fe5e824bafb8b8e58a5a90d5",
                "license": "MIT",
                "source": "https://www.nuget.org/packages/ilspycmd/9.1.0.7988"
            },
            {
                "id": "il2cpp-dumper",
                "version": "6.7.42",
                "bundled": false,
                "binary": "Il2CppDumper.exe",
                "sha256": "TBD",
                "license": "MIT",
                "source": "https://github.com/Perfare/Il2CppDumper/releases/tag/v6.7.42",
                "download_url": "https://github.com/Perfare/Il2CppDumper/releases/download/v6.7.42/Il2CppDumper-net6.0-x64-v6.7.42.zip",
                "install_subpath": "Il2CppDumper.exe"
            }
        ]
    }"#;

    #[test]
    fn manifest_parses_correctly() {
        let m: Manifest = serde_json::from_str(SAMPLE_MANIFEST).unwrap();
        assert_eq!(m.version, 1);
        assert_eq!(m.tools.len(), 3);

        let ar = m.find("asset-ripper").unwrap();
        assert!(ar.bundled);
        assert_eq!(ar.license, "GPL-3.0");
        assert!(ar.notice_required);

        let il2 = m.find("il2cpp-dumper").unwrap();
        assert!(!il2.bundled);
        assert!(il2.download_url.is_some());
        assert_eq!(il2.install_subpath.as_deref(), Some("Il2CppDumper.exe"));
    }

    #[test]
    fn find_returns_none_for_unknown_id() {
        let m: Manifest = serde_json::from_str(SAMPLE_MANIFEST).unwrap();
        assert!(m.find("nonexistent").is_none());
    }

    #[test]
    fn sha256_bytes_verify_known_hash() {
        // SHA-256 of the empty byte slice
        let empty_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert!(verify_sha256_bytes(&[], empty_hash).unwrap());
    }

    #[test]
    fn sha256_bytes_mismatch_returns_false() {
        assert!(!verify_sha256_bytes(&[1, 2, 3], "deadbeef").unwrap());
    }

    #[test]
    fn sha256_bytes_placeholder_refused() {
        // Security: placeholder checksum must error (not silently pass).
        // See code-review P1 finding: previous Ok(true) allowed arbitrary code execution
        // for tools shipped with sha256 = "TBD" in the manifest.
        let result_tbd = verify_sha256_bytes(&[99, 88, 77], "TBD");
        assert!(result_tbd.is_err(), "TBD placeholder must be refused");

        let result_empty = verify_sha256_bytes(&[99, 88, 77], "");
        assert!(result_empty.is_err(), "Empty checksum must be refused");
    }
}
