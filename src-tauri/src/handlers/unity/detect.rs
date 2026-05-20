// detect.rs — Unity project detection helpers.
//
// Detection rules (ranked by reliability):
//   confidence 0.95: *_Data/ dir present AND (Managed/*.dll OR GameAssembly.dll + global-metadata.dat)
//   confidence 0.70: *_Data/ present but no confirmed scripting backend
//   confidence 0.00: no *_Data/ dir found
//
// IL2CPP false-positive guard: require BOTH GameAssembly.dll AND global-metadata.dat
// AND the *_Data/ directory — consistent with the spec security note.

use std::path::{Path, PathBuf};

use crate::domain::{
    detection::{DetectionCandidate, DetectionResult},
    project::ScriptingBackend,
};

use super::version::parse_engine_version_from_ggm;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run Unity-specific detection on `path`.
///
/// Returns a `DetectionResult` whose inner candidate has `handler_id = "unity"`.
pub async fn detect_unity(path: &Path) -> DetectionResult {
    // All FS probes are sync; run inside spawn_blocking so we don't block the async runtime.
    let path_owned = path.to_path_buf();
    tokio::task::spawn_blocking(move || detect_sync(&path_owned))
        .await
        .unwrap_or_else(|_| low_confidence_result())
}

// ---------------------------------------------------------------------------
// Sync implementation (called from spawn_blocking)
// ---------------------------------------------------------------------------

fn detect_sync(path: &Path) -> DetectionResult {
    let Some(data_dir) = find_data_dir(path) else {
        return DetectionResult::Chooser { candidates: vec![] };
    };

    let mono_present = is_mono_backend(&data_dir);
    let il2cpp_present = is_il2cpp_backend(path, &data_dir);

    let (backend, confidence) = match (mono_present, il2cpp_present) {
        (true, _) => (ScriptingBackend::Mono, 0.95_f32),
        (_, true) => (ScriptingBackend::Il2Cpp, 0.95_f32),
        _ => (ScriptingBackend::Unknown, 0.70_f32),
    };

    let engine_version = data_dir
        .join("globalgamemanagers")
        .is_file()
        .then(|| parse_engine_version_from_ggm(&data_dir.join("globalgamemanagers")).ok())
        .flatten()
        .unwrap_or_default();

    let reason = build_reason(&data_dir, &backend, &engine_version);

    let candidate = DetectionCandidate {
        handler_id: "unity".into(),
        display_name: "Unity".into(),
        confidence,
        reason: Some(reason),
    };

    // confidence >= 0.8 → Auto
    if confidence >= 0.8 {
        DetectionResult::Auto { candidate }
    } else {
        DetectionResult::Chooser {
            candidates: vec![candidate],
        }
    }
}

/// Scripting backend and confidence extracted from a detect result.
///
/// Returns `(backend, engine_version)` from the winning candidate hints,
/// or defaults if the result is a low-confidence chooser.
pub fn extract_hints(result: &DetectionResult) -> (ScriptingBackend, Option<String>) {
    // Decode backend + version from the reason string encoded by detect_sync.
    // reason format: "backend=mono|il2cpp|unknown;version=VVVV;ggm=true|false"
    let reason = candidate_reason(result).unwrap_or_default();
    let backend = if reason.contains("backend=mono") {
        ScriptingBackend::Mono
    } else if reason.contains("backend=il2cpp") {
        ScriptingBackend::Il2Cpp
    } else {
        ScriptingBackend::Unknown
    };
    let version = reason
        .split(';')
        .find(|s| s.starts_with("version="))
        .and_then(|s| s.strip_prefix("version="))
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    (backend, version)
}

// ---------------------------------------------------------------------------
// Filesystem probes
// ---------------------------------------------------------------------------

/// Find a `*_Data/` directory directly under `root`.
///
/// Unity exports exactly one `<GameName>_Data/` folder next to the executable.
/// We accept the first match found.
pub(super) fn find_data_dir(root: &Path) -> Option<PathBuf> {
    let rd = std::fs::read_dir(root).ok()?;
    for entry in rd.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("_Data") && entry.path().is_dir() {
            return Some(entry.path());
        }
    }
    None
}

/// Check for Mono backend: `*_Data/Managed/` dir with at least one `.dll`.
fn is_mono_backend(data_dir: &Path) -> bool {
    let managed = data_dir.join("Managed");
    if !managed.is_dir() {
        return false;
    }
    has_dll(&managed)
}

/// Check for IL2CPP backend: `GameAssembly.dll` (in parent of data_dir)
/// AND `*_Data/il2cpp_data/Metadata/global-metadata.dat`.
fn is_il2cpp_backend(root: &Path, data_dir: &Path) -> bool {
    let game_assembly = find_game_assembly_dll(root).is_some();
    let metadata = data_dir
        .join("il2cpp_data")
        .join("Metadata")
        .join("global-metadata.dat")
        .is_file();
    game_assembly && metadata
}

/// True if `dir` contains at least one `.dll` file.
pub(super) fn has_dll(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .ok()
        .map(|rd| {
            rd.flatten().any(|e| {
                e.path()
                    .extension()
                    .map(|x| x.eq_ignore_ascii_case("dll"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Find `GameAssembly.dll` directly under `root` (case-insensitive on Windows
/// but we check the typical casing first for speed).
pub(super) fn find_game_assembly_dll(root: &Path) -> Option<PathBuf> {
    let candidate = root.join("GameAssembly.dll");
    if candidate.is_file() {
        return Some(candidate);
    }
    // Case-insensitive fallback scan
    std::fs::read_dir(root).ok()?.flatten().find_map(|e| {
        let name = e.file_name();
        let n = name.to_string_lossy();
        if n.to_ascii_lowercase() == "gameassembly.dll" {
            Some(e.path())
        } else {
            None
        }
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn low_confidence_result() -> DetectionResult {
    DetectionResult::Chooser { candidates: vec![] }
}

fn build_reason(data_dir: &Path, backend: &ScriptingBackend, engine_version: &str) -> String {
    let ggm = data_dir.join("globalgamemanagers").is_file();
    format!(
        "backend={};version={};ggm={}",
        backend,
        engine_version,
        ggm
    )
}

fn candidate_reason(result: &DetectionResult) -> Option<String> {
    match result {
        DetectionResult::Auto { candidate } => candidate.reason.clone(),
        DetectionResult::Chooser { candidates } => {
            candidates.first().and_then(|c| c.reason.clone())
        }
    }
}
