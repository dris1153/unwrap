// tree_builder.rs — walk an extracted asset directory and build an AssetTree.
//
// Two-pass approach:
//   1. walkdir::WalkDir recurse → produce AssetNode for every entry.
//   2. Collect into HashMap; set root NodeId.
//
// Node IDs are SHA-256(rel_path)[..16] — deterministic across runs for the
// same extraction output. Parent links are set from the relative path parent.
//
// Performance notes:
//   - walkdir is sync; the entire walk runs inside spawn_blocking.
//   - We avoid collect::<Vec> by building the HashMap incrementally.
//   - Path strings stored as String (compact_str available but String is
//     sufficient here since node count rarely exceeds 200k).

use std::{
    collections::HashMap,
    path::Path,
};

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::domain::{
    error::AppError,
    tree::{AssetKind, AssetNode, AssetTree, NodeId},
};

use super::kind_table::ext_to_kind;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Walk `root` recursively and build an `AssetTree`.
///
/// Runs on a blocking thread via `tokio::task::spawn_blocking` — safe to
/// `.await` from async context.
pub async fn walk(root: &Path) -> Result<AssetTree, AppError> {
    let root_owned = root.to_path_buf();
    tokio::task::spawn_blocking(move || walk_sync(&root_owned))
        .await
        .map_err(|e| AppError::Io(format!("spawn_blocking tree walk: {e}")))?
}

// ---------------------------------------------------------------------------
// Sync implementation
// ---------------------------------------------------------------------------

fn walk_sync(root: &Path) -> Result<AssetTree, AppError> {
    let mut nodes: HashMap<String, AssetNode> = HashMap::new();

    // Canonical root node id: SHA-256 of empty string (represents ".")
    let root_id = node_id_for_rel(Path::new(""));

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path());

        let id = node_id_for_rel(rel);

        let kind = if entry.file_type().is_dir() {
            AssetKind::Folder
        } else {
            ext_to_kind(rel)
        };

        // Determine parent: parent of relative path, or None for root itself.
        let parent: Option<NodeId> = if rel == Path::new("") {
            None
        } else {
            let parent_rel = rel.parent().unwrap_or(Path::new(""));
            Some(node_id_for_rel(parent_rel))
        };

        let name = entry
            .file_name()
            .to_string_lossy()
            .into_owned();

        // Size: 0 for directories (avoids extra syscall on every dir entry).
        let size = if entry.file_type().is_file() {
            entry.metadata().map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        let node = AssetNode {
            id: id.clone(),
            parent,
            name: if name.is_empty() {
                root.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| ".".to_string())
            } else {
                name
            },
            kind,
            size,
            source_path: entry.path().to_string_lossy().into_owned(),
            metadata: HashMap::new(),
        };

        nodes.insert(id.0.clone(), node);
    }

    // Ensure root node exists even for empty directories.
    if !nodes.contains_key(&root_id.0) {
        let root_name = root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| ".".to_string());
        nodes.insert(
            root_id.0.clone(),
            AssetNode {
                id: root_id.clone(),
                parent: None,
                name: root_name,
                kind: AssetKind::Folder,
                size: 0,
                source_path: root.to_string_lossy().into_owned(),
                metadata: HashMap::new(),
            },
        );
    }

    Ok(AssetTree {
        root: root_id,
        nodes,
    })
}

/// Compute a deterministic `NodeId` from a relative path.
///
/// Uses SHA-256(UTF-8 forward-slash-normalized path)[..16] so that IDs are
/// stable across OS path separator differences.
fn node_id_for_rel(rel: &Path) -> NodeId {
    // Normalize to forward slashes for cross-platform determinism.
    let normalized = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/");
    let hex = hex::encode(Sha256::digest(normalized.as_bytes()));
    NodeId(hex[..16].to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_tree() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        fs::create_dir(root.join("Managed")).unwrap();
        fs::write(root.join("Managed").join("Assembly-CSharp.dll"), b"MZ").unwrap();
        fs::write(root.join("Managed").join("UnityEngine.dll"), b"MZ").unwrap();
        fs::write(root.join("globalgamemanagers"), b"\x00\x00\x00\x00").unwrap();
        fs::create_dir(root.join("Resources")).unwrap();
        fs::write(root.join("Resources").join("unity_default_resources"), vec![0u8; 128]).unwrap();
        dir
    }

    #[tokio::test]
    async fn walk_returns_expected_count() {
        let dir = make_tree();
        let tree = walk(dir.path()).await.unwrap();
        // root + Managed + 2 dlls + globalgamemanagers + Resources + unity_default_resources = 7
        assert!(tree.nodes.len() >= 7, "expected ≥7 nodes, got {}", tree.nodes.len());
    }

    #[tokio::test]
    async fn walk_root_node_is_folder() {
        let dir = make_tree();
        let tree = walk(dir.path()).await.unwrap();
        let root_node = tree.nodes.get(&tree.root.0).expect("root node must exist");
        assert_eq!(root_node.kind, AssetKind::Folder);
        assert!(root_node.parent.is_none());
    }

    #[tokio::test]
    async fn walk_dll_classified_as_script() {
        let dir = make_tree();
        let tree = walk(dir.path()).await.unwrap();
        let dll_node = tree
            .nodes
            .values()
            .find(|n| n.name == "Assembly-CSharp.dll")
            .expect("dll node must exist");
        assert_eq!(dll_node.kind, AssetKind::Script);
    }

    #[tokio::test]
    async fn node_id_deterministic() {
        let id1 = node_id_for_rel(Path::new("Managed/Assembly-CSharp.dll"));
        let id2 = node_id_for_rel(Path::new("Managed/Assembly-CSharp.dll"));
        assert_eq!(id1, id2);
        assert_eq!(id1.0.len(), 16);
    }
}
