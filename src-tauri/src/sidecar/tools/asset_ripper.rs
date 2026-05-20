// asset_ripper.rs — typed wrapper for AssetRipper sidecar.
//
// AssetRipper (GPL-3.0) is invoked as a subprocess only — never linked.
// License notice: see src-tauri/NOTICE.txt.
//
// Progress parsing: AssetRipper.GUI.Free does not expose a JSON-only CLI mode.
// The wrapper uses `ProgressParser::JsonLines` (optimistic) but falls back to
// regex heuristics when lines are not JSON.
//
// Known regex patterns from AssetRipper console output:
//   - Phase keywords: "Loading", "Extracting", "Writing", "Exporting", "Finished"
//   - Progress marker: "\[(\d+)%\]" or "(\d+) of (\d+)"
//   - Engine version: "Unity (\d+\.\d+\.\d+[a-z]\d*)"
//   - IL2CPP detection: "Scripting backend.*IL2CPP" (case-insensitive)
//   - Assets count: "Exported\s+(\d+)\s+asset"
#![allow(dead_code)]

use std::{path::Path, path::PathBuf, time::Duration};

use regex::Regex;
use std::sync::OnceLock;
use tracing::debug;

use crate::domain::error::AppError;
use crate::events::{emit_progress, ProgressPayload};
use crate::handlers::HandlerCtx;
use crate::sidecar::{
    spawn::{ProgressParser, SidecarSpec},
    progress::SidecarMsg,
};

#[allow(unused_imports)]
use super::pump_progress;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Scripting backend detected from AssetRipper output.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptingBackend {
    Mono,
    Il2Cpp,
    /// Could not determine from output.
    Unknown,
}

/// Report returned after a successful AssetRipper extraction run.
#[derive(Debug, Clone)]
pub struct ExtractReport {
    pub output_dir: PathBuf,
    /// Total number of assets extracted (parsed from log).
    pub assets_count: u32,
    /// Unity engine version string, e.g. "2022.3.14f1".
    pub engine_version: Option<String>,
    /// Detected scripting backend.
    pub scripting_backend: ScriptingBackend,
}

// ---------------------------------------------------------------------------
// Compiled regexes (lazy, thread-safe)
// ---------------------------------------------------------------------------

fn re_percent() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\[(\d+)%\]").unwrap())
}

fn re_of_n() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(\d+)\s+of\s+(\d+)").unwrap())
}

fn re_engine_version() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"Unity\s+(\d+\.\d+\.\d+[a-zA-Z]\d*)").unwrap())
}

fn re_assets_count() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?i)exported\s+(\d+)\s+asset").unwrap())
}

fn re_assets_count_alt() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    // Fallback: "14238 assets" at end of progress line
    R.get_or_init(|| Regex::new(r"(\d+)\s+asset").unwrap())
}

fn re_il2cpp() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?i)il2cpp|scripting.{0,20}il.?2.?cpp").unwrap())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run AssetRipper on `input` (a Unity game directory or bundle), writing
/// extracted assets to `output`.
///
/// Progress events are emitted via `ctx.app`. The process is killed if the
/// associated `SidecarManager` receives a cancel request for `ctx.operation_id`.
///
/// # Timeouts
/// Hard-coded to 10 minutes; configurable in v2.
pub async fn extract(
    input: &Path,
    output: &Path,
    ctx: &HandlerCtx,
) -> Result<ExtractReport, AppError> {
    let spec = SidecarSpec {
        operation_id: ctx.operation_id.clone(),
        bin: "AssetRipper".into(), // Tauri resolves from bundle/dev path automatically
        args: vec![
            input.to_string_lossy().into_owned(),
            "-o".into(),
            output.to_string_lossy().into_owned(),
            // AssetRipper GUI.Free does not have a --log-format flag;
            // we use JsonLines as a best-effort parser; the regex fallback
            // handles plain-text lines.
            "--headless".into(),
        ],
        workdir: ctx.cache_dir.clone(),
        timeout_duration: Duration::from_secs(600),
        progress_parser: ProgressParser::JsonLines,
    };

    emit_progress(
        &ctx.app,
        ProgressPayload {
            operation_id: ctx.operation_id.clone(),
            phase: "extracting".into(),
            percent: Some(0.0),
            message: "Starting AssetRipper…".into(),
        },
    );

    let mut handle = ctx.sidecar.spawn(spec).await?;

    let mut logs: Vec<String> = Vec::new();
    let mut engine_version: Option<String> = None;
    let mut scripting_backend = ScriptingBackend::Unknown;
    let mut assets_count: u32 = 0;

    // Drain the channel, parse each message, and relay progress.
    loop {
        match handle.stdout_rx.recv().await {
            None => break,
            Some(SidecarMsg::Progress { percent, message }) => {
                let phase = classify_phase(&message);
                emit_progress(
                    &ctx.app,
                    ProgressPayload {
                        operation_id: ctx.operation_id.clone(),
                        phase: phase.to_string(),
                        percent: Some(percent),
                        message: message.clone(),
                    },
                );
                parse_log_line(&message, &mut engine_version, &mut scripting_backend, &mut assets_count);
                logs.push(message);
            }
            Some(SidecarMsg::Log(line)) => {
                debug!(op_id = %ctx.operation_id, line = %line, "AssetRipper log");
                // Try regex-based progress extraction from plain-text line.
                let pct = extract_percent_from_line(&line);
                let phase = classify_phase(&line);
                emit_progress(
                    &ctx.app,
                    ProgressPayload {
                        operation_id: ctx.operation_id.clone(),
                        phase: phase.to_string(),
                        percent: pct,
                        message: line.clone(),
                    },
                );
                parse_log_line(&line, &mut engine_version, &mut scripting_backend, &mut assets_count);
                logs.push(line);
            }
            Some(SidecarMsg::Done) => {
                debug!(op_id = %ctx.operation_id, "AssetRipper finished");
                break;
            }
            Some(SidecarMsg::Error(msg)) => {
                return Err(AppError::SidecarFailed(format!(
                    "AssetRipper failed: {}",
                    msg
                )));
            }
        }
    }

    // Final pass over all log lines if counts still zero (dense output case).
    if assets_count == 0 {
        for line in &logs {
            if let Some(caps) = re_assets_count_alt().captures(line) {
                if let Ok(n) = caps[1].parse::<u32>() {
                    if n > assets_count {
                        assets_count = n;
                    }
                }
            }
        }
    }

    emit_progress(
        &ctx.app,
        ProgressPayload {
            operation_id: ctx.operation_id.clone(),
            phase: "done".into(),
            percent: Some(100.0),
            message: format!(
                "Extraction complete: {} assets",
                if assets_count > 0 {
                    assets_count.to_string()
                } else {
                    "?".to_string()
                }
            ),
        },
    );

    Ok(ExtractReport {
        output_dir: output.to_path_buf(),
        assets_count,
        engine_version,
        scripting_backend,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract a percent value from a plain-text log line using known patterns.
fn extract_percent_from_line(line: &str) -> Option<f32> {
    if let Some(caps) = re_percent().captures(line) {
        return caps[1].parse::<f32>().ok();
    }
    if let Some(caps) = re_of_n().captures(line) {
        let current: f32 = caps[1].parse().ok()?;
        let total: f32 = caps[2].parse().ok()?;
        if total > 0.0 {
            return Some((current / total * 100.0).min(100.0));
        }
    }
    None
}

/// Map a log line to a phase label for the UI.
fn classify_phase(line: &str) -> &'static str {
    let lower = line.to_lowercase();
    if lower.contains("load") || lower.contains("read") {
        "loading"
    } else if lower.contains("export") || lower.contains("extract") || lower.contains("convert") {
        "extracting"
    } else if lower.contains("writ") || lower.contains("sav") || lower.contains("output") {
        "writing"
    } else if lower.contains("finish") || lower.contains("done") || lower.contains("complete") {
        "done"
    } else {
        "processing"
    }
}

/// Update report accumulators by parsing a single log line.
fn parse_log_line(
    line: &str,
    engine_version: &mut Option<String>,
    scripting_backend: &mut ScriptingBackend,
    assets_count: &mut u32,
) {
    if engine_version.is_none() {
        if let Some(caps) = re_engine_version().captures(line) {
            *engine_version = Some(caps[1].to_string());
        }
    }

    if *scripting_backend == ScriptingBackend::Unknown && re_il2cpp().is_match(line) {
        *scripting_backend = ScriptingBackend::Il2Cpp;
    }

    if let Some(caps) = re_assets_count().captures(line) {
        if let Ok(n) = caps[1].parse::<u32>() {
            if n > *assets_count {
                *assets_count = n;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_phase_loading() {
        assert_eq!(classify_phase("Loading assets from disk"), "loading");
        assert_eq!(classify_phase("Reading bundle file"), "loading");
    }

    #[test]
    fn classify_phase_extracting() {
        assert_eq!(classify_phase("Extracting Texture2D"), "extracting");
        assert_eq!(classify_phase("Exporting 1284 of 3071"), "extracting");
    }

    #[test]
    fn classify_phase_writing() {
        assert_eq!(classify_phase("Writing output files"), "writing");
    }

    #[test]
    fn extract_percent_bracket() {
        assert_eq!(extract_percent_from_line("[42%] Processing"), Some(42.0));
    }

    #[test]
    fn extract_percent_of_n() {
        assert_eq!(
            extract_percent_from_line("Processing 512 of 1024"),
            Some(50.0)
        );
    }

    #[test]
    fn parse_log_engine_version() {
        let mut ev: Option<String> = None;
        let mut sb = ScriptingBackend::Unknown;
        let mut ac: u32 = 0;
        parse_log_line("Detected Unity 2022.3.14f1 project", &mut ev, &mut sb, &mut ac);
        assert_eq!(ev.as_deref(), Some("2022.3.14f1"));
    }

    #[test]
    fn parse_log_il2cpp_detection() {
        let mut ev: Option<String> = None;
        let mut sb = ScriptingBackend::Unknown;
        let mut ac: u32 = 0;
        parse_log_line("Scripting backend: IL2CPP detected", &mut ev, &mut sb, &mut ac);
        assert_eq!(sb, ScriptingBackend::Il2Cpp);
    }

    #[test]
    fn parse_log_assets_count() {
        let mut ev: Option<String> = None;
        let mut sb = ScriptingBackend::Unknown;
        let mut ac: u32 = 0;
        parse_log_line("Exported 14238 assets to output/", &mut ev, &mut sb, &mut ac);
        assert_eq!(ac, 14238);
    }
}
