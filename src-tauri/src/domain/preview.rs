use serde::{Deserialize, Serialize};

use crate::domain::project::ScriptingBackend;

/// 3D model container format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFormat {
    Glb,
    Fbx,
    Obj,
}

/// A cross-reference link inside decompiled code (phase-05+).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRef {
    pub symbol: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

/// Full decompile result returned by the `decompile` command (phase-07).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecompilePayload {
    /// Origin of the content: "raw" | "ilspy" | "il2cpp-dump"
    pub source: String,
    pub backend: ScriptingBackend,
    /// 0.0–1.0 accuracy estimate.
    pub confidence: f32,
    /// Always "csharp" for v1.
    pub language: String,
    /// Full file text including header comment injected by backend.
    pub content: String,
    /// Best-effort class name, e.g. "HollowVale.Gameplay.PlayerController".
    pub class_fullname: Option<String>,
    /// Assembly DLL basename, e.g. "Assembly-CSharp.dll".
    pub assembly: Option<String>,
    /// Outgoing references parsed from `using` statements.
    pub references: Vec<CodeRef>,
    /// Wall-clock decompile time in milliseconds.
    pub elapsed_ms: u32,
}

/// Discriminated union sent to the frontend for the preview panel.
/// Each variant carries only the data the renderer needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PreviewPayload {
    Image {
        path: String,
        width: u32,
        height: u32,
    },
    Audio {
        path: String,
        duration_ms: u32,
    },
    Model3D {
        path: String,
        format: ModelFormat,
    },
    Text {
        content: String,
        language: Option<String>,
    },
    Code {
        content: String,
        language: String,
        references: Vec<CodeRef>,
        // Phase-07 decompile fields (present when node was decompiled; None for raw .cs preview).
        source: Option<String>,
        backend: Option<ScriptingBackend>,
        confidence: Option<f32>,
        class_fullname: Option<String>,
        assembly: Option<String>,
        elapsed_ms: Option<u32>,
    },
    Hex {
        path: String,
        size: u64,
    },
    Empty,
}
