// dummy_dlls.rs — locate IL2CPP dummy DLLs produced by Il2CppDumper.
//
// Il2CppDumper writes a flat set of dummy DLLs to <cache>/<project_id>/dummy_dlls/.
// This module provides a best-effort search for the DLL that contains a given class.
// For v1 we use a simple naming heuristic: the class namespace prefix often matches
// the DLL filename (e.g. "UnityEngine.CoreModule" → "UnityEngine.CoreModule.dll").
// If that fails we return None and the caller falls back to dump.cs slicing.

use std::path::{Path, PathBuf};

/// Attempt to locate the dummy DLL for `class_fullname` under `dummy_dir`.
///
/// Strategy (in order):
/// 1. Strip the final component of `class_fullname` to get candidate namespace prefixes,
///    then look for a `<prefix>.dll` in `dummy_dir` (longest match first).
/// 2. If no match found, return `None`.
pub fn locate_for_class(dummy_dir: &Path, class_fullname: &str) -> Option<PathBuf> {
    if !dummy_dir.is_dir() {
        return None;
    }

    // Build candidate DLL names from longest namespace prefix to shortest.
    let parts: Vec<&str> = class_fullname.split('.').collect();
    // Try from (all parts - 1) down to 1 component (skip class name itself).
    for end in (1..parts.len()).rev() {
        let candidate_stem = parts[..end].join(".");
        let candidate = dummy_dir.join(format!("{candidate_stem}.dll"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    // Fallback: scan directory for any DLL whose stem is a prefix of the fullname.
    if let Ok(entries) = std::fs::read_dir(dummy_dir) {
        let mut best: Option<PathBuf> = None;
        let mut best_len = 0usize;

        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().map(|e| e.eq_ignore_ascii_case("dll")).unwrap_or(false) {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    if class_fullname.starts_with(stem) && stem.len() > best_len {
                        best_len = stem.len();
                        best = Some(p);
                    }
                }
            }
        }

        if best.is_some() {
            return best;
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn finds_exact_namespace_dll() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("UnityEngine.CoreModule.dll"), b"").unwrap();
        let result = locate_for_class(dir.path(), "UnityEngine.CoreModule.Transform");
        assert!(result.is_some());
        assert!(result.unwrap().file_name().unwrap() == "UnityEngine.CoreModule.dll");
    }

    #[test]
    fn returns_none_for_no_match() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("SomeOther.dll"), b"").unwrap();
        let result = locate_for_class(dir.path(), "UnityEngine.CoreModule.Transform");
        assert!(result.is_none());
    }

    #[test]
    fn returns_none_for_missing_dir() {
        let result = locate_for_class(Path::new("/nonexistent/path"), "Foo.Bar");
        assert!(result.is_none());
    }
}
