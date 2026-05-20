# Tech Stack — Unwrap Reverse Engineering Tool

**Decision date:** 2026-05-20
**Status:** Approved (auto-mode bootstrap)
**Scope:** Windows-first desktop app; Unity decompilation MVP; extensible to other formats.

---

## Stack Summary

| Layer | Choice | Why |
|-------|--------|-----|
| Desktop shell | **Tauri 2** | 10MB bundle, 1.4s startup, Rust perf, WebView2 native Win11 |
| UI runtime | **React 18 + TypeScript + Vite** | Modern DX, fast HMR, type safety |
| Styling | **Tailwind CSS 4 + shadcn/ui** | Dark-first, composable, matches VS Code/Linear aesthetic |
| Routing | **TanStack Router** | File-based, type-safe nav |
| Server state | **TanStack Query** | Async ops to Rust backend |
| Client state | **Zustand** | Tiny, no boilerplate |
| Code editor | **Monaco Editor** | C# decompile viewer w/ syntax highlight + folding |
| Backend lang | **Rust** | Tauri-native, async via tokio, file scanning perf |
| Process mgmt | **tokio::process + tauri-plugin-shell** | Sidecar binaries, cancellation, progress streaming |
| Storage | **SQLite (rusqlite)** | Project cache, asset index |
| 3D preview | **three.js / @react-three/fiber** | GLB/FBX/OBJ orbit viewer |
| Audio preview | **wavesurfer.js** | Waveform + scrub |
| Image preview | **Native `<img>` + zoom-pan-pinch** | Checker bg, zoom |
| Hex viewer | **react-hexkit (or custom)** | Endianness toggle |

---

## External Tools (Sidecars)

Shipped as separate binaries beside Tauri exe, invoked via subprocess (no GPL infection):

| Tool | License | Purpose | Strategy |
|------|---------|---------|----------|
| **AssetRipper.exe** | GPL-3.0 | Unity asset extraction (all types, Unity 3.5→6000) | Subprocess only (GPL-safe via separate process) |
| **ILSpyCmd** | MIT | Mono .NET → C# decompile | Embedded sidecar |
| **Il2CppDumper.exe** | MIT | IL2CPP → dummy DLLs (then ILSpy) | Subprocess, conditional on IL2CPP detection |
| **Cpp2IL** (future) | MIT | Better IL2CPP reconstruction | Phase 2 |

**Auto-detection flow:**
1. Detect Mono (`globalgamemanagers` → `BuildSettings` + `Managed/*.dll`) → ILSpy path
2. Detect IL2CPP (`GameAssembly.dll` + `global-metadata.dat`) → Il2CppDumper → ILSpy path
3. Assets → AssetRipper for all (handles both)

---

## Plugin Architecture (Designed Now, Built Incrementally)

**V1 (MVP):** Hard-coded Unity handler implementing internal `IFormatHandler` Rust trait. Subprocess wrappers around external tools.

**Trait surface (frozen for v1, stable for v2):**

```rust
#[async_trait]
pub trait FormatHandler: Send + Sync {
    fn id(&self) -> &str;                                      // e.g., "unity"
    fn display_name(&self) -> &str;                            // e.g., "Unity Game"
    async fn detect(&self, path: &Path) -> DetectionResult;    // confidence 0.0-1.0
    async fn open(&self, path: &Path) -> Result<ProjectHandle>;
    async fn tree(&self, handle: &ProjectHandle) -> AssetTree;
    async fn preview(&self, node: &NodeId) -> PreviewPayload;  // image/audio/3d/text/hex/code
    async fn export(&self, node: &NodeId, dest: &Path) -> Result<()>;
}
```

**V2:** Same trait exposed via subprocess-plugin contract (stdin/stdout JSON-RPC). Python/JS plugin SDKs.

**V3:** Marketplace, signed plugins, WASM sandbox.

---

## Format Detection Pipeline

```
file/folder dropped
    ↓
magic bytes scan (UnityFS, MZ for PE, PK for zip, etc.)
    ↓
extension heuristic (*.assets, *.bundle, *.exe, *.apk)
    ↓
structural check (Unity: Data/Managed dir, GameAssembly.dll)
    ↓
ranked handler list with confidence scores
    ↓
auto-pick highest > 0.8; else show chooser
```

---

## Directory Layout (target)

```
Unwrap/
├── src-tauri/            # Rust backend, Tauri config
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/     # Tauri IPC commands
│   │   ├── handlers/     # FormatHandler implementations
│   │   │   └── unity/    # Unity-specific
│   │   ├── sidecar/      # Subprocess wrappers (AssetRipper, ILSpy)
│   │   ├── detection/    # Format detection pipeline
│   │   ├── project/      # Workspace/project model
│   │   └── db/           # SQLite project cache
│   ├── bin/              # Sidecar binaries (AssetRipper, ILSpyCmd, etc.)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                  # React frontend
│   ├── app/              # TanStack Router routes
│   ├── components/       # Reusable UI
│   ├── features/         # Domain features (asset-tree, preview, decompile)
│   ├── hooks/
│   ├── stores/           # Zustand
│   ├── lib/              # Tauri command wrappers
│   └── styles/
├── docs/                 # Documentation
├── plans/                # Implementation plans
└── package.json
```

---

## Non-Goals (Explicit)

- ❌ Cross-platform Linux/macOS in v1 (Windows only)
- ❌ Editing/repacking assets in v1 (read-only viewer first)
- ❌ Plugin marketplace in v1
- ❌ Cloud sync, collaboration
- ❌ Real-time game memory hooking (this is static analysis)
- ❌ DRM circumvention (legal scope: own-asset extraction, modding, translation, education)

---

## Risk Register

| Risk | Mitigation |
|------|------------|
| AssetRipper GPL infection | Subprocess only, never link library — verified safe pattern |
| WebView2 not installed on older Windows | Auto-installer bootstrapper (Tauri default) |
| Large asset trees crash UI | Virtualized list (TanStack Virtual), lazy load |
| IL2CPP decompile accuracy low | Document as known limitation; surface Cpp2IL path in v2 |
| Sidecar binary size bloats installer | Lazy-download on first IL2CPP detection |

---

## Unresolved Questions

- Should bundle Python runtime for UnityPy fallback? (Decision: NO for v1; revisit if AssetRipper has gaps)
- Final app name (working: Unwrap). Brainstorm during design phase.
- ILSpyCmd vs ILSpyCommand.NetCore — pick latest stable build
- Should asset cache persist across sessions or rebuild on open? (Decision: persist, SQLite-keyed by file hash)
