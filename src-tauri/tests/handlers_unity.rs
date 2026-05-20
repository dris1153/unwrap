// handlers_unity.rs — integration tests for UnityHandler.
//
// Uses the synthetic fixture at tests/fixtures/unity-synthetic-mono/.
// Tests that require a real Unity build are marked #[ignore].
//
// Run only these tests:
//   cargo test --test handlers_unity

use std::path::PathBuf;

/// Absolute path to the synthetic Mono fixture.
fn mono_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unity-synthetic-mono")
}

/// Absolute path to a non-Unity directory (the Cargo manifest dir itself).
fn non_unity_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// ---------------------------------------------------------------------------
// Detection tests
// ---------------------------------------------------------------------------

#[test]
fn test_detect_mono_unity() {
    let fixture = mono_fixture();
    assert!(fixture.exists(), "synthetic fixture missing: {}", fixture.display());

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            use unwrap_lib::handlers::unity::detect::detect_unity;
            detect_unity(&fixture).await
        });

    use unwrap_lib::domain::detection::DetectionResult;
    match result {
        DetectionResult::Auto { candidate } => {
            assert_eq!(candidate.handler_id, "unity");
            assert!(
                candidate.confidence >= 0.95,
                "expected confidence >= 0.95, got {}",
                candidate.confidence
            );
            // Reason must encode mono backend
            let reason = candidate.reason.unwrap_or_default();
            assert!(
                reason.contains("backend=mono"),
                "expected reason to contain 'backend=mono', got: {reason}"
            );
        }
        DetectionResult::Chooser { candidates } => {
            panic!(
                "expected Auto result for Mono fixture, got Chooser with {} candidates",
                candidates.len()
            );
        }
    }
}

#[test]
fn test_detect_no_unity() {
    let dir = non_unity_dir();

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            use unwrap_lib::handlers::unity::detect::detect_unity;
            detect_unity(&dir).await
        });

    use unwrap_lib::domain::detection::DetectionResult;
    match result {
        DetectionResult::Auto { candidate } => {
            panic!(
                "non-Unity dir should not return Auto, got confidence {}",
                candidate.confidence
            );
        }
        DetectionResult::Chooser { candidates } => {
            // Either empty or all candidates have confidence < 0.8
            for c in &candidates {
                assert!(
                    c.confidence < 0.8,
                    "non-Unity dir should not produce high-confidence candidate, got {}",
                    c.confidence
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Version parsing
// ---------------------------------------------------------------------------

#[test]
fn test_parse_engine_version_from_synthetic_fixture() {
    use unwrap_lib::handlers::unity::version::parse_engine_version_from_ggm;

    let ggm = mono_fixture()
        .join("unity-synthetic-mono_Data")
        .join("globalgamemanagers");

    assert!(ggm.exists(), "synthetic ggm missing: {}", ggm.display());

    let version = parse_engine_version_from_ggm(&ggm).expect("should parse version");
    assert_eq!(version, "2022.3.18f1");
}

// ---------------------------------------------------------------------------
// Cache key
// ---------------------------------------------------------------------------

#[test]
fn test_cache_key_deterministic_for_fixture() {
    use unwrap_lib::handlers::unity::cache_key::compute_project_id;

    let fixture = mono_fixture();
    let id1 = compute_project_id(&fixture);
    let id2 = compute_project_id(&fixture);
    assert_eq!(id1, id2, "project id must be deterministic");
    assert_eq!(id1.0.len(), 16, "project id must be 16 hex chars");
}

// ---------------------------------------------------------------------------
// Tree builder
// ---------------------------------------------------------------------------

#[test]
fn test_tree_walk_on_synthetic_fixture() {
    let fixture = mono_fixture()
        .join("unity-synthetic-mono_Data");

    let tree = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            use unwrap_lib::handlers::unity::tree_builder::walk;
            walk(&fixture).await
        })
        .expect("tree walk should succeed");

    // Expected nodes: root + Managed + Assembly-CSharp.dll + UnityEngine.dll
    //                + globalgamemanagers + sharedassets0.assets
    //                + Resources + unity_default_resources = 8
    assert!(
        tree.nodes.len() >= 5,
        "expected >= 5 nodes, got {}",
        tree.nodes.len()
    );

    // Root node must exist and be a Folder
    let root_node = tree.nodes.get(&tree.root.0).expect("root node must exist");
    use unwrap_lib::domain::tree::AssetKind;
    assert_eq!(root_node.kind, AssetKind::Folder);
}

// ---------------------------------------------------------------------------
// Open round-trip (requires real Unity build — ignored in CI)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires a real Unity build at UNITY_BUILD_PATH env var"]
fn test_open_round_trip() {
    let path = std::env::var("UNITY_BUILD_PATH")
        .expect("set UNITY_BUILD_PATH to a real Unity build folder");
    let p = std::path::Path::new(&path);

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            use unwrap_lib::handlers::unity::detect::detect_unity;
            let result = detect_unity(p).await;
            use unwrap_lib::domain::detection::DetectionResult;
            match result {
                DetectionResult::Auto { candidate } => {
                    assert!(candidate.confidence >= 0.95);
                }
                DetectionResult::Chooser { .. } => {
                    panic!("expected Auto for real Unity build");
                }
            }
        });
}
