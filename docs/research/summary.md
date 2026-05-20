# Research Summary — Unwrap Bootstrap

Synthesis of 4 parallel research reports.

---

## 1. Unity RE Ecosystem (researcher-01)

**Winners (by license + capability):**
- **AssetRipper** (GPL-3.0) — best asset coverage, subprocess only
- **ILSpy** (MIT) — Mono .NET decompiler, embeddable
- **Il2CppDumper** (MIT) — IL2CPP→dummy DLLs, subprocess
- **Cpp2IL** (MIT, experimental) — better IL2CPP, Phase 2

**Avoid embedding:** AssetRipper, dnSpy (both GPL viral).
**Defer:** UnityPy (Python runtime bundling cost > value for v1).

**Mono vs IL2CPP reality:**
- Mono: ~95% accurate C# recovery (DLL → ILSpy)
- IL2CPP: ~70-85% pseudo-C# (Il2CppDumper → ILSpy)

## 2. Desktop Framework (researcher-02)

**Decision: Tauri 2**

| Metric | Tauri 2 | Electron | Avalonia | WPF |
|--------|---------|----------|----------|-----|
| Bundle | 8-10MB | ~150MB | ~70MB | bundled w/ .NET |
| Startup | 1.4s | 3.2s | ~2s | ~1.5s |
| Cross-plat | yes | yes | yes | no |
| Modern UI ease | high (web) | high | medium | low |
| AssetRipper native? | sidecar | sidecar | in-process | in-process |

Tauri wins on bundle + startup + DX. Sidecar pattern handles .NET tool integration cleanly.

## 3. UI/UX Patterns (researcher-03)

**Layout consensus across RE tools:**
- Left sidebar (file/asset tree)
- Center editor (tabs, splits, preview)
- Right inspector (collapsible)
- No modal dialogs — use pinned panels

**Steal from VS Code / Linear / Rider:**
- Dark first (#0D1117 family)
- JetBrains Mono for code, Inter for UI
- Shadows for depth, not borders
- Cmd+K fuzzy palette for jump-to-symbol
- Drag-drop as primary entry; recent files grid below

**Asset-specific:**
- Image: zoom/pan + checker bg + histogram
- Audio: waveform scrubber
- 3D: orbit controls + grid
- Text: side-by-side translate column

## 4. Plugin Architecture (researcher-04)

**Decision: Hard-coded Unity v1 + `IFormatHandler` Rust trait designed now**

- v1: Subprocess CLI pattern for external tools + internal Unity handler
- v2: Same trait exposed for script plugins (Python/JS via JSON-RPC)
- v3: Marketplace, signed plugins, WASM sandbox

**Detection: multi-stage fallback** (magic bytes → extension → structural → user prompt)

**Subprocess best practices:**
- JSON stdout for progress/results
- `CancellationToken` → process kill
- 5-min default timeout, configurable
- Trap exit code + stderr for context

---

## Consolidated MVP Scope

1. Tauri 2 + React + TypeScript + Tailwind/shadcn shell
2. Drag-drop a Unity build folder → AssetRipper extracts → show tree
3. Asset previews: image, audio, 3D model, text
4. Decompile C# scripts (Mono via ILSpy direct; IL2CPP via Il2CppDumper + ILSpy)
5. SQLite project cache (asset tree, recents)
6. Cmd+K fuzzy search across project
7. Read-only viewer (edit/repack deferred to v2)

## Open Questions (for design + planning phases)

- Final app name? (working: Unwrap)
- Asset cache TTL / location preference?
- Should support drop-by-archive-file (.zip of game) in v1 or v2?
- Plugin SDK shape for v2 — JSON-RPC vs WASM?
