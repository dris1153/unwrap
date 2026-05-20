use serde::{Deserialize, Serialize};

/// One candidate returned by `detect_all` — pairs handler with its score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionCandidate {
    pub handler_id: String,
    pub display_name: String,
    pub confidence: f32,
    pub reason: Option<String>,
}

/// Result of the full detection pipeline.
/// Auto: exactly one handler scored >= 0.8; the frontend can proceed automatically.
/// Chooser: multiple candidates or none with high confidence; show picker UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DetectionResult {
    /// Single confident match — frontend opens without user input.
    Auto { candidate: DetectionCandidate },
    /// Ambiguous or low confidence — frontend shows chooser dialog.
    Chooser { candidates: Vec<DetectionCandidate> },
}
