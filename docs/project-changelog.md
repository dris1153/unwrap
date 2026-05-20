# Changelog

All notable changes to Unwrap are documented here. Format: [Keep a Changelog](https://keepachangelog.com).

## [Unreleased]

### Security

- `sidecar::manifest::verify_sha256` and `verify_sha256_bytes` now **refuse** to short-circuit on placeholder `"TBD"` or empty-string checksums. Previously, these returned `Ok(true)`, allowing tools shipped with `sha256: "TBD"` in `sidecar-manifest.json` to bypass integrity verification entirely. The lazy-download path (Il2CppDumper) is the primary affected surface. Test `sha256_bytes_placeholder_refused` updated to assert the new fail-closed behavior.
- Code-review P1 fix tracked in `plans/20260520-1530-unwrap-mvp-bootstrap/reports/code-review-final.md` finding #1.

### Added

- Native folder picker via Browse files button (Tauri dialog plugin v2.7.1) — respects user's file explorer selection on Windows
- Ctrl/Cmd+O keyboard shortcut for opening/browsing projects
- Loading skeleton + error state UI for recent projects grid
- Empty state card when no recent projects in database ("No recent projects yet — drop a Unity build folder to start")

### Fixed

- Window minimize, maximize, close buttons now respond to clicks (Tauri 2 capability permissions: `core:window:allow-minimize`, `core:window:allow-toggle-maximize`, `core:window:allow-close`)
- Custom title bar now draggable (added `core:window:allow-start-dragging` permission)
- Recent projects grid now reads real `list_recents()` IPC endpoint from SQLite (was mock data)

### Changed

- Open archive (.zip) button now displays "Coming in v0.2 — archive extraction" tooltip in disabled state (deferred to v0.2)

### Deprecated

- `MOCK_RECENTS` constant in `src/lib/mock-recents.ts` (kept for storybook/test reuse; see recent-projects-grid.tsx for real data flow)

### Known issues (v0.1.x track)

- `SidecarManager::kill()` is a no-op because spawned PIDs are stored as `0` placeholders (`src-tauri/src/sidecar/mod.rs:50,74,106`). Frontend `cancel` IPC and app-shutdown cleanup do nothing for in-flight sidecars. Partial mitigation: `kill_on_drop(true)` on `Child` fires on Drop, but the `Child` is moved into a tokio task in `spawn.rs:90` and survives `cleanup_all`. Tracked as code-review P1 finding #2.
- `read_file_chunk` accepts any user-readable path (no cache-dir prefix allowlist). Defense-in-depth, not exploit-grade for a local-only tool — code-review P2 finding #3.

## [0.1.0] — 2026-05-20

### Added

- **Phase 10: Testing & Docs (v0.1 MVP complete)**
  - Vitest setup: jsdom environment, `@testing-library/react`, Tauri API stubs in `src/test-setup.ts`
  - 51 Vitest tests across 5 test files:
    - `use-flattened-tree.test.ts` — 10 tests covering collapse, expand, filter, depth, hasChildren
    - `use-tree-store.test.ts` — 10 tests covering toggleExpanded, setSelected, isolation, clearProject
    - `use-tab-store.test.ts` — 11 tests covering openTab, closeTab neighbor selection, closeAllForProject
    - `use-translate-store.test.ts` — 11 tests covering locale mutators, setFilter, updateDraft, clearDraft
    - `built-in-commands.test.ts` — 9 tests covering filterBuiltInCommands case-insensitive matching
  - 5 new Rust tests in `db::repo::translations` (inline `#[cfg(test)]`): bulk insert round-trip, idempotent conflict, upsert + retrieve, update on conflict, filter narrows results
  - `LICENSE` — MIT license, copyright 2026 Unwrap Project
  - `BUILD.md` — contributor setup guide: prereqs, sidecar acquisition, dev/test/build, troubleshooting
  - `docs/keyboard-shortcuts.md` — full shortcut reference for v0.1
  - `docs/qa-checklist.md` — manual QA checklist against all 4 wireframes + installed MSI smoke test
  - `.github/workflows/ci.yml` — CI pipeline: lint-and-typecheck, test, build jobs on `windows-latest`
  - `bundle.resources` in `tauri.conf.json` — ensures `NOTICE.txt` ships in MSI at `%PROGRAMFILES%\Unwrap\`
  - `README.md` updated to reflect v0.1 MVP completion, full stack, prereqs, test and build commands

### Verification (phase 10)

- `pnpm tsc --noEmit` — Pass
- `pnpm build` — Pass
- `pnpm test` — Pass (51/51 Vitest tests)
- `cargo test` — Pass (84 unit + 5 integration tests; 1 ignored: requires real Unity build)
- `cargo clippy --all-targets -- -D warnings` — Pass (zero warnings)

## [0.1.0-bootstrap] — 2026-05-20

### Added

- **Project bootstrap pipeline** — Executed via `/ck:bootstrap --auto` with integrated research, design, and planning phases
- **Phase 01: Project Scaffolding**
  - Tauri 2 desktop scaffold with React 18, TypeScript, Vite, Tailwind CSS 4
  - Custom Tauri title bar with Windows window controls (minimize/maximize/close)
  - Brand assets: Stack mark SVG (emerald #10B981), Satoshi 900 + JetBrains Mono self-hosted fonts, generated favicon
  - Rust toolchain locked to MSVC (Windows): `rustup default stable-x86_64-pc-windows-msvc`

- **Phase 02: Design System & App Shell**
  - Full design token system in `globals.css` with Tailwind @theme block
  - UI primitive components: Button, Input, Tooltip, ScrollArea, Separator, Kbd, StatusDot
  - App shell layout: TitleBar, Toolbar, Sidebar (collapsible, width-persist), Inspector (collapsible), StatusBar
  - Welcome route (`/`) pixel-matched to wireframe with mock recents grid (6 recent projects)
  - Empty Project route (`/project/$projectId`) shell scaffolded
  - TanStack Router v1 with file-based routing and auto-generated route tree
  - Zustand state store with persist middleware (sidebar/inspector widths, active rail tab)
  - One icon substitution applied: FolderNotchOpen → FolderSimplePlus (Phosphor v2 version mismatch)

- **Phase 03: Rust Backend Core**
  - Core domain models: Project, AssetTree, AssetNode, PreviewPayload enums
  - FormatHandler async trait + FormatHandlerRegistry for extensible handler system
  - Detection pipeline with parallel `detect()` fanout across all registered handlers + confidence ranking
  - Magic-byte signatures: UnityFS, PE/MZ (Windows .exe), ELF (Linux), PK/zip
  - SQLite v1 schema with 3 tables: `projects`, `recents`, `asset_trees` (rusqlite-bundled)
  - SidecarManager: spawn/kill with timeout, progress event streaming via mpsc, JsonLines stderr parser
  - 8 Tauri IPC commands registered:
    - `open_project` (real: opens folder, detects format, returns project ID)
    - `project_info` (real: returns cached project metadata)
    - `tree` (stub → NotImplemented)
    - `preview` (stub → NotImplemented)
    - `export` (stub → NotImplemented)
    - `search` (stub → NotImplemented)
    - `decompile` (stub → NotImplemented)
    - `recent_projects` (real: returns cached recents from SQLite)
  - AppError enum with Display + Serialize impl (thiserror trait derive)
  - TypeScript IPC wrappers and domain type mirrors in `src/lib/ipc.ts` and `src/lib/types.ts`
  - 28 Rust source files across modular structure: domain/, handlers/, detection/, sidecar/, db/, commands/, events/
  - 2 TypeScript utility modules: `lib/ipc.ts` (command invocation), `lib/types.ts` (domain models)

### Verification

- `pnpm tsc --noEmit` — Pass (TypeScript strict mode)
- `pnpm build` — Pass (Vite + TypeScript)
- `cargo check` — Pass
- `cargo clippy --all-targets -- -D warnings` — Pass (zero warnings)
- `cargo test` — Pass (7/7 unit tests: 5 magic-byte detection + 2 progress parser)

### Changed

- Rust default toolchain switched from GNU to MSVC during phase-01 (required for Tauri on Windows)

### Notes

- Phases 04–10 remain in plan; next cook run will execute phase-04 (Sidecar Tools Integration)
- Design system locked; only minor token tweaks expected in future phases
- Wireframes 01–04 all have matching scaffolding; full implementation pending phase-06 (Asset Browse & Preview)
