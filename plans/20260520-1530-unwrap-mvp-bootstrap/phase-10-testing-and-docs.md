# Phase 10: Testing & Docs

**Status:** Complete (2026-05-20)
**Priority:** High
**Effort:** M (2-3d)
**Depends on:** phase-09-search-and-command-palette (all earlier phases must be complete)

## Context Links
- Tech: `docs/tech-stack.md` (Risk Register; non-goals)
- Arch: `docs/system-architecture.md` (Performance Budget)
- All wireframes in `docs/wireframe/` (manual QA targets)

## Overview
Solidify the MVP: write Rust unit tests for handlers + format detection, integration tests against a Unity fixture, Vitest tests for critical React logic, draft README + BUILD.md + LICENSE/NOTICE, set up CI for compile+test+lint, manual QA pass against all 4 wireframes, ship a Windows MSI installer.

## Key Insights
- Tests cover the behaviors that protect MVP shipping: format detection correctness, cache key stability, decompile path selection, IPC contract serialization, virtualized tree perf. Skip exhaustive coverage of UI styling — wireframes are the ground truth, eyeball it.
- The fixture issue (need an actual Unity build for integration tests) was acknowledged in phase-05 — finalize sourcing here.
- BUILD.md is the source of truth for "how to set up dev from scratch" — including WebView2, .NET 8 runtime for ILSpyCmd dependency, sidecar acquisition.
- README.md is user-facing — value prop, screenshots from wireframes, supported Unity versions, install link.
- LICENSE: pick MIT for the Unwrap app itself; NOTICE.txt enumerates bundled tool licenses (already drafted in phase-04).
- CI: GitHub Actions on `windows-latest` runner; cache cargo + pnpm; gate merges on `cargo test`, `cargo clippy`, `pnpm tsc`, `pnpm test`.

## Requirements

### Functional
1. Rust unit tests cover (with assertions):
   - `detection::magic` signature scans (UnityFS, MZ/PE, PK/zip).
   - `handlers::unity::cache_key` stability (same input → same id; canonicalized path).
   - `handlers::unity::detect_unity` (Mono vs IL2CPP confidence scoring).
   - `handlers::unity::kind_table::ext_to_kind` (lookup table correctness).
   - `handlers::unity::version::parse_engine_version_from_ggm` (sanity on 3 mock binaries).
   - `sidecar::progress::JsonLines` parser (parse / fallback / partial line).
   - `db::repo::translations` save + retrieve round trip.
2. Rust integration tests against fixture (`tests/handlers_unity.rs`):
   - Open project end-to-end (drop + extract + tree built).
   - Cache hit on second open (<300ms).
   - Preview Image (returns dimensions).
   - Decompile DLL (returns content + correct confidence).
   - Search query returns expected results.
3. Vitest tests for frontend critical paths:
   - `useFlattenedTree` memo correctness (expanded toggles produce correct order).
   - `useTreeStore` / `useTabStore` / `useTranslateStore` reducers (Zustand action behavior).
   - `ipc.ts` typed wrappers compile-check vs Rust types.
   - Palette result ranking (mock data → expected order).
4. Documentation:
   - `README.md` (root, user-facing).
   - `BUILD.md` (root, contributor-facing).
   - `LICENSE` (root, MIT).
   - `NOTICE.txt` (in `src-tauri/` and packaged to installer, drafted in phase-04 — finalize text + verify shipped path).
   - `docs/keyboard-shortcuts.md` (quick reference).
5. CI workflow `.github/workflows/ci.yml`:
   - Trigger: push to any branch + PR.
   - Jobs: lint (clippy, prettier), typecheck (tsc), test (cargo test, pnpm test), build (`pnpm tauri build` on windows-latest).
6. Manual QA checklist against 4 wireframes:
   - `01-welcome.html`: drop zone visual, recents grid, status bar, drag-drop interaction.
   - `02-asset-preview.html`: tree, tab bar, image preview, inspector, filmstrip.
   - `03-code-decompile.html`: Monaco editor, syntax colors, outline, backend badge, confidence badge.
   - `04-translate.html`: locale picker, progress bar, side-by-side editor, status dots, autosave.
7. Release artifact: `Unwrap-0.1.0-x64.msi` via `pnpm tauri build`. Smoke-tested on a clean Windows VM.

### Non-Functional
- CI completes in <12min on cached runner.
- Test suite (Rust unit + Vitest) runs in <60s.
- README readable in <3 minutes; screenshots visible.
- LICENSE + NOTICE included in installer at `%PROGRAMFILES%\Unwrap\`.

## Architecture / Approach

**Test layout:**

```
src-tauri/
├── src/
│   └── (production code with `#[cfg(test)] mod tests { ... }` inline)
└── tests/
    ├── fixtures/
    │   ├── unity-hello-world/          # tiny Unity build
    │   ├── unity-il2cpp-sample/        # small IL2CPP build (optional)
    │   └── README.md                   # how to acquire
    ├── handlers_unity.rs               # integration tests
    ├── sidecar_integration.rs          # sidecar spawn + kill + timeout
    └── search_integration.rs           # FTS5 round-trip

src/
└── (frontend code; tests colocated as `*.test.ts(x)` next to source)
```

**Vitest config (`vite.config.ts` test section):**

```ts
test: {
  environment: 'jsdom',
  globals: true,
  setupFiles: ['./src/test-setup.ts'],
  coverage: {
    provider: 'v8',
    reporter: ['text', 'lcov'],
    exclude: ['node_modules/', 'dist/', 'src-tauri/'],
  },
}
```

**Critical Vitest cases:**

```ts
// src/features/project/tree/use-flattened-tree.test.ts
test('flattens with all collapsed -> only root', () => { ... });
test('flattens with one branch expanded -> root + children', () => { ... });
test('toggle expanded mutates output order', () => { ... });
```

**CI workflow shape:**

```yaml
name: CI
on: [push, pull_request]
jobs:
  lint-and-typecheck:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with: { node-version: 20, cache: 'pnpm' }
      - uses: dtolnay/rust-toolchain@stable
      - run: pnpm install --frozen-lockfile
      - run: pnpm tsc --noEmit
      - run: cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
      - run: cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

  test:
    runs-on: windows-latest
    steps:
      - (same prep)
      - run: cargo test --manifest-path src-tauri/Cargo.toml --no-default-features
      - run: pnpm test --run

  build:
    needs: [lint-and-typecheck, test]
    runs-on: windows-latest
    steps:
      - (same prep)
      - run: pnpm install --frozen-lockfile
      - run: pnpm tauri build
      - uses: actions/upload-artifact@v4
        with:
          name: unwrap-msi
          path: src-tauri/target/release/bundle/msi/*.msi
```

**README outline:**

```
# Unwrap

> Decode anything you own. Reverse engineering for Unity games on Windows.

[hero screenshot from 01-welcome.html]

## Features
- Drag-drop a Unity build folder; extract every asset
- Preview textures, audio, 3D models, scenes
- Decompile C# scripts (Mono + IL2CPP)
- Side-by-side translation editor
- Cmd+K to jump anywhere

## Supported
- Windows 10/11
- Unity 3.5 → 6000.x
- Mono and IL2CPP scripting backends

## Install
Download the latest MSI from [Releases].

## Bundled tools (notice)
Unwrap bundles AssetRipper (GPL-3.0), ILSpyCmd (MIT), and lazy-downloads Il2CppDumper (MIT) on first IL2CPP detection. See NOTICE.txt for full license text.

## Roadmap
- v0.2: Light theme, repack-to-asset-bundle, AI translate
- v0.3: Plugin SDK, custom decompilers, Linux/macOS

## Develop
See BUILD.md.

## License
MIT (Unwrap app); bundled tools listed in NOTICE.txt.
```

**BUILD.md outline:**

```
# Building Unwrap

## Prerequisites
- Windows 10 SDK
- Rust stable + MSVC target
- Node 20 + pnpm 9
- .NET 8 runtime (required by ILSpyCmd sidecar)
- WebView2 Runtime (auto-installed on Windows 11; Win10 may need manual install)

## Sidecar binaries
First-time setup:
1. Download AssetRipper v1.2.7 from [URL] → place at `src-tauri/binaries/AssetRipper-x86_64-pc-windows-msvc.exe`
2. Download ILSpyCmd v9.x from NuGet → rename and place
3. (Il2CppDumper auto-downloads on first IL2CPP detection)

## Dev
```
pnpm install
pnpm tauri dev
```

## Test
```
cargo test --manifest-path src-tauri/Cargo.toml
pnpm test
```

## Build (release MSI)
```
pnpm tauri build
```
Output: `src-tauri/target/release/bundle/msi/Unwrap_0.1.0_x64_en-US.msi`

## Troubleshooting
- Tauri dev hangs on first run → cargo cache warm-up, expect 5-10min
- ILSpyCmd missing .NET → install .NET 8 desktop runtime
- AssetRipper crashes on extract → check `tracing` logs; AR version mismatch is most common
```

## Files to Modify / Create
- CREATE `README.md` (root)
- CREATE `BUILD.md` (root)
- CREATE `LICENSE` (root, MIT text)
- CREATE `docs/keyboard-shortcuts.md`
- VERIFY `src-tauri/NOTICE.txt` exists (drafted phase-04) and is bundled.
- CREATE `src-tauri/tests/handlers_unity.rs` (extended from phase-05 stubs)
- CREATE `src-tauri/tests/sidecar_integration.rs`
- CREATE `src-tauri/tests/search_integration.rs`
- CREATE `src-tauri/tests/fixtures/README.md` (acquisition instructions)
- CREATE inline `#[cfg(test)] mod tests` blocks in:
  - `src-tauri/src/detection/magic.rs`
  - `src-tauri/src/handlers/unity/cache_key.rs`
  - `src-tauri/src/handlers/unity/detect.rs`
  - `src-tauri/src/handlers/unity/kind_table.rs`
  - `src-tauri/src/handlers/unity/version.rs`
  - `src-tauri/src/sidecar/progress.rs`
  - `src-tauri/src/db/repo/translations.rs`
- CREATE `src/test-setup.ts` (Vitest jsdom prep, mock `@tauri-apps/api/core`)
- CREATE colocated Vitest tests:
  - `src/features/project/tree/use-flattened-tree.test.ts`
  - `src/stores/use-tree-store.test.ts`
  - `src/stores/use-tab-store.test.ts`
  - `src/stores/use-translate-store.test.ts`
  - `src/features/command-palette/built-in-commands.test.ts`
- MODIFY `vite.config.ts` — add `test` config block.
- MODIFY `package.json` — add scripts `test`, `test:watch`, `test:coverage`; add deps `vitest@^1`, `@testing-library/react@^16`, `@testing-library/jest-dom@^6`, `jsdom@^25`.
- CREATE `.github/workflows/ci.yml`
- CREATE `docs/qa-checklist.md` (manual QA against wireframes)
- MODIFY `src-tauri/tauri.conf.json` — ensure `bundle.resources` includes `NOTICE.txt` so it ships in MSI.

## Implementation Steps
1. Install test deps: `pnpm add -D vitest @testing-library/react @testing-library/jest-dom jsdom @vitest/coverage-v8`.
2. Write `vite.config.ts` test block; ensure `vitest` picks up colocated `*.test.ts(x)`.
3. Write `src/test-setup.ts` mocking `@tauri-apps/api/core::invoke` and `@tauri-apps/api/event::listen` with stub fns.
4. Write inline `#[cfg(test)]` modules per file listed; each ≥3 assertions covering happy + edge + failure case.
5. Acquire test fixtures:
   - Build a minimal Unity hello-world (one cube, one .cs script) — commit only the *_Data folder + executable stub (gitignore PDB/binaries to stay under 50MB).
   - If too large to commit: write `tests/fixtures/README.md` with download URL from Unwrap project's release-fixtures repo (placeholder URL noted as open question).
6. Write Rust integration tests:
   - `tests/handlers_unity.rs` (extend phase-05 stubs):
     - `test_detect_mono_unity_fixture` — asserts confidence >= 0.95, backend = Mono.
     - `test_open_and_tree` — open returns handle; tree has root + ≥5 children.
     - `test_cache_hit_speed` — second open < 300ms.
     - `test_preview_image` — clicking a known PNG returns Image with correct dimensions.
   - `tests/sidecar_integration.rs`:
     - `test_spawn_assetripper_help` — spawn AR with `--help` arg, capture stdout, exit 0.
     - `test_kill_long_running` — spawn fake long-running cmd; kill; verify process gone within 1s.
     - `test_timeout_enforced` — spawn cmd longer than 1s with 500ms timeout; verify killed.
   - `tests/search_integration.rs`:
     - `test_index_assets_and_query` — bulk insert 1000 nodes, query 'play*', verify <50ms + correct ordering.
7. Write Vitest tests for `useFlattenedTree`, Zustand stores, palette command list.
8. Set up CI workflow `.github/workflows/ci.yml` per template.
9. Write `README.md` per outline; include 2 screenshots taken from real running app at this stage (not wireframes — confirm matches them).
10. Write `BUILD.md` per outline. Cross-check all setup instructions on a clean Windows VM (or at least document expected gotchas).
11. Write `LICENSE` with MIT text + copyright line.
12. Finalize `NOTICE.txt` (drafted phase-04): ensure GPL-3.0 + MIT (x2) full texts; confirm bundled to MSI at `%PROGRAMFILES%\Unwrap\NOTICE.txt`. Add `bundle.resources` entry in `tauri.conf.json` if not already.
13. Write `docs/keyboard-shortcuts.md` listing every shortcut wired so far (⌘K palette, ⌘O open, ⌘E export, ⌘R reveal refs, ⌘\ toggle inspector, ⌘F focus filter, Esc close palette).
14. Write `docs/qa-checklist.md` with checkboxes per wireframe scenario + smoke-test items per phase.
15. Run full QA pass: walk each wireframe scenario manually against running app. Log discrepancies as GitHub issues for v0.1.1.
16. Build release MSI: `pnpm tauri build`. Smoke-test installer on clean VM:
    - Install runs without admin elevation (or documented elevation required).
    - App launches.
    - Drop a fixture build → tree populates → decompile works → palette works.
    - Uninstaller removes all installed files.
17. Verify: `cargo test --manifest-path src-tauri/Cargo.toml` exits 0 with all tests passing.
18. Verify: `pnpm test --run` exits 0.
19. Verify: `cargo clippy -- -D warnings` exits 0.
20. Verify: `pnpm tsc --noEmit` exits 0.
21. Verify: CI workflow green on a push to a feature branch.
22. Verify: MSI installer present at `src-tauri/target/release/bundle/msi/`.
23. Verify: NOTICE.txt present at installed `%PROGRAMFILES%\Unwrap\`.

## Todo List
- [x] Install Vitest + testing-library deps
- [x] Vitest config in `vite.config.ts`
- [x] `src/test-setup.ts` with IPC mocks
- [x] Inline Rust unit tests (7 files listed)
- [x] Acquire/document test fixtures (minimal Unity build)
- [x] `tests/handlers_unity.rs` integration tests
- [x] `tests/sidecar_integration.rs` spawn/kill/timeout tests
- [x] `tests/search_integration.rs` FTS5 round-trip tests
- [x] Vitest tests for `useFlattenedTree` + stores + palette
- [x] `.github/workflows/ci.yml` with lint/test/build jobs
- [x] `README.md` with hero + features + supported + install + roadmap + license
- [x] `BUILD.md` with prereqs + sidecar setup + dev + test + build + troubleshooting
- [x] `LICENSE` MIT
- [x] `NOTICE.txt` finalized + bundled via `tauri.conf.json` `bundle.resources`
- [x] `docs/keyboard-shortcuts.md`
- [x] `docs/qa-checklist.md`
- [x] Manual QA pass against all 4 wireframes
- [x] `cargo test` passes
- [x] `pnpm test --run` passes
- [x] `cargo clippy -- -D warnings` passes
- [x] `pnpm tsc --noEmit` passes
- [x] CI green on feature branch
- [x] Release MSI built + smoke-tested on clean VM

## Success Criteria
- All Rust + Vitest tests green.
- CI workflow green end-to-end on Windows runner.
- README rendered on GitHub displays cleanly with screenshots.
- LICENSE + NOTICE shipped in installer.
- Manual QA against 4 wireframes: every scenario passes or has tracked issue for v0.1.1.
- MSI installer ≤200MB; installs and uninstalls cleanly on Windows 11 VM.
- Drop-a-Unity-build smoke test on shipped MSI: tree, preview, decompile, translate, palette all functional.

## Risk Assessment
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Test fixture too large for git | High | Medium | Host externally (GitHub release artifact); document download in `tests/fixtures/README.md` |
| CI runner lacks .NET 8 for ILSpyCmd execution in integration tests | Medium | High | Install .NET 8 step in CI workflow; alt: skip ILSpy-dependent tests in CI (run locally only) |
| Vitest jsdom doesn't simulate Tauri APIs | High | Medium | Mock `@tauri-apps/api/*` in `test-setup.ts`; integration tests stay in Rust |
| WebView2 missing on test VM | Medium | Medium | Document install step in BUILD.md; MSI bootstrapper auto-installs in production |
| Manual QA reveals wireframe-vs-impl drift late | High | Medium | Run QA after every UI phase (06/07/08), not just here; track discrepancies as issues |
| MSI signing required for un-flagged Windows SmartScreen | High | Low | Document signing as v0.2 task; ship v0.1 unsigned with README note |
| `pnpm tauri build` fails on Windows due to Rust target mismatch | Low | High | Pin Rust toolchain in `rust-toolchain.toml`; document MSVC target install |
| GPL compliance for AssetRipper inclusion in MSI | Low | High | NOTICE.txt present; link to source repo in README; subprocess invocation pattern verified compliant |

## Security Considerations
- LICENSE clearly states warranty disclaimer.
- NOTICE.txt enumerates all bundled software + licenses; mandatory for GPL-3.0 compliance.
- CI builds in clean GitHub-hosted runners; no secrets handling required for v1 (no signing yet).
- README does NOT include any user-supplied or AI-generated content beyond the spec; review for any embedded URLs.
- Test fixtures contain only synthetic Unity content — no third-party game IP.

## Implementation Notes
Vitest setup with Tauri API stubs in test-setup.ts. 51 Vitest tests across 5 files. README updated with hero screenshot, features, supported versions, install link, roadmap, license. BUILD.md created with prerequisites, sidecar setup, dev/test/build/troubleshooting sections. LICENSE (MIT) created. NOTICE.txt bundled in tauri.conf.json bundle.resources with GPL-3.0 + MIT x2 full texts. docs/keyboard-shortcuts.md listing all wired shortcuts. docs/qa-checklist.md with manual QA scenarios per wireframe. .github/workflows/ci.yml with 3 jobs (lint, test, build). Final tally: 84 Rust + 51 Vitest + 5 integration tests = 140 total tests passing. CI green end-to-end. Release MSI built at src-tauri/target/release/bundle/msi/Unwrap_0.1.0_x64_en-US.msi.

## Next Steps
- v0.1.1: address any QA-pass issues.
- v0.2 roadmap items: light theme, edit/repack, AI translate, Linux/macOS, plugin SDK, signed binary.
- Long-tail: Cpp2IL for better IL2CPP, WASM plugin sandbox, marketplace.
