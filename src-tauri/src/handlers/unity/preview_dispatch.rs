// preview_dispatch.rs — dispatch an AssetNode to a PreviewPayload.
//
// Rules per AssetKind:
//   Texture  → PreviewPayload::Image  (dimensions from image-crate header sniff, no full decode)
//   Audio    → PreviewPayload::Audio  (duration_ms from symphonia probe, fallback 0)
//   Mesh     → PreviewPayload::Model3D
//   Text     → read file ≤256KB → PreviewPayload::Text; if larger → PreviewPayload::Hex
//   Script   → PreviewPayload::Code  (raw .cs content if file exists; decompile is phase-07)
//   Folder   → PreviewPayload::Empty
//   else     → PreviewPayload::Hex
//
// All file reads are bounded to prevent accidental large allocations.

use std::path::Path;

use crate::domain::{
    error::AppError,
    preview::{ModelFormat, PreviewPayload},
    tree::{AssetKind, AssetNode},
};

/// Maximum bytes read for a Text preview.
const TEXT_LIMIT: u64 = 256 * 1024; // 256 KiB

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Build a `PreviewPayload` for `node`.
///
/// Runs synchronous I/O inside `spawn_blocking` for image/audio probes.
pub async fn dispatch(node: &AssetNode) -> Result<PreviewPayload, AppError> {
    match node.kind {
        AssetKind::Folder => Ok(PreviewPayload::Empty),

        AssetKind::Texture => {
            let path = node.source_path.clone();
            tokio::task::spawn_blocking(move || image_preview(&path))
                .await
                .map_err(|e| AppError::Io(format!("spawn_blocking image: {e}")))?
        }

        AssetKind::Audio => {
            let path = node.source_path.clone();
            tokio::task::spawn_blocking(move || audio_preview(&path))
                .await
                .map_err(|e| AppError::Io(format!("spawn_blocking audio: {e}")))?
        }

        AssetKind::Mesh => {
            let format = path_to_model_format(&node.source_path);
            Ok(PreviewPayload::Model3D {
                path: node.source_path.clone(),
                format,
            })
        }

        AssetKind::Script => {
            let path = node.source_path.clone();
            tokio::task::spawn_blocking(move || script_preview(&path))
                .await
                .map_err(|e| AppError::Io(format!("spawn_blocking script: {e}")))?
        }

        AssetKind::Text => {
            let path = node.source_path.clone();
            tokio::task::spawn_blocking(move || text_preview(&path))
                .await
                .map_err(|e| AppError::Io(format!("spawn_blocking text: {e}")))?
        }

        // All other kinds: hex view
        _ => {
            let size = std::fs::metadata(&node.source_path)
                .map(|m| m.len())
                .unwrap_or(node.size);
            Ok(PreviewPayload::Hex {
                path: node.source_path.clone(),
                size,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Per-kind helpers (all sync, called from spawn_blocking)
// ---------------------------------------------------------------------------

/// Read image dimensions via header sniff — no full decode.
fn image_preview(source_path: &str) -> Result<PreviewPayload, AppError> {
    use image::ImageReader;

    let reader = ImageReader::open(source_path)
        .map_err(|e| AppError::Io(format!("open image: {e}")))?
        .with_guessed_format()
        .map_err(|e| AppError::Io(format!("guess image format: {e}")))?;

    let (width, height) = reader
        .into_dimensions()
        .map_err(|e| AppError::Io(format!("read image dimensions: {e}")))?;

    Ok(PreviewPayload::Image {
        path: source_path.to_string(),
        width,
        height,
    })
}

/// Probe audio duration via symphonia (header only, no full decode).
fn audio_preview(source_path: &str) -> Result<PreviewPayload, AppError> {
    use symphonia::core::{
        formats::FormatOptions,
        io::MediaSourceStream,
        meta::MetadataOptions,
        probe::Hint,
    };

    let file = std::fs::File::open(source_path)
        .map_err(|e| AppError::Io(format!("open audio: {e}")))?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = Path::new(source_path).extension() {
        hint.with_extension(&ext.to_string_lossy());
    }

    let meta_opts = MetadataOptions::default();
    let fmt_opts = FormatOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .map_err(|e| AppError::Io(format!("symphonia probe: {e}")))?;

    let track = probed.format.default_track();
    let duration_ms = track
        .and_then(|t| {
            t.codec_params.time_base.zip(t.codec_params.n_frames).map(
                |(tb, frames)| {
                    let secs = frames as f64 * tb.numer as f64 / tb.denom as f64;
                    (secs * 1000.0) as u32
                },
            )
        })
        .unwrap_or(0);

    Ok(PreviewPayload::Audio {
        path: source_path.to_string(),
        duration_ms,
    })
}

/// Return raw .cs source as Code preview (decompile is phase-07).
fn script_preview(source_path: &str) -> Result<PreviewPayload, AppError> {
    let meta = std::fs::metadata(source_path)
        .map_err(|e| AppError::Io(format!("stat script: {e}")))?;

    // Only read raw text for .cs files; .dll falls back to hex
    if Path::new(source_path)
        .extension()
        .map(|e| e.eq_ignore_ascii_case("cs"))
        .unwrap_or(false)
        && meta.len() <= TEXT_LIMIT
    {
        let content = std::fs::read_to_string(source_path)
            .unwrap_or_else(|_| String::from("(binary content)"));
        return Ok(PreviewPayload::Code {
            content,
            language: "csharp".to_string(),
            references: vec![],
            source: Some("raw".to_string()),
            backend: None,
            confidence: Some(1.0),
            class_fullname: None,
            assembly: None,
            elapsed_ms: Some(0),
        });
    }

    Ok(PreviewPayload::Hex {
        path: source_path.to_string(),
        size: meta.len(),
    })
}

/// Read text content up to TEXT_LIMIT; fall back to hex for large files.
fn text_preview(source_path: &str) -> Result<PreviewPayload, AppError> {
    let meta = std::fs::metadata(source_path)
        .map_err(|e| AppError::Io(format!("stat text: {e}")))?;

    if meta.len() > TEXT_LIMIT {
        return Ok(PreviewPayload::Hex {
            path: source_path.to_string(),
            size: meta.len(),
        });
    }

    let bytes = std::fs::read(source_path)
        .map_err(|e| AppError::Io(format!("read text: {e}")))?;

    // Best-effort UTF-8; replace invalid bytes rather than failing.
    let content = String::from_utf8_lossy(&bytes).into_owned();

    let language = language_from_path(source_path);

    Ok(PreviewPayload::Text { content, language })
}

/// Guess a language hint from file extension (used for syntax highlighting).
fn language_from_path(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .and_then(|ext| match ext.as_str() {
            "json" => Some("json"),
            "xml" => Some("xml"),
            "yaml" | "yml" => Some("yaml"),
            "html" | "htm" => Some("html"),
            "csv" => Some("csv"),
            "md" => Some("markdown"),
            "txt" | "log" => None,
            _ => None,
        })
        .map(String::from)
}

/// Map a mesh file extension to a `ModelFormat`.
fn path_to_model_format(path: &str) -> ModelFormat {
    let ext = Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "glb" | "gltf" => ModelFormat::Glb,
        "fbx" => ModelFormat::Fbx,
        _ => ModelFormat::Obj,
    }
}
