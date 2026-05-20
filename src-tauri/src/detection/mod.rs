use std::path::Path;

use futures::future::join_all;

use crate::domain::detection::{DetectionCandidate, DetectionResult};
use crate::handlers::FormatHandlerRegistry;

pub mod magic;
pub mod structural;

/// Run the full detection pipeline for `path` against all registered handlers.
///
/// Pipeline order (highest-priority first):
/// 1. Magic-byte probe (fast, file-content based).
/// 2. Extension heuristic (delegated to each handler's `detect()`).
/// 3. Structural check (directory-pattern probe).
///
/// All handlers' `detect()` futures run in parallel via `join_all`.
/// The candidate with the highest confidence wins.
/// If the best confidence >= 0.8 → `DetectionResult::Auto`.
/// Otherwise → `DetectionResult::Chooser` (user picks from ranked list).
pub async fn run_pipeline(path: &Path, registry: &FormatHandlerRegistry) -> DetectionResult {
    // Boost hints from magic-byte and structural probes.
    let magic_hint = magic::probe(path);
    let struct_hint = if path.is_dir() {
        structural::probe(path)
    } else {
        None
    };

    let handlers = registry.handlers();

    if handlers.is_empty() {
        // No handlers registered yet (pre-phase-05). Return empty chooser.
        return DetectionResult::Chooser {
            candidates: Vec::new(),
        };
    }

    // Fan out detect() calls in parallel.
    let raw: Vec<DetectionResult> = join_all(handlers.iter().map(|h| h.detect(path))).await;

    // Collect all candidates, merging handler results with probe hints.
    let mut candidates: Vec<DetectionCandidate> = handlers
        .iter()
        .zip(raw)
        .map(|(handler, result)| {
            // Extract confidence from whatever the handler returned.
            let (mut confidence, reason) = match &result {
                DetectionResult::Auto { candidate } => {
                    (candidate.confidence, candidate.reason.clone())
                }
                DetectionResult::Chooser { candidates } => candidates
                    .first()
                    .map(|c| (c.confidence, c.reason.clone()))
                    .unwrap_or((0.0, None)),
            };

            // Boost confidence when magic-byte or structural probes agree.
            let id = handler.id();
            if magic_hint == Some(id) || struct_hint == Some(id) {
                // Cap at 1.0 after boost.
                confidence = (confidence + 0.3).min(1.0);
            }

            DetectionCandidate {
                handler_id: id.to_string(),
                display_name: handler.display_name().to_string(),
                confidence,
                reason,
            }
        })
        .collect();

    // Sort descending by confidence.
    candidates.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Auto-pick if top candidate is confident enough.
    if candidates
        .first()
        .map(|c| c.confidence >= 0.8)
        .unwrap_or(false)
    {
        let winner = candidates.into_iter().next().expect("checked non-empty");
        DetectionResult::Auto { candidate: winner }
    } else {
        DetectionResult::Chooser { candidates }
    }
}
