use std::{fs::File, io::Read, path::Path};

/// Known magic-byte patterns and the handler hint they map to.
/// Returns `Some("handler_id")` if the file starts with the given signature.
const SIGNATURES: &[(&[u8], &str)] = &[
    // "UnityFS" — Unity asset bundle / compressed bundle
    (b"UnityFS", "unity"),
    // "MZ" — PE/Windows executable
    (&[0x4D, 0x5A], "pe"),
    // ELF — Linux/Android native binary
    (&[0x7F, 0x45, 0x4C, 0x46], "elf"),
    // PK\x03\x04 — ZIP archive (APK, JAR, OBB, ...)
    (&[0x50, 0x4B, 0x03, 0x04], "zip"),
];

/// Read the first `n` bytes of `path` without buffering the whole file.
fn read_header(path: &Path, n: usize) -> std::io::Result<Vec<u8>> {
    let mut f = File::open(path)?;
    let mut buf = vec![0u8; n];
    let read = f.read(&mut buf)?;
    buf.truncate(read);
    Ok(buf)
}

/// Probe `path` with all known magic-byte signatures.
/// Returns `Some(handler_hint)` for the first match, `None` if no match.
pub fn probe(path: &Path) -> Option<&'static str> {
    // Read enough bytes to cover the longest known signature (7 bytes for "UnityFS").
    let header = read_header(path, 8).ok()?;
    for (sig, hint) in SIGNATURES {
        if header.starts_with(sig) {
            return Some(hint);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn make_file(bytes: &[u8]) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("tempfile");
        f.write_all(bytes).expect("write");
        f
    }

    #[test]
    fn detects_unity_fs() {
        let f = make_file(b"UnityFS\x00some data here");
        assert_eq!(probe(f.path()), Some("unity"));
    }

    #[test]
    fn detects_pe_mz() {
        let f = make_file(&[0x4D, 0x5A, 0x90, 0x00, 0x03]);
        assert_eq!(probe(f.path()), Some("pe"));
    }

    #[test]
    fn detects_elf() {
        let f = make_file(&[0x7F, 0x45, 0x4C, 0x46, 0x02, 0x01]);
        assert_eq!(probe(f.path()), Some("elf"));
    }

    #[test]
    fn detects_zip() {
        let f = make_file(&[0x50, 0x4B, 0x03, 0x04, 0x14]);
        assert_eq!(probe(f.path()), Some("zip"));
    }

    #[test]
    fn unknown_returns_none() {
        let f = make_file(b"\x00\x01\x02\x03\x04\x05\x06\x07");
        assert_eq!(probe(f.path()), None);
    }
}
