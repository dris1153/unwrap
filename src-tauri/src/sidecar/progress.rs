// SidecarMsg and parse_line used by phase-05+ handlers; suppress dead_code until then.
#![allow(dead_code)]

use serde::Deserialize;

/// A message emitted by a running sidecar process.
#[derive(Debug, Clone)]
pub enum SidecarMsg {
    /// Parsed progress update from a JSON line.
    Progress { percent: f32, message: String },
    /// Plain-text log line (non-JSON stdout).
    Log(String),
    /// Process exited normally.
    Done,
    /// Process exited with an error message.
    Error(String),
}

/// Intermediate JSON shape expected from sidecar stdout.
#[derive(Debug, Deserialize)]
struct ProgressLine {
    percent: Option<f32>,
    message: Option<String>,
}

/// Parse a single stdout line into a `SidecarMsg`.
///
/// Lines that start with `{` are treated as JSON progress objects.
/// All other lines become `SidecarMsg::Log`.
pub fn parse_line(line: &str) -> SidecarMsg {
    let trimmed = line.trim();
    if trimmed.starts_with('{') {
        match serde_json::from_str::<ProgressLine>(trimmed) {
            Ok(p) => SidecarMsg::Progress {
                percent: p.percent.unwrap_or(0.0),
                message: p.message.unwrap_or_default(),
            },
            Err(_) => SidecarMsg::Log(line.to_string()),
        }
    } else {
        SidecarMsg::Log(line.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_progress() {
        let msg = parse_line(r#"{"percent":42.5,"message":"extracting textures"}"#);
        match msg {
            SidecarMsg::Progress { percent, message } => {
                assert!((percent - 42.5).abs() < 0.001);
                assert_eq!(message, "extracting textures");
            }
            other => panic!("expected Progress, got {other:?}"),
        }
    }

    #[test]
    fn plain_text_becomes_log() {
        let msg = parse_line("some plain log line");
        match msg {
            SidecarMsg::Log(s) => assert_eq!(s, "some plain log line"),
            other => panic!("expected Log, got {other:?}"),
        }
    }
}
