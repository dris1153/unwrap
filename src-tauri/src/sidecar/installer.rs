// installer.rs — lazy-download + zip extraction for non-bundled sidecar tools.
// Used for Il2CppDumper which is NOT bundled due to size/necessity constraints.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use tracing::{debug, info, warn};

use crate::domain::error::AppError;
use crate::events::{emit_progress, ProgressPayload};
use crate::handlers::HandlerCtx;
use crate::sidecar::manifest::{read_manifest, verify_sha256, verify_sha256_bytes};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Ensure a bundled binary is copied to the user-writable cache dir and
/// return the cached path. Subsequent calls reuse the cache.
///
/// Why this exists: bundled sidecars normally live next to the app exe
/// (`src-tauri/binaries/` in dev, `%PROGRAMFILES%\Unwrap\binaries\` in
/// production MSI installs). Spawning them from there has two problems:
///   1. .NET apps treat `AppContext.BaseDirectory` (= exe location) as their
///      content root and write Razor cache, temp session files, etc. under
///      it. In dev that lands inside `src-tauri/binaries/temp/` which the
///      Tauri CLI file-watcher reacts to by rebuilding the application,
///      killing the running app mid-operation.
///   2. `%PROGRAMFILES%\Unwrap\` is read-only for non-admin users on
///      Windows; the spawned exe fails its first write and crashes during
///      startup. The MSI install path would be entirely broken.
///
/// Solution: copy the bundled binary once into `<cache_dir>/bin/`. Spawning
/// from there relocates the .NET BaseDirectory writes into the per-user
/// cache, out of both watched and read-only trees.
///
/// Cache-hit policy: match by file size only. We can't use the manifest sha
/// because the dev path for ILSpyCmd (`pnpm sidecars`) installs the dotnet-
/// tool exe which has a different hash than the production wrapper sha
/// pinned in `sidecar-manifest.json` — see `scripts/download-sidecars.ps1`.
pub async fn ensure_bundled_cached(
    tool_id: &str,
    ctx: &HandlerCtx,
) -> Result<PathBuf, AppError> {
    let manifest = read_manifest()?;
    let entry = manifest
        .find(tool_id)
        .ok_or_else(|| AppError::SidecarFailed(format!("tool '{tool_id}' not in manifest")))?;
    if !entry.bundled {
        return Err(AppError::SidecarFailed(format!(
            "'{tool_id}' is not bundled — use ensure_installed for lazy tools"
        )));
    }

    let source = resolve_bundled_source(&entry.binary)?;
    let target = ctx.cache_dir.join("bin").join(&entry.binary);

    if let (Ok(sm), Ok(tm)) = (std::fs::metadata(&source), std::fs::metadata(&target)) {
        if sm.len() == tm.len() {
            return Ok(target);
        }
    }

    tokio::fs::create_dir_all(target.parent().unwrap())
        .await
        .map_err(|e| AppError::Io(format!("create bin dir: {e}")))?;
    tokio::fs::copy(&source, &target)
        .await
        .map_err(|e| AppError::Io(format!("copy bundled binary: {e}")))?;

    tracing::info!(
        tool = %tool_id,
        src = %source.display(),
        dst = %target.display(),
        "cached bundled binary"
    );

    Ok(target)
}

/// Locate a bundled binary on disk. Searches the standard dev + prod paths.
fn resolve_bundled_source(binary_name: &str) -> Result<PathBuf, AppError> {
    // 1. Next to the running executable — production MSI + `tauri dev` run.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for cand in [dir.join("binaries").join(binary_name), dir.join(binary_name)] {
                if cand.is_file() {
                    return Ok(cand);
                }
            }
        }
    }
    // 2. Working directory — `cargo run` / `cargo test` from workspace root.
    if let Ok(cwd) = std::env::current_dir() {
        for cand in [
            cwd.join("src-tauri").join("binaries").join(binary_name),
            cwd.join("binaries").join(binary_name),
        ] {
            if cand.is_file() {
                return Ok(cand);
            }
        }
    }
    Err(AppError::SidecarFailed(format!(
        "bundled binary '{}' not found in any expected location",
        binary_name
    )))
}

/// Ensure `tool_id` is installed and return the path to its executable.
///
/// If the binary already exists and passes the checksum, returns immediately.
/// Otherwise downloads the zip, verifies checksum, extracts the binary.
///
/// # Errors
/// Returns `AppError::SidecarFailed` if:
/// - `tool_id` is not found in the manifest.
/// - The tool is marked `bundled: true` (use the bundled path directly).
/// - Download or extraction fails after all retries.
pub async fn ensure_installed(tool_id: &str, ctx: &HandlerCtx) -> Result<PathBuf, AppError> {
    let manifest = read_manifest()?;
    let entry = manifest.find(tool_id).ok_or_else(|| {
        AppError::SidecarFailed(format!("tool '{}' not found in manifest", tool_id))
    })?;

    if entry.bundled {
        return Err(AppError::SidecarFailed(format!(
            "tool '{}' is bundled — use the bundled binary path, not ensure_installed",
            tool_id
        )));
    }

    let download_url = entry.download_url.as_deref().ok_or_else(|| {
        AppError::SidecarFailed(format!("tool '{}' has no download_url in manifest", tool_id))
    })?;

    // Reject non-https download URLs (security: no plain HTTP downloads).
    if !download_url.starts_with("https://") {
        return Err(AppError::SidecarFailed(format!(
            "insecure download URL rejected (must be https://): {}",
            download_url
        )));
    }

    let install_subpath = entry.install_subpath.as_deref().ok_or_else(|| {
        AppError::SidecarFailed(format!(
            "tool '{}' has no install_subpath in manifest",
            tool_id
        ))
    })?;

    // Target path: %LOCALAPPDATA%\Unwrap\bin\<binary>
    let target = ctx.cache_dir.join("bin").join(&entry.binary);

    // Fast path: already installed and checksum matches.
    if target.exists() {
        match verify_sha256(&target, &entry.sha256).await {
            Ok(true) => {
                debug!(tool = %tool_id, path = %target.display(), "already installed, checksum OK");
                return Ok(target);
            }
            Ok(false) => {
                warn!(tool = %tool_id, "existing binary checksum mismatch — re-downloading");
            }
            Err(e) => {
                warn!(tool = %tool_id, "checksum verify error: {e} — re-downloading");
            }
        }
    }

    tokio::fs::create_dir_all(target.parent().unwrap())
        .await
        .map_err(|e| AppError::Io(format!("create bin dir: {e}")))?;

    // Download with retry.
    emit_progress(
        &ctx.app,
        ProgressPayload {
            operation_id: ctx.operation_id.clone(),
            phase: "downloading".into(),
            percent: Some(0.0),
            message: format!("Downloading {}…", tool_id),
        },
    );

    let zip_bytes = download_with_retry(download_url, 3, &ctx.app, &ctx.operation_id).await?;

    // Verify checksum of zip bytes before extracting.
    if !verify_sha256_bytes(&zip_bytes, &entry.sha256)? {
        return Err(AppError::SidecarFailed(format!(
            "downloaded zip for '{}' failed checksum verification — refusing to extract",
            tool_id
        )));
    }

    emit_progress(
        &ctx.app,
        ProgressPayload {
            operation_id: ctx.operation_id.clone(),
            phase: "extracting".into(),
            percent: Some(90.0),
            message: format!("Extracting {}…", tool_id),
        },
    );

    // Build the full list of entries to extract: the main binary plus any
    // companion files declared in the manifest (e.g. Il2CppDumper's config.json).
    let mut entries: Vec<String> = vec![install_subpath.to_string()];
    entries.extend(entry.extra_files.iter().cloned());

    let dest_dir = target.parent().unwrap();
    extract_zip_files(&zip_bytes, dest_dir, &entries).await?;

    // Verify every requested file is actually on disk before declaring success.
    for name in &entries {
        let expected = dest_dir.join(name);
        if !expected.exists() {
            return Err(AppError::SidecarFailed(format!(
                "extraction reported success but '{}' is not at expected path: {}",
                name,
                expected.display()
            )));
        }
    }

    emit_progress(
        &ctx.app,
        ProgressPayload {
            operation_id: ctx.operation_id.clone(),
            phase: "installed".into(),
            percent: Some(100.0),
            message: format!("{} installed successfully", tool_id),
        },
    );

    info!(tool = %tool_id, path = %target.display(), "sidecar installed");
    Ok(target)
}

// ---------------------------------------------------------------------------
// Download
// ---------------------------------------------------------------------------

/// Download `url` with up to `attempts` retries and exponential backoff.
///
/// Returns the raw bytes of the response body.
pub async fn download_with_retry(
    url: &str,
    attempts: u32,
    app: &tauri::AppHandle,
    operation_id: &str,
) -> Result<Vec<u8>, AppError> {
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| AppError::SidecarFailed(format!("build http client: {e}")))?;

    let mut last_err: Option<AppError> = None;

    for attempt in 0..attempts {
        if attempt > 0 {
            let backoff = Duration::from_secs(2u64.pow(attempt));
            warn!(url = %url, attempt = attempt + 1, "retrying download after {:?}", backoff);
            tokio::time::sleep(backoff).await;
        }

        match download_once(&client, url, app, operation_id).await {
            Ok(bytes) => return Ok(bytes),
            Err(e) => {
                warn!(url = %url, attempt = attempt + 1, "download attempt failed: {e}");
                last_err = Some(e);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| {
        AppError::SidecarFailed(format!("download failed after {} attempts", attempts))
    }))
}

async fn download_once(
    client: &reqwest::Client,
    url: &str,
    app: &tauri::AppHandle,
    operation_id: &str,
) -> Result<Vec<u8>, AppError> {
    use futures::StreamExt;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::SidecarFailed(format!("http get: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::SidecarFailed(format!(
            "http {} for {}",
            response.status(),
            url
        )));
    }

    let total_bytes = response.content_length();
    let mut downloaded: u64 = 0;
    let mut body = Vec::new();

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| AppError::SidecarFailed(format!("stream read: {e}")))?;
        downloaded += chunk.len() as u64;
        body.extend_from_slice(&chunk);

        if let Some(total) = total_bytes {
            let pct = (downloaded as f32 / total as f32) * 80.0; // 0-80%, extraction takes 80-100%
            emit_progress(
                app,
                ProgressPayload {
                    operation_id: operation_id.to_string(),
                    phase: "downloading".into(),
                    percent: Some(pct),
                    message: format!(
                        "Downloading… {:.0}/{:.0} MB",
                        downloaded as f64 / 1_048_576.0,
                        total as f64 / 1_048_576.0
                    ),
                },
            );
        }
    }

    debug!("downloaded {} bytes from {}", body.len(), url);
    Ok(body)
}

// ---------------------------------------------------------------------------
// Zip extraction
// ---------------------------------------------------------------------------

/// Extract `entry_names` from `zip_bytes` into `dest_dir`.
///
/// Opens the archive once and iterates entries, matching each entry name
/// by exact filename or path-suffix (so flat zips and nested zips both work).
/// Returns the on-disk paths of the extracted files in arbitrary order.
///
/// # Security
/// - Rejects entries with `..` or absolute paths (zip-slip prevention).
/// - Only extracts entries explicitly requested.
///
/// # Errors
/// Returns `Err(AppError::SidecarFailed)` if any requested name is not
/// present in the archive — partial extraction is treated as failure so
/// callers don't end up with half a sidecar installation.
pub async fn extract_zip_files(
    zip_bytes: &[u8],
    dest_dir: &Path,
    entry_names: &[String],
) -> Result<Vec<PathBuf>, AppError> {
    let zip_bytes = zip_bytes.to_vec();
    let dest_dir = dest_dir.to_path_buf();
    let entry_names: Vec<String> = entry_names.to_vec();

    tokio::task::spawn_blocking(move || extract_zip_files_sync(&zip_bytes, &dest_dir, &entry_names))
        .await
        .map_err(|e| AppError::SidecarFailed(format!("spawn_blocking: {e}")))?
}

/// Backward-compatible single-entry wrapper.
pub async fn extract_zip(
    zip_bytes: &[u8],
    dest_dir: &Path,
    entry_name: &str,
) -> Result<(), AppError> {
    let names = vec![entry_name.to_string()];
    extract_zip_files(zip_bytes, dest_dir, &names)
        .await
        .map(|_| ())
}

fn extract_zip_files_sync(
    zip_bytes: &[u8],
    dest_dir: &Path,
    entry_names: &[String],
) -> Result<Vec<PathBuf>, AppError> {
    use std::collections::HashSet;
    use std::io::Cursor;

    if entry_names.is_empty() {
        return Ok(Vec::new());
    }

    let cursor = Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| AppError::SidecarFailed(format!("open zip: {e}")))?;

    // Pending set drains as entries are extracted so we can short-circuit
    // once everything requested is on disk.
    let mut pending: HashSet<String> = entry_names.iter().cloned().collect();
    let mut extracted: Vec<PathBuf> = Vec::with_capacity(entry_names.len());

    for i in 0..archive.len() {
        if pending.is_empty() {
            break;
        }

        let mut file = archive
            .by_index(i)
            .map_err(|e| AppError::SidecarFailed(format!("zip entry {i}: {e}")))?;

        let raw_name = file.name().to_string();

        // Find which pending name this entry matches (basename or suffix).
        let matched_name = pending.iter().find(|name| {
            raw_name == **name
                || raw_name.ends_with(&format!("/{}", name))
                || raw_name.ends_with(&format!("\\{}", name))
        }).cloned();

        let Some(name) = matched_name else { continue };

        // Security: reject zip-slip paths.
        if raw_name.contains("..") || raw_name.starts_with('/') || raw_name.starts_with('\\') {
            return Err(AppError::SidecarFailed(format!(
                "zip entry '{}' rejected (zip-slip path)",
                raw_name
            )));
        }

        let out_path = dest_dir.join(&name);

        let mut out_file = std::fs::File::create(&out_path)
            .map_err(|e| AppError::Io(format!("create {}: {e}", out_path.display())))?;

        std::io::copy(&mut file, &mut out_file)
            .map_err(|e| AppError::Io(format!("extract {}: {e}", raw_name)))?;

        debug!(entry = %raw_name, dest = %out_path.display(), "extracted zip entry");
        pending.remove(&name);
        extracted.push(out_path);
    }

    if !pending.is_empty() {
        let missing: Vec<String> = pending.into_iter().collect();
        return Err(AppError::SidecarFailed(format!(
            "entries not found in zip: {:?} ({} entries in archive)",
            missing,
            archive.len()
        )));
    }

    Ok(extracted)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal in-memory zip with a single entry.
    fn make_test_zip(entry_name: &str, content: &[u8]) -> Vec<u8> {
        use std::io::Write;
        let mut buf = Vec::new();
        let cursor = std::io::Cursor::new(&mut buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let opts =
            zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file(entry_name, opts).unwrap();
        zip.write_all(content).unwrap();
        zip.finish().unwrap();
        buf
    }

    #[tokio::test]
    async fn extract_zip_finds_entry() {
        let zip_bytes = make_test_zip("Tool.exe", b"fake exe content");
        let dir = tempfile::tempdir().unwrap();
        extract_zip(&zip_bytes, dir.path(), "Tool.exe")
            .await
            .unwrap();
        let out = dir.path().join("Tool.exe");
        assert!(out.exists());
        assert_eq!(std::fs::read(&out).unwrap(), b"fake exe content");
    }

    #[tokio::test]
    async fn extract_zip_missing_entry_errors() {
        let zip_bytes = make_test_zip("Other.exe", b"data");
        let dir = tempfile::tempdir().unwrap();
        let result = extract_zip(&zip_bytes, dir.path(), "Tool.exe").await;
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("not found in zip"), "unexpected: {msg}");
    }

    #[tokio::test]
    async fn extract_zip_rejects_zipslip() {
        let zip_bytes = make_test_zip("../../../etc/passwd", b"bad");
        let dir = tempfile::tempdir().unwrap();
        let result = extract_zip(&zip_bytes, dir.path(), "../../../etc/passwd").await;
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("zip-slip"), "expected zip-slip error, got: {msg}");
    }

    #[test]
    fn extract_zip_sync_subdirectory_match() {
        use std::io::Write;
        // Entry stored as "subdir/Tool.exe" should match install_subpath "Tool.exe"
        let mut buf = Vec::new();
        let cursor = std::io::Cursor::new(&mut buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("subdir/Tool.exe", opts).unwrap();
        zip.write_all(b"nested content").unwrap();
        zip.finish().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let names = vec!["Tool.exe".to_string()];
        extract_zip_files_sync(&buf, dir.path(), &names).unwrap();
        let out = dir.path().join("Tool.exe");
        assert!(out.exists());
    }

    /// Build a zip with multiple distinct entries for multi-extract tests.
    fn make_multi_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        use std::io::Write;
        let mut buf = Vec::new();
        let cursor = std::io::Cursor::new(&mut buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (name, content) in entries {
            zip.start_file(*name, opts).unwrap();
            zip.write_all(content).unwrap();
        }
        zip.finish().unwrap();
        buf
    }

    #[tokio::test]
    async fn extract_zip_files_extracts_multiple_entries() {
        let zip_bytes = make_multi_zip(&[
            ("Il2CppDumper.exe", b"main binary"),
            ("config.json", b"{\"foo\": 1}"),
            ("Il2CppDumper-x86.exe", b"x86 binary"),
            ("ghidra.py", b"# helper"),
        ]);
        let dir = tempfile::tempdir().unwrap();
        let requested = vec!["Il2CppDumper.exe".to_string(), "config.json".to_string()];
        let out_paths = extract_zip_files(&zip_bytes, dir.path(), &requested)
            .await
            .expect("extract should succeed");

        assert_eq!(out_paths.len(), 2);
        assert!(dir.path().join("Il2CppDumper.exe").exists());
        assert!(dir.path().join("config.json").exists());
        // Non-requested entries must NOT be extracted.
        assert!(!dir.path().join("Il2CppDumper-x86.exe").exists());
        assert!(!dir.path().join("ghidra.py").exists());
    }

    #[tokio::test]
    async fn extract_zip_files_errors_on_missing_entry() {
        let zip_bytes = make_multi_zip(&[("a.txt", b"a"), ("b.txt", b"b")]);
        let dir = tempfile::tempdir().unwrap();
        let requested = vec!["a.txt".to_string(), "missing.txt".to_string()];
        let result = extract_zip_files(&zip_bytes, dir.path(), &requested).await;
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("entries not found") && msg.contains("missing.txt"),
            "unexpected error: {msg}"
        );
    }

    // -----------------------------------------------------------------------
    // ensure_bundled_cached behavior — tested via the inner helper so we don't
    // have to mock read_manifest() or HandlerCtx.
    // -----------------------------------------------------------------------

    /// Single helper that mirrors the cache-or-copy logic of
    /// `ensure_bundled_cached` without the manifest + ctx plumbing. Tests pass
    /// source/target paths directly so the cache decision can be observed.
    async fn cache_or_copy(source: &Path, target: &Path) -> Result<bool, AppError> {
        if let (Ok(sm), Ok(tm)) = (std::fs::metadata(source), std::fs::metadata(target)) {
            if sm.len() == tm.len() {
                return Ok(false); // cache hit, no copy
            }
        }
        tokio::fs::create_dir_all(target.parent().unwrap()).await
            .map_err(|e| AppError::Io(format!("create bin dir: {e}")))?;
        tokio::fs::copy(source, target).await
            .map_err(|e| AppError::Io(format!("copy bundled binary: {e}")))?;
        Ok(true)
    }

    #[tokio::test]
    async fn cache_or_copy_copies_on_first_call() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("bin-src.exe");
        let target = dir.path().join("cache").join("bin-cached.exe");
        std::fs::write(&source, b"binary-payload-v1").unwrap();
        assert!(!target.exists());

        let copied = cache_or_copy(&source, &target).await.unwrap();
        assert!(copied, "expected first call to copy");
        assert!(target.exists());
        assert_eq!(std::fs::read(&target).unwrap(), b"binary-payload-v1");
    }

    #[tokio::test]
    async fn cache_or_copy_reuses_when_size_matches() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("bin-src.exe");
        let target = dir.path().join("cache").join("bin-cached.exe");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        // Same byte length → cache hit decision.
        std::fs::write(&source, b"AAAAAAAA").unwrap();
        std::fs::write(&target, b"BBBBBBBB").unwrap();

        let copied = cache_or_copy(&source, &target).await.unwrap();
        assert!(!copied, "expected cache hit, no copy");
        // Target content unchanged because the function did not copy.
        assert_eq!(std::fs::read(&target).unwrap(), b"BBBBBBBB");
    }
}
