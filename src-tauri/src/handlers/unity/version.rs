// version.rs — Parse Unity engine version from a globalgamemanagers binary.
//
// Unity serialized-file header layout (little-endian, first ~64 bytes):
//
//   Offset  Size  Field
//   ------  ----  -----
//   0       4     metadata_size (u32 BE)
//   4       4     file_size     (u32 BE) [or u64 on newer formats]
//   8       4     version       (u32 BE) — serialized file format version
//   12      4     data_offset   (u32 BE)
//   16      4     endianness    (u32, 0=LE 1=BE)
//   20      4     reserved
//   24      ..    version string (null-terminated ASCII), e.g. "2022.3.18f1\0"
//
// The version string starts at byte 20 in Unity serialized-file format ≥ 9
// (covers Unity 5.x through Unity 6000.x). For pre-5.x builds (format ≤ 8)
// the string is at offset 8 — but those are outside Unwrap's supported range.
//
// We read the first 64 bytes and scan for the null-terminated version string
// starting at offset 20. If bytes there are not ASCII-printable or the string
// doesn't match a Unity version regex, we fall back to a broader scan.

use std::path::Path;

use crate::domain::error::AppError;

/// The expected byte offset of the version string in Unity ggm (format ≥ 9).
const VERSION_STRING_OFFSET: usize = 20;
/// We never need more than the first 64 bytes for detection.
const HEADER_READ_SIZE: usize = 64;

/// Parse Unity engine version (e.g. `"2022.3.18f1"`) from a
/// `globalgamemanagers` (or any Unity serialized-file header).
///
/// Returns `Err(AppError::Io)` if the file cannot be read.
/// Returns `Err(AppError::DetectFailed)` if the version string is not found.
pub fn parse_engine_version_from_ggm(path: &Path) -> Result<String, AppError> {
    use std::io::Read;

    let mut f = std::fs::File::open(path)
        .map_err(|e| AppError::Io(format!("open ggm: {e}")))?;

    let mut buf = [0u8; HEADER_READ_SIZE];
    let n = f.read(&mut buf).map_err(|e| AppError::Io(format!("read ggm: {e}")))?;
    let header = &buf[..n];

    // Primary: null-terminated ASCII string starting at offset 20.
    if header.len() > VERSION_STRING_OFFSET {
        let slice = &header[VERSION_STRING_OFFSET..];
        if let Some(v) = read_nul_terminated_version(slice) {
            return Ok(v);
        }
    }

    // Fallback: scan entire header for a Unity-version-like ASCII string.
    if let Some(v) = scan_for_version(header) {
        return Ok(v);
    }

    Err(AppError::DetectFailed(
        "Unity engine version string not found in ggm header".into(),
    ))
}

/// Extract a null-terminated Unity version string from `slice`.
/// Accepts strings matching `\d+\.\d+\.\d+[a-z]\d+` (e.g. `2022.3.18f1`).
fn read_nul_terminated_version(slice: &[u8]) -> Option<String> {
    // Find first NUL or end of slice.
    let end = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
    if end == 0 || end > 32 {
        return None; // Too short or too long to be a version string
    }
    let s = std::str::from_utf8(&slice[..end]).ok()?;
    if looks_like_unity_version(s) {
        Some(s.to_string())
    } else {
        None
    }
}

/// Scan raw bytes for a Unity version pattern anywhere in the buffer.
fn scan_for_version(header: &[u8]) -> Option<String> {
    // Convert to UTF-8 lossy so we can use str methods.
    let text = String::from_utf8_lossy(header);
    // Find digit sequences that look like "NNNN.N.NNfN".
    for start in 0..text.len().saturating_sub(6) {
        let candidate = &text[start..text.len().min(start + 24)];
        // Quick filter: must start with a digit
        if !candidate.starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }
        // Extract up to the first non-version character
        let end = candidate
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '.')
            .unwrap_or(candidate.len());
        let tok = &candidate[..end];
        if looks_like_unity_version(tok) {
            return Some(tok.to_string());
        }
    }
    None
}

/// Returns true when `s` matches the Unity version format `MAJOR.MINOR.PATCHxN`
/// where x is a lowercase letter (a=alpha, b=beta, f=final, p=patch, rc=rc, c=china).
pub fn looks_like_unity_version(s: &str) -> bool {
    // Must have at least two dots
    let parts: Vec<&str> = s.splitn(3, '.').collect();
    if parts.len() < 3 {
        return false;
    }
    // MAJOR and MINOR must be numeric
    if !parts[0].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if !parts[1].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let major: u32 = parts[0].parse().unwrap_or(0);
    // Unity versions are 4–6000+; reject obviously wrong numbers
    if !(4..=9000).contains(&major) {
        return false;
    }
    // PATCH part must contain at least one digit + a letter suffix
    let patch = parts[2];
    let has_digit = patch.chars().any(|c| c.is_ascii_digit());
    let has_suffix = patch.chars().any(|c| c.is_ascii_alphabetic());
    has_digit && has_suffix
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Build a synthetic globalgamemanagers header with version `"2022.3.18f1"`.
    ///
    /// Layout matches Unity serialized-file format ≥ 9:
    ///   bytes 0-3:   metadata_size (big-endian u32) = 0
    ///   bytes 4-7:   file_size (big-endian u32) = 0
    ///   bytes 8-11:  serialized format version (big-endian u32) = 22 (Unity 2022.x typical)
    ///   bytes 12-15: data_offset (big-endian u32) = 0
    ///   bytes 16-19: endianness + reserved = 0
    ///   bytes 20+:   null-terminated version string
    fn make_synthetic_ggm(version: &str) -> Vec<u8> {
        let mut buf = vec![0u8; HEADER_READ_SIZE];
        // Format version = 22 at offset 8, big-endian
        buf[8] = 0;
        buf[9] = 0;
        buf[10] = 0;
        buf[11] = 22;
        // Version string at offset 20
        let vbytes = version.as_bytes();
        let copy_len = vbytes.len().min(HEADER_READ_SIZE - VERSION_STRING_OFFSET - 1);
        buf[VERSION_STRING_OFFSET..VERSION_STRING_OFFSET + copy_len]
            .copy_from_slice(&vbytes[..copy_len]);
        buf[VERSION_STRING_OFFSET + copy_len] = 0; // null terminator
        buf
    }

    #[test]
    fn parse_known_version_2022() {
        let data = make_synthetic_ggm("2022.3.18f1");
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(&data).unwrap();
        let result = parse_engine_version_from_ggm(f.path()).unwrap();
        assert_eq!(result, "2022.3.18f1");
    }

    #[test]
    fn parse_known_version_2019() {
        let data = make_synthetic_ggm("2019.4.40f1");
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(&data).unwrap();
        let result = parse_engine_version_from_ggm(f.path()).unwrap();
        assert_eq!(result, "2019.4.40f1");
    }

    #[test]
    fn parse_known_version_6000() {
        let data = make_synthetic_ggm("6000.0.1f1");
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(&data).unwrap();
        let result = parse_engine_version_from_ggm(f.path()).unwrap();
        assert_eq!(result, "6000.0.1f1");
    }

    #[test]
    fn looks_like_unity_version_valid() {
        assert!(looks_like_unity_version("2022.3.18f1"));
        assert!(looks_like_unity_version("2019.4.40f1"));
        assert!(looks_like_unity_version("5.6.7f2"));
        assert!(looks_like_unity_version("6000.0.1f1"));
        assert!(looks_like_unity_version("2021.2.1a5")); // alpha
        assert!(looks_like_unity_version("2023.1.0b3")); // beta
    }

    #[test]
    fn looks_like_unity_version_invalid() {
        assert!(!looks_like_unity_version("1.2.3")); // no letter suffix
        assert!(!looks_like_unity_version("not.a.version"));
        assert!(!looks_like_unity_version(""));
        assert!(!looks_like_unity_version("1.0")); // only two parts
    }
}
