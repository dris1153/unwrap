// decompile_dispatch.rs — route a script AssetNode to a DecompilePayload.
//
// Dispatch rules:
//   .cs  → read file directly (source = "raw", confidence = 1.0)
//   .dll → ILSpy decompile (source = "ilspy", confidence per backend)
//   else → Err(UnsupportedDecompileTarget)
//
// Cache layout: <cache_dir>/<project_id>/decompiled/<sha256(dll_bytes, first 64KB)>/<basename>.cs

use std::path::{Path, PathBuf};

use regex::Regex;
use sha2::{Digest, Sha256};
use tracing::debug;

use crate::{
    domain::{
        error::AppError,
        preview::{CodeRef, DecompilePayload},
        project::ScriptingBackend,
        tree::AssetNode,
    },
    handlers::HandlerCtx,
    sidecar::tools::ilspy,
};

/// ILSpy version injected into the header comment.
const ILSPY_VERSION: &str = "9.1.0.7988";

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Decompile `node` and return a `DecompilePayload`.
///
/// Called by `commands/decompile.rs` after validating the handle/node.
pub async fn decompile(
    node: &AssetNode,
    backend: ScriptingBackend,
    ctx: &HandlerCtx,
    project_id: &str,
) -> Result<DecompilePayload, AppError> {
    let ext = Path::new(&node.source_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "cs" => decompile_raw_cs(node).await,
        "dll" => decompile_via_ilspy(node, backend, ctx, project_id).await,
        _ => Err(AppError::UnsupportedDecompileTarget),
    }
}

// ---------------------------------------------------------------------------
// Raw .cs path
// ---------------------------------------------------------------------------

async fn decompile_raw_cs(node: &AssetNode) -> Result<DecompilePayload, AppError> {
    let started = std::time::Instant::now();
    let content = tokio::fs::read_to_string(&node.source_path)
        .await
        .map_err(|e| AppError::Io(format!("read cs: {e}")))?;

    let references = parse_using_statements(&content);
    let class_fullname = detect_main_class(&content);

    Ok(DecompilePayload {
        source: "raw".into(),
        backend: ScriptingBackend::Unknown,
        confidence: 1.0,
        language: "csharp".into(),
        content: content.clone(),
        class_fullname,
        assembly: Some(node.name.clone()),
        references,
        elapsed_ms: started.elapsed().as_millis() as u32,
    })
}

// ---------------------------------------------------------------------------
// ILSpy path
// ---------------------------------------------------------------------------

async fn decompile_via_ilspy(
    node: &AssetNode,
    backend: ScriptingBackend,
    ctx: &HandlerCtx,
    project_id: &str,
) -> Result<DecompilePayload, AppError> {
    let dll_path = Path::new(&node.source_path);
    let cache_path = compute_decompile_cache_path(dll_path, ctx, project_id).await?;

    // Cache hit: read the previously decompiled file.
    if cache_path.exists() {
        debug!(cache = %cache_path.display(), "decompile cache hit");
        return read_cached(&cache_path, backend, node).await;
    }

    let started = std::time::Instant::now();

    // ILSpy writes one or more .cs files into the output directory.
    // We pass the cache_path's parent as the output dir, then find the file after.
    let out_dir = cache_path
        .parent()
        .ok_or_else(|| AppError::Io("cache path has no parent".into()))?;

    ilspy::decompile(dll_path, out_dir, ctx).await?;

    // ILSpy may write <AssemblyName>.cs or nested files; find any .cs in out_dir.
    let cs_file = find_cs_file(out_dir).await?;

    let raw = tokio::fs::read_to_string(&cs_file)
        .await
        .map_err(|e| AppError::Io(format!("read ilspy output: {e}")))?;

    // Copy to expected cache_path name if ILSpy wrote a different filename.
    if cs_file != cache_path {
        tokio::fs::copy(&cs_file, &cache_path)
            .await
            .map_err(|e| AppError::Io(format!("copy to cache: {e}")))?;
    }

    let hash = short_hash(dll_path);
    let header = format!(
        "// Decompiled with ILSpy {} · {} · build #{}\n",
        ILSPY_VERSION,
        backend,
        hash
    );
    let content = format!("{header}{raw}");

    let confidence = match backend {
        ScriptingBackend::Mono => 0.95,
        ScriptingBackend::Il2Cpp => 0.70,
        ScriptingBackend::Unknown => 0.50,
    };

    Ok(DecompilePayload {
        source: "ilspy".into(),
        backend,
        confidence,
        language: "csharp".into(),
        class_fullname: detect_main_class(&content),
        assembly: Some(node.name.clone()),
        references: parse_using_statements(&content),
        content,
        elapsed_ms: started.elapsed().as_millis() as u32,
    })
}

// ---------------------------------------------------------------------------
// Cache path computation
// ---------------------------------------------------------------------------

/// Returns `<cache_dir>/<project_id>/decompiled/<sha256(first64k of dll)>/<basename>.cs`
async fn compute_decompile_cache_path(
    dll: &Path,
    ctx: &HandlerCtx,
    project_id: &str,
) -> Result<PathBuf, AppError> {
    let dll_owned = dll.to_path_buf();
    let hash = tokio::task::spawn_blocking(move || dll_content_hash(&dll_owned))
        .await
        .map_err(|e| AppError::Io(format!("spawn_blocking hash: {e}")))?;

    let basename = dll
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("assembly");

    let dir = ctx
        .cache_dir
        .join(project_id)
        .join("decompiled")
        .join(&hash);

    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| AppError::Io(format!("create decompile cache dir: {e}")))?;

    Ok(dir.join(format!("{basename}.cs")))
}

/// SHA-256 of first 64 KiB of `dll`; hex-encoded, first 16 chars.
fn dll_content_hash(dll: &Path) -> String {
    use std::io::Read;
    let mut hasher = Sha256::new();
    if let Ok(mut f) = std::fs::File::open(dll) {
        let mut buf = vec![0u8; 64 * 1024];
        if let Ok(n) = f.read(&mut buf) {
            hasher.update(&buf[..n]);
        }
    }
    let hex = hex::encode(hasher.finalize());
    hex[..16].to_string()
}

/// Short 7-char hash of the DLL path (used in header comment `build #XXXXXXX`).
fn short_hash(dll: &Path) -> String {
    let bytes = dll.to_string_lossy().as_bytes().to_vec();
    let hex = hex::encode(Sha256::digest(&bytes));
    hex[..7].to_string()
}

// ---------------------------------------------------------------------------
// Cache read helper
// ---------------------------------------------------------------------------

async fn read_cached(
    cache_path: &Path,
    backend: ScriptingBackend,
    node: &AssetNode,
) -> Result<DecompilePayload, AppError> {
    let started = std::time::Instant::now();
    let content = tokio::fs::read_to_string(cache_path)
        .await
        .map_err(|e| AppError::Io(format!("read cache: {e}")))?;

    let confidence = match backend {
        ScriptingBackend::Mono => 0.95,
        ScriptingBackend::Il2Cpp => 0.70,
        ScriptingBackend::Unknown => 0.50,
    };

    Ok(DecompilePayload {
        source: "ilspy".into(),
        backend,
        confidence,
        language: "csharp".into(),
        class_fullname: detect_main_class(&content),
        assembly: Some(node.name.clone()),
        references: parse_using_statements(&content),
        content,
        elapsed_ms: started.elapsed().as_millis() as u32,
    })
}

// ---------------------------------------------------------------------------
// Content analysis helpers
// ---------------------------------------------------------------------------

/// Best-effort: find first `public class <Name>` in the content.
/// Returns the simple class name (not fully qualified).
pub fn detect_main_class(content: &str) -> Option<String> {
    // Match: optional whitespace, `public class`, whitespace, identifier
    let re = Regex::new(r"(?m)^\s*public\s+class\s+(\w+)").ok()?;
    re.captures(content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

/// Parse `using Foo.Bar;` statements into `CodeRef` list.
pub fn parse_using_statements(content: &str) -> Vec<CodeRef> {
    let re = match Regex::new(r"(?m)^using\s+([\w\.]+);") {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    re.captures_iter(content)
        .filter_map(|c| c.get(1))
        .map(|m| CodeRef {
            symbol: m.as_str().to_string(),
            file: None,
            line: None,
        })
        .collect()
}

/// Find the first `.cs` file in `dir` (non-recursive for flat ILSpy output).
async fn find_cs_file(dir: &Path) -> Result<PathBuf, AppError> {
    let mut entries = tokio::fs::read_dir(dir)
        .await
        .map_err(|e| AppError::Io(format!("read output dir: {e}")))?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| AppError::Io(format!("read dir entry: {e}")))?
    {
        let p = entry.path();
        if p.extension()
            .map(|e| e.eq_ignore_ascii_case("cs"))
            .unwrap_or(false)
        {
            return Ok(p);
        }
    }

    Err(AppError::Io(
        "ILSpy produced no .cs file in output directory".into(),
    ))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_main_class_basic() {
        let src = "using System;\npublic class PlayerController : MonoBehaviour {\n}";
        assert_eq!(detect_main_class(src), Some("PlayerController".into()));
    }

    #[test]
    fn detect_main_class_none() {
        let src = "using System;\n// no class here";
        assert_eq!(detect_main_class(src), None);
    }

    #[test]
    fn parse_using_statements_basic() {
        let src = "using System;\nusing UnityEngine;\nusing System.Collections.Generic;\n";
        let refs = parse_using_statements(src);
        assert_eq!(refs.len(), 3);
        assert_eq!(refs[0].symbol, "System");
        assert_eq!(refs[1].symbol, "UnityEngine");
        assert_eq!(refs[2].symbol, "System.Collections.Generic");
    }

    #[test]
    fn parse_using_statements_empty() {
        let refs = parse_using_statements("// no usings");
        assert!(refs.is_empty());
    }

    #[test]
    fn short_hash_length() {
        let h = short_hash(Path::new("/some/assembly.dll"));
        assert_eq!(h.len(), 7);
    }
}
