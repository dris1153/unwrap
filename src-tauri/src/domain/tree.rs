use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Opaque node identifier within an asset tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        NodeId(s)
    }
}

/// Category of an asset node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Folder,
    Texture,
    Audio,
    Mesh,
    Text,
    Script,
    Scene,
    Material,
    Shader,
    Animation,
    Prefab,
    Binary,
}

/// A single node in the asset tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetNode {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub name: String,
    pub kind: AssetKind,
    pub size: u64,
    /// Filesystem path (after extraction by sidecar).
    pub source_path: String,
    pub metadata: HashMap<String, Value>,
}

/// Full asset tree for an open project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTree {
    pub root: NodeId,
    pub nodes: HashMap<String, AssetNode>,
}
