# Changelog

All notable changes to Unwrap are documented here. Format: [Keep a Changelog](https://keepachangelog.com).

## [Unreleased]

### Fixed (phase 01 — indexing fix)

- **Real Unity builds now extract correctly.** Opening `D:\Applications\Steam\steamapps\common\Ladies Dont Tempt My Immortality` (and any other genuine Unity Steam build) previously produced an empty asset tree with StatusBar lying "Indexed". Five compounding defects converged into a silent no-op:
  1. **Bundled AssetRipper-x86_64-pc-windows-msvc.exe is `AssetRipper.GUI.Web` (ASP.NET web app), not a CLI extractor.** The old `asset_ripper.rs` passed `<path> -o <output> --headless` — unknown args ignored, binary started a web server, idled until 10-minute timeout. Now the wrapper spawns the binary on a random local port and drives its HTTP REST endpoints (`POST /LoadFolder`, `GET /Collections/Count`, `POST /Export/UnityProject`, `POST /Reset`) over a `reqwest::Client`. API surface documented in `plans/20260521-0900-v0.1.1-indexing-fix-and-multi-engine-guard/research-assetripper-api.md`.
  2. **Sidecar stderr was dropped** at `src-tauri/src/sidecar/spawn.rs:70` (`Stdio::null()`). Now piped and merged into the same channel as stdout with `[stderr]` prefix. mpsc channel cap raised 64 → 512 to absorb stderr bursts. Benefits ILSpyCmd and Il2CppDumper too.
  3. **`tree_builder::walk` walked the wrong directory.** AssetRipper writes to `<output>/ExportedProject/Assets/…` but the handler walked the parent. Now `extract()` resolves the actual output dir (prefers `ExportedProject` subfolder; falls back to root only when the export root has > 5 entries; errors out on empty) and `unity/mod.rs:184` walks `report.output_dir`.
  4. **`assets_count == 0` no longer treated as success.** `extract()` returns `Err(SidecarFailed)` when `/Collections/Count` returns 0 (load failure) or when the export produces no files (`resolve_output_dir → None`).
  5. **StatusBar no longer hard-codes `indexState="indexed"`.** `src/app/project/$projectId.tsx` now derives the state from real tree size: `indexing` while query loading, `indexed` when tree has > 1 node, `empty` otherwise. New `empty` variant rendered with red `Warning` icon and label "No assets indexed — see logs". `indexing` variant renders an animated `Spinner` with "Indexing…" label.
- AssetRipper diagnostic log persisted to `%LOCALAPPDATA%\Unwrap\logs\<op_id>-asset-ripper.log` via the binary's own `--log --log-path` flags, so failed extractions are debuggable without rerunning.
- Windows long-path UNC `\\?\` prefix stripped before sending paths to AssetRipper's `Path=` form field (avoids ".NET file-not-found" errors on canonicalized Steam paths).

### Fixed (phase 01b — post-QA bugfix bundle)

- **AssetRipper no longer triggers Tauri dev rebuild loop.** The spawned process inherited the parent CWD (`src-tauri/` during `tauri dev`) and wrote session/temp files to `src-tauri/binaries/temp/<uuid>/`. Tauri's file-watcher detected those new files and fired `cargo run` rebuilds, killing the running app mid-extraction. Fix: scope the spawn's CWD to a per-operation directory under the user cache root (`%LOCALAPPDATA%\Unwrap\…\sidecar-workdir\<op_id>`). Defense-in-depth: added `src-tauri/binaries/*.log` and `src-tauri/binaries/temp` to `.gitignore`.
- **Unity 6 ggm headers no longer panic the version parser.** `parse_engine_version_from_ggm`'s fallback `scan_for_version` was converting raw bytes through `String::from_utf8_lossy` then byte-indexing the resulting `str`. Invalid binary bytes became 3-byte `U+FFFD` replacement chars and `&text[start..start + 24]` panicked when the upper bound landed inside one (`"end byte index 24 is not a char boundary; it is inside '�' (bytes 23..26) of '�L��6000.2.6f2'"`). Rewrote the scanner to walk raw `&[u8]` collecting ASCII version-shape runs `[0-9A-Za-z.]` length 5..=32 and validating each via the existing `looks_like_unity_version` predicate. No UTF-8 lossy conversion, no panic surface. 3 new regression tests added (`scan_for_version_unity6_with_garbage_prefix`, `scan_for_version_returns_none_on_garbage_only`, `scan_for_version_rejects_oversized_run`).
- **Cache no longer remembers empty extractions forever.** The cache-hit path returned the cached tree even when it had only the root node (a sign of a previous failed extraction). Now read-side validates `cached_tree.nodes.len() > 1` before returning — if not, deletes the bad row and falls through to re-extract. Write-side mirrors the check: extractions producing a single-node tree are no longer persisted (only `projects::upsert` and `recents::upsert` run). **One-time manual step for existing dev installs:** delete `%LOCALAPPDATA%\Unwrap\Unwrap\data\cache.db` before relaunching to flush previously-poisoned entries (e.g. the user's `Ladies Dont Tempt My Immortality` cache).
- 4 new tests added (3 version regression + 1 sidecar workdir isolation); 89/89 Rust lib tests pass.

### Fixed (phase 01c — IL2CPP path + error UX)

- **IL2CPP support now works end-to-end.** Phase-01b manual QA revealed both test games (`Ladies Dont Tempt My Immortality` and `Leaf it Alone`) are IL2CPP — not Mono as the StatusBar appeared to show. Six compounding bugs blocked the IL2CPP code path from ever succeeding:
  - `sidecar-manifest.json` pointed at non-existent `Il2CppDumper v6.7.42` (latest stable is v6.7.46, released Jul 2024). HTTP 404 on every download attempt.
  - Filename pattern was wrong (`Il2CppDumper-net6.0-x64-v6.7.42.zip`); real Perfare releases use `Il2CppDumper-{framework}-win-v{ver}.zip` format.
  - `sha256` was the `"TBD"` placeholder; phase-01 security hardening (rightly) refused to verify, so even a correct URL would have failed at integrity check.
  - `installer::extract_zip` only extracted ONE named entry per call. Il2CppDumper.exe requires `config.json` next to it at runtime; the missing config caused .NET startup failure on every dump attempt.
  - Frontend silently swallowed IPC errors at [drop-zone.tsx:91](../src/features/welcome/drop-zone.tsx#L91) — `console.error` + `setOpening(false)` with no user-visible toast. User saw modal stuck at "preparing / 0% / Starting…" then auto-dismiss.
  - StatusBar's `backendType === "il2cpp" ? "il2cpp" : "mono"` ternary at [$projectId.tsx:51](../src/app/project/$projectId.tsx#L51) collapsed three backend states into two; any non-`il2cpp` value (including `unknown` from a failed detection) rendered as "Mono". This caused all initial user confusion: the games WERE IL2CPP, but the display lied.

  **Fixes shipped together:**
  - Manifest updated to Il2CppDumper `v6.7.46`, URL `Il2CppDumper-win-v6.7.46.zip` (.NET Framework 4.x variant — 11.2 MB, runs natively on Windows 10/11 with zero runtime install). SHA-256 `f5fc60dfc5c034c1ff3fc651514416ebd47658a63493207734537fd25fa2dea2` verified against actual GitHub release.
  - New optional manifest field `extra_files: Vec<String>` with `#[serde(default)]`. For `il2cpp-dumper` set to `["config.json"]`. Other tools (asset-ripper, ilspycmd) inherit empty default — no behavior change.
  - `installer::extract_zip_files(zip_bytes, dest_dir, entry_names: &[String]) -> Vec<PathBuf>` opens the archive once and extracts multiple named entries by basename or path-suffix match. Returns `Err` if any requested entry is missing — partial extraction treated as failure so half-installed sidecars don't slip past verification. `extract_zip` retained as backward-compatible single-entry wrapper. `ensure_installed` now extracts `install_subpath + extra_files` and verifies all expected paths exist before returning success.
  - `<ErrorToast />` (new) mounted at app root via [src/app/__root.tsx](../src/app/__root.tsx). Single-error zustand store at [src/stores/use-error-store.ts](../src/stores/use-error-store.ts). Auto-dismisses after 10 s, manual X button, optional "Copy log path" button for `SIDECAR_FAILED`-coded errors. Drop-zone catch parses `AppError` shape via new `parseAppError` helper in `ipc.ts` and surfaces actionable messages instead of silent dismiss.
  - StatusBar grows new `"unknown"` `backendType` variant — gray dot + "Unknown backend" label. `$projectId.tsx` maps `scripting_backend` explicitly to all three states; collapsing into Mono fallback is gone.
  - 3 new tests (2 Rust for `extract_zip_files` happy/missing-entry paths; 1 Vitest for ErrorToast render + auto-dismiss + manual close + copy-path button). Total 91/91 Rust lib + 52/52 Vitest.

### Fixed (phase 01d — Il2CppDumper ReadKey hang)

- **Il2CppDumper no longer triggers the 5-minute sidecar timeout.** Phase-01c manual QA showed the install + extract + dump pipeline succeeded end-to-end in ~5 seconds, then Il2CppDumper printed `"Press any key to exit..."` and hung on `Console.ReadKey()` waiting for keyboard input that never comes when invoked headless. After 5 minutes our `sidecar/spawn.rs` timeout fired, killed the process with exit code `-2`, and the wrapper returned `SidecarMsg::Error("exit code -2")` — upstream treated the successful dump as a failure and silently dismissed the open-project modal. Fix: `spawn.rs` stdout reader now matches against a small list of console-hang signatures (`"press any key to exit"`, `"press any key to continue"`, case-insensitive) and when detected gracefully kills the child + treats the run as `Done`. Safe globally — none of our other sidecars (AssetRipper, ILSpyCmd) print those patterns. 2 new unit tests added (positive + negative pattern matching).

### Fixed (phase 01e — bundled binary cache)

- **AssetRipper no longer triggers Tauri dev rebuild loop (root cause, take two).** Phase-01b's `current_dir` fix turned out to be insufficient — ASP.NET Core uses `AppContext.BaseDirectory` (the directory containing the executable) for its content root and Razor compilation cache, NOT the OS-process current working directory. AssetRipper was therefore still writing temp files into `src-tauri/binaries/temp/<guid>/` regardless of our CWD setting. Tauri's CLI file-watcher saw those writes and triggered `cargo run` rebuilds, killing the running app mid-extraction.
- **Production crash prevented.** Same exe-dir-relative writes would crash AssetRipper in installed MSI builds because `%PROGRAMFILES%\Unwrap\binaries\` is read-only for non-admin users. Phase-01e fixes both the dev rebuild loop AND the prod read-only crash by copying bundled binaries into the user cache before spawning.
- **New `installer::ensure_bundled_cached(tool_id, ctx)`** helper copies any `bundled: true` manifest entry from its source location (`src-tauri/binaries/` in dev, `<exe>/binaries/` in prod) into `<cache_dir>/bin/` once. Cache hit is decided by file-size match — the manifest's sha256 isn't usable here because the dev path for ILSpyCmd ships the dotnet-tool wrapper which intentionally differs from the production-wrapper sha. Subsequent spawns reuse the cached copy. ~50 MB one-time copy for AssetRipper, ~5 MB for ILSpyCmd.
- **Applied to both bundled sidecars**: `sidecar::tools::asset_ripper::extract` and `sidecar::tools::ilspy::decompile` now resolve their binary via `ensure_bundled_cached` instead of running directly from the source/install tree. Dead `resolve_binary_path` helper in `asset_ripper.rs` removed.
- 2 new tests added in `installer.rs` (cache-or-copy on first call; cache hit when file-size matches). Total Rust lib tests 93 → 95.
- Bug M (manifest sha mismatch on re-run, seen in phase-01d QA logs) is auto-resolved by this change: the bundled binary now sits unchanged in source and is read-only-copied into cache, so post-spawn modifications no longer drift the cached copy out of sync.

### Known issues carrying into v0.1.2

- Manifest verifier logs `WARN ... bundled sidecar binary is missing path=target\debug\binaries\...` on dev startup. Cosmetic only — `installer::resolve_bundled_source` now handles the dev location correctly, but the startup verifier still checks the wrong path. Fix scheduled for v0.1.2 when manifest path resolution is next touched.
- `LoadingModal` displays "0%" instead of the actual emitted progress percent (e.g. `dumping_il2cpp` at 10%) during long IL2CPP install + dump phases. Likely a subscribe-too-late race or a phase-string mapping gap in the frontend. Deferred to phase-02 alongside other UX polish (toast i18n, ComingSoon dropdown).
- `LoadingModal` displays "0%" instead of the actual emitted progress percent (e.g. `dumping_il2cpp` at 10%) during long IL2CPP install + dump phases. Likely a subscribe-too-late race or a phase-string mapping gap in the frontend. Deferred to phase-02 alongside other UX polish (toast i18n, ComingSoon dropdown).

### Security

- `sidecar::manifest::verify_sha256` and `verify_sha256_bytes` now **refuse** to short-circuit on placeholder `"TBD"` or empty-string checksums. Previously, these returned `Ok(true)`, allowing tools shipped with `sha256: "TBD"` in `sidecar-manifest.json` to bypass integrity verification entirely. The lazy-download path (Il2CppDumper) is the primary affected surface. Test `sha256_bytes_placeholder_refused` updated to assert the new fail-closed behavior.
- Code-review P1 fix tracked in `plans/20260520-1530-unwrap-mvp-bootstrap/reports/code-review-final.md` finding #1.

### Added

- **Multi-language documentation page** at `docs.html` (root)
  - Languages: English (authoritative), Vietnamese, Simplified Chinese — translations marked machine-assisted in `meta.note`
  - 11 sections: intro, features, architecture, workflows, how-to-use, tech stack, keyboard shortcuts, bundled tools, troubleshooting, license + hero
  - 5 hand-coded SVG diagrams (architecture 3-layer, 3 workflows, plugin architecture)
  - 4 wireframe screenshots embedded
  - npm script `pnpm docs:serve` for local serve at http://localhost:5173/docs.html
  - Vanilla JS i18n loader (~150 LOC functional, no dependencies)
  - Fetch-based JSON loading; in-page error banner with serve instructions if opened via file://
  - Brand-aligned with Unwrap app (emerald accent, Satoshi + JetBrains Mono, zinc-950 dark)
- Native folder picker via Browse files button (Tauri dialog plugin v2.7.1) — respects user's file explorer selection on Windows
- Ctrl/Cmd+O keyboard shortcut for opening/browsing projects
- Loading skeleton + error state UI for recent projects grid
- Empty state card when no recent projects in database ("No recent projects yet — drop a Unity build folder to start")

### Fixed

- `pnpm docs:serve` initially used `-s` (SPA mode) flag which caused `/docs.html` to redirect to `/docs` then return Vite `index.html`. Removed `-s`; docs.html now serves correctly.
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
