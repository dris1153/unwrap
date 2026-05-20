# System Architecture — Unwrap

**Scope:** Windows-first desktop reverse engineering tool, Unity MVP, plugin-extensible.

---

## High-Level Layers

```
┌─────────────────────────────────────────────────────────────┐
│  PRESENTATION (React + TypeScript)                          │
│  ┌──────────┬───────────────┬──────────────────────────┐   │
│  │ FileTree │  Editor/Tabs  │  Inspector (collapsible) │   │
│  │ Sidebar  │  ┌─────────┐  │  - Props                 │   │
│  │          │  │ Preview │  │  - Refs                  │   │
│  │ Recents  │  │ panel   │  │  - Metadata              │   │
│  │ Search   │  └─────────┘  │                          │   │
│  └──────────┴───────────────┴──────────────────────────┘   │
│                          │                                  │
│                          ▼ (Tauri IPC, JSON)                │
├─────────────────────────────────────────────────────────────┤
│  APPLICATION (Rust / Tauri backend)                         │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Tauri Commands (IPC entry)                        │    │
│  │  open_project, scan, tree, preview, export, ...    │    │
│  └────────────────────────────────────────────────────┘    │
│                          │                                  │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Format Detection Service                          │    │
│  │  magic-bytes → extension → structural → user       │    │
│  └────────────────────────────────────────────────────┘    │
│                          │                                  │
│  ┌────────────────────────────────────────────────────┐    │
│  │  FormatHandler Registry (trait-based)              │    │
│  │  ┌─────────────┐  ┌──────────┐  ┌──────────────┐  │    │
│  │  │ UnityHandler│  │ (future) │  │ (future)     │  │    │
│  │  │             │  │ Unreal   │  │ PE/ELF       │  │    │
│  │  └──────┬──────┘  └──────────┘  └──────────────┘  │    │
│  └─────────┼──────────────────────────────────────────┘    │
│            │                                                │
│  ┌─────────▼──────────────────────────────────────────┐    │
│  │  Sidecar Process Manager                           │    │
│  │  (spawn, IPC, cancellation, progress, timeout)     │    │
│  └─────────┬──────────────────────────────────────────┘    │
│            │                                                │
│  ┌─────────▼──────────────────────────────────────────┐    │
│  │  Project Cache (SQLite)                            │    │
│  │  - Recent projects                                 │    │
│  │  - Asset tree snapshots (keyed by file hash)       │    │
│  │  - User notes/bookmarks                            │    │
│  └────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  EXTERNAL TOOLS (subprocess sidecars)                       │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────┐    │
│  │ AssetRipper  │ │  ILSpyCmd    │ │  Il2CppDumper    │    │
│  │  (GPL,sub)   │ │  (MIT)       │ │  (MIT, IL2CPP)   │    │
│  └──────────────┘ └──────────────┘ └──────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Domain Models

```rust
pub struct Project {
    pub id: ProjectId,                 // hash of root path
    pub root_path: PathBuf,
    pub format_id: String,             // "unity"
    pub engine_version: Option<String>,
    pub scripting_backend: ScriptingBackend, // Mono | IL2CPP
    pub created_at: DateTime<Utc>,
    pub last_opened: DateTime<Utc>,
}

pub struct AssetTree {
    pub root: NodeId,
    pub nodes: HashMap<NodeId, AssetNode>,
}

pub struct AssetNode {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub name: String,
    pub kind: AssetKind,               // Folder | Texture | Audio | Mesh | Text | Script | Binary
    pub size: u64,
    pub source_path: PathBuf,          // where AssetRipper extracted it
    pub metadata: HashMap<String, Value>,
}

pub enum PreviewPayload {
    Image { path: PathBuf, width: u32, height: u32 },
    Audio { path: PathBuf, duration_ms: u32 },
    Model3D { path: PathBuf, format: ModelFormat },
    Text { content: String, language: Option<String> },
    Code { content: String, language: String, references: Vec<CodeRef> },
    Hex { path: PathBuf, size: u64 },
    Empty,
}
```

---

## Key Workflows

### 1. Open Unity Project

```
User drops folder
    ↓
detect() runs across all registered handlers in parallel
    ↓
UnityHandler returns confidence=0.95 (found GameAssembly.dll + Data/)
    ↓
Detect scripting backend (Mono vs IL2CPP)
    ↓
Spawn AssetRipper sidecar → extracts to temp/<projectId>/
    ↓
Build AssetTree from extracted structure
    ↓
Cache tree in SQLite (keyed by source hash)
    ↓
Emit progress events via Tauri event channel
    ↓
UI populates left sidebar tree
```

### 2. Preview Texture

```
User clicks texture node in tree
    ↓
React: invoke('preview', { nodeId })
    ↓
Rust: lookup node → return PreviewPayload::Image { path, w, h }
    ↓
React: <img src={convertFileSrc(path)}/> with zoom-pan-pinch wrapper
    ↓
Inspector panel shows: dimensions, format, mipmaps, size
```

### 3. Decompile C# Script

```
User clicks script in tree
    ↓
If Mono: read managed DLL → ILSpyCmd subprocess → output.cs
If IL2CPP: read dummy DLL (already produced by Il2CppDumper) → ILSpyCmd
    ↓
Stream C# text back via stdout
    ↓
Monaco Editor renders with C# syntax highlighting
    ↓
Inspector shows: namespace, references, used-by (xref index)
```

### 4. Translate Text (Phase 2)

```
User finds TextAsset / string in tree
    ↓
Side-by-side editor: original | translation column
    ↓
Save translation to SQLite (keyed by node id + locale)
    ↓
(Phase 2+) Repack into new asset bundle via AssetRipper export mode
```

---

## IPC / Communication Patterns

- **React → Rust:** `invoke('command_name', { params })` via Tauri
- **Rust → React (events):** `app.emit("progress", payload)` for streaming progress
- **Rust → Sidecar:** `tokio::process::Command` with redirected stdin/stdout, line-by-line JSON parse
- **Cancellation:** `CancellationToken` propagated through async tree; sidecar gets SIGTERM/Kill

---

## Security & Trust Model

- All loaded files are user-supplied (trusted owner of own game files)
- Sidecar subprocesses run with user privileges (no elevation needed)
- v1: No external network calls except for optional update check (opt-in)
- v2+: Plugin signing, WASM sandbox for third-party code

---

## Performance Budget

| Operation | Target |
|-----------|--------|
| Cold start | < 2s |
| Open 1GB Unity build | < 30s (AssetRipper bound) |
| Render asset tree (10k nodes) | < 100ms (virtualized) |
| Switch preview tab | < 50ms |
| Search across project | < 200ms (SQLite FTS) |

---

## Unresolved Questions

- Tauri sidecar binary signing vs separate auto-update for AssetRipper?
- Should the extracted-asset temp dir be configurable / persistent?
- Multi-project workspace (open 2+ games side-by-side)? Defer to v2.
