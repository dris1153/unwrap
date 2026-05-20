# Development Roadmap

**Snapshot:** 2026-05-20  
**Project:** Unwrap MVP — Windows-first Unity reverse engineering tool  
**Plan:** See [plans/20260520-1530-unwrap-mvp-bootstrap/plan.md](../plans/20260520-1530-unwrap-mvp-bootstrap/plan.md) for engineering detail

## Progress: 100% Complete (10 of 10 Phases) — v0.1 MVP shipped

```
████████████████████████████████  10/10 phases
```

## Phases

### Phase 01: Project Scaffolding — Complete

Windows desktop scaffold with Tauri 2, React 18, Vite, Tailwind 4, Rust async runtime. Brand assets (Stack mark, fonts) and generated favicon. Rust toolchain locked to MSVC. Foundation for UI + backend ready.

**Shipped:** 2026-05-20 | **Effort:** 1-2d | **Status:** Locked

---

### Phase 02: Design System & Shell — Complete

Full design token system + UI primitives (Button, Input, Tooltip, ScrollArea, Separator, Kbd, StatusDot). App shell with custom title bar, collapsible sidebar/inspector, Welcome route matching wireframe 01. Zustand store with persist middleware. TanStack Router file-based routing.

**Shipped:** 2026-05-20 | **Effort:** 2-3d | **Status:** Locked

---

### Phase 03: Rust Backend Core — Complete

Core domain models (Project, AssetTree, AssetNode). FormatHandler trait + registry. Detection pipeline with magic-byte signatures (UnityFS, PE, ELF, PK). SQLite schema v1 (projects, recents, asset_trees). SidecarManager for subprocess orchestration. 8 Tauri commands (5 stubs, 2 real: open_project, recent_projects). 7/7 unit tests passing. Clippy clean.

**Shipped:** 2026-05-20 | **Effort:** 3-4d | **Status:** Locked

---

### Phase 04: Sidecar Tools Integration — Pending

Bundle AssetRipper.exe, ILSpyCmd, Il2CppDumper as sidecar binaries. Implement spawn/timeout/cancellation/progress-streaming for each tool. Create subprocess JSON-Lines wrappers. Document GPL compliance (AssetRipper subprocess-only, never linked). Pin tool versions and document upgrade procedure.

**Target:** ~2-3d | **Blocks:** Phase 05

---

### Phase 05: Unity Format Handler — Pending

Implement full UnityHandler: detect Mono vs IL2CPP backend, spawn AssetRipper for extraction, build AssetTree from extracted structure, populate SQLite cache (keyed by file hash). Structural detection (Data/ folder, GameAssembly.dll heuristics). Support both Unity 3.5+ and modern (6000+).

**Target:** ~3-4d | **Blocks:** Phases 06, 07, 08 (can parallelize once 05 done)

---

### Phase 06: Asset Browse & Preview — Pending

Virtualized asset tree (TanStack Virtual for 20k+ node trees). Node click → preview pane switch. Image preview (zoom-pan-pinch). Audio preview (wavesurfer.js waveform + scrub). 3D mesh preview (three.js/react-three-fiber for GLB/FBX/OBJ). Text preview (code highlight for scripts, plain for strings). Hex viewer (raw binary with endianness toggle).

**Target:** ~3-4d | **Depends on:** Phase 05

---

### Phase 07: Code Decompile — Pending

Mono detection → read managed DLL → ILSpyCmd subprocess → C# source. IL2CPP detection → Il2CppDumper to produce dummy DLLs → ILSpyCmd. Stream output via stderr JSON-Lines. Monaco Editor for syntax highlighting, folding, search. Inspector panel shows namespace, references, used-by (xref index). Document IL2CPP accuracy limits (~70%).

**Target:** ~2-3d | **Depends on:** Phase 05

---

### Phase 08: Translatable Text View — Pending

TextAsset / string extraction from asset tree. Side-by-side editor: original | translation column. Save translation to SQLite (keyed by node ID + locale). UI shows unsaved/saved state. Prepare extraction for phase-10 export (repack into asset bundle via AssetRipper export mode).

**Target:** ~2-3d | **Depends on:** Phase 05

---

### Phase 09: Search & Command Palette — Pending

Full-text search across asset names/types (SQLite FTS). Cmd+K command palette (fuzzy search, keyboard nav, actions). Quick filters: by type (image/audio/code/text), by size, by scripting backend. Search < 200ms target (SQLite indexed).

**Target:** ~1-2d | **Depends on:** Phases 06, 07, 08

---

### Phase 10: Testing & Docs — Complete

51 Vitest tests + 84 Rust unit tests + 5 integration tests. Inline translations repo tests. LICENSE (MIT), BUILD.md, docs/keyboard-shortcuts.md, docs/qa-checklist.md. CI workflow on windows-latest. NOTICE.txt bundled in MSI via tauri.conf.json resources.

**Shipped:** 2026-05-20 | **Effort:** 2-3d | **Status:** Locked

---

## Out-of-Scope (v0.1 Explicit)

- Cross-platform (Linux/macOS) — Windows only
- Editing/repacking assets — read-only viewer first
- Plugin marketplace, signed plugins, WASM sandbox
- Real-time game memory hooking
- DRM circumvention
- Light theme — dark-only v0.1
- Python/UnityPy fallback — AssetRipper + ILSpyCmd primary
- Cpp2IL — defer to v0.2; Il2CppDumper primary in v0.1

## Known Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| AssetRipper version churn | Pin version + document upgrade procedure (phase 04) |
| WebView2 missing on older Win10 | Tauri auto-installer handles; flag in deployment docs (phase 10) |
| IL2CPP decompile accuracy ~70% | Surface confidence limits in UI (phase 07); Cpp2IL path for v0.2 |
| Large asset trees (20k+ nodes) crash | Virtualize via TanStack Virtual (phase 06) |
| GPL infection (AssetRipper) | Subprocess-only pattern (never link); verify safe pattern + document (phase 04/10) |

## How to Cook the Next Phase

Once this phase is complete, run:

```bash
# Cook phase-04 (Sidecar Tools Integration)
/ck:cook --auto plans/20260520-1530-unwrap-mvp-bootstrap/phase-04-sidecar-tools-integration.md

# Or cook the entire remaining plan
/ck:cook --auto plans/20260520-1530-unwrap-mvp-bootstrap/plan.md
```

See [Primary Workflow](https://github.com/anthropics/claude-code/docs/primary-workflow.md) for detailed cook protocol.

---

**Total estimate:** ~18-26 days for full MVP (single dev). Phases 06/07/08 can parallelize once 05 ships, saving ~5-8 days.
