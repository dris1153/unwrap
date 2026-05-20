use std::path::Path;

/// Structural directory-pattern detection.
///
/// Performs lightweight filesystem checks (directory presence, well-known file
/// names) to produce a handler hint without reading file contents.
///
/// Phase-05 will fill in the Unity-specific logic here (checking for
/// `*_Data/`, `GameAssembly.dll`, `global-metadata.dat`, etc.).
/// For now this is a stub that always returns `None`.
pub fn probe(_path: &Path) -> Option<&'static str> {
    // TODO(phase-05): implement Unity structural detection
    None
}
