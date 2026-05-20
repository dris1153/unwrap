---
title: "Unwrap MVP Bootstrap"
description: "Windows-first Tauri 2 + React desktop tool for Unity reverse engineering: asset extract, decompile, translate (read-only v1)."
status: completed
priority: P1
effort: 18-26d
branch: main
tags: [tauri, rust, react, unity, reverse-engineering, mvp]
created: 2026-05-20
---

# Unwrap — MVP Bootstrap Plan

**Created:** 2026-05-20
**Mode:** /ck:plan --auto (bootstrap from /ck:bootstrap)
**Target:** Windows-first MVP, Unity decompile read-only viewer
**Stack:** Tauri 2 + React 18 + TypeScript + Tailwind 4 + Rust
**Plan dir:** plans/20260520-1530-unwrap-mvp-bootstrap

## Goal
Ship a polished read-only Unity reverse engineering viewer on Windows. User drops a Unity build folder; app detects scripting backend (Mono/IL2CPP), extracts assets via AssetRipper sidecar, surfaces a virtualized asset tree, lets user preview images/audio/3D/text/hex, decompile C# scripts via ILSpyCmd (+ Il2CppDumper for IL2CPP), and view translatable strings side-by-side. SQLite cache by file hash. Cmd+K palette. Pixel-match 4 wireframes. No edit/repack v1.

## Phases

| # | Name | Status | Effort | File |
|---|------|--------|--------|------|
| 01 | Project Scaffolding | Complete | S (1-2d) | [phase-01-project-scaffolding.md](phase-01-project-scaffolding.md) |
| 02 | Design System & Shell | Complete | M (2-3d) | [phase-02-design-system-and-shell.md](phase-02-design-system-and-shell.md) |
| 03 | Rust Backend Core | Complete | L (3-4d) | [phase-03-rust-backend-core.md](phase-03-rust-backend-core.md) |
| 04 | Sidecar Tools Integration | Complete | M (2-3d) | [phase-04-sidecar-tools-integration.md](phase-04-sidecar-tools-integration.md) |
| 05 | Unity Format Handler | Complete | L (3-4d) | [phase-05-unity-format-handler.md](phase-05-unity-format-handler.md) |
| 06 | Asset Browse & Preview | Complete | L (3-4d) | [phase-06-asset-browse-and-preview.md](phase-06-asset-browse-and-preview.md) |
| 07 | Code Decompile | Complete | M (2-3d) | [phase-07-code-decompile.md](phase-07-code-decompile.md) |
| 08 | Translatable Text View | Complete | M (2-3d) | [phase-08-translatable-text-view.md](phase-08-translatable-text-view.md) |
| 09 | Search & Command Palette | Complete | S (1-2d) | [phase-09-search-and-command-palette.md](phase-09-search-and-command-palette.md) |
| 10 | Testing & Docs | Complete | M (2-3d) | [phase-10-testing-and-docs.md](phase-10-testing-and-docs.md) |

## Completion Log

| Date | Phase | Status | Notes |
|------|-------|--------|-------|
| 2026-05-20 | 01 Project Scaffolding | Complete | Tauri 2 + React 18 + Vite + Tailwind 4 + brand + fonts shipped. Toolchain switched to MSVC. |
| 2026-05-20 | 02 Design System & Shell | Complete | Full app shell + Welcome route. 1 icon substitution (FolderNotchOpen → FolderSimplePlus). |
| 2026-05-20 | 03 Rust Backend Core | Complete | 28 Rust + 2 TS files. Clippy clean. 7 unit tests pass. |
| 2026-05-20 | 04 Sidecar Tools Integration | Complete | AR 1.3.14 + ILSpy 9.1 bundled; Il2CppDumper lazy-download config; NOTICE.txt + GPL-3.0 attribution. 24 tests. |
| 2026-05-20 | 05 Unity Format Handler | Complete | UnityHandler impl FormatHandler; synthetic fixture; 43 unit + 5 integration tests. |
| 2026-05-20 | 06 Asset Browse & Preview | Complete | Virtualized tree, tabs, image/audio/3D/text/hex previews, inspector. Three.js + Monaco lazy-loaded. |
| 2026-05-20 | 07 Code Decompile | Complete | decompile command, Monaco unwrap-dark theme, backend/confidence badges. 51 tests. |
| 2026-05-20 | 08 Translatable Text View | Complete | DB v2 + 4 parsers + side-by-side editor with debounced autosave. 76 tests. |
| 2026-05-20 | 09 Search + Palette | Complete | DB v3 FTS5 + cmdk palette + 5 built-in commands + global shortcuts. 79 tests. |
| 2026-05-20 | 10 Testing + Docs | Complete | Vitest 51 tests + LICENSE + BUILD.md + CI workflow + NOTICE bundled. 140 total tests pass. |

**Active progress:** 100% (10/10 phases complete — MVP complete).

## Dependency Chain
phase-01 -> phase-02 -> phase-03 -> phase-04 -> phase-05 -> (06 || 07 || 08) -> phase-09 -> phase-10

(Phases 06/07/08 share phase-05 as common prerequisite; can be parallelized once 05 is done — saves ~5-8d.)

## Out-of-Scope (Explicit)
- Cross-platform (Linux/macOS) — Windows only v1
- Editing/repacking assets — read-only v1
- Plugin marketplace, signed plugins, WASM sandbox
- Real-time game memory hooking
- DRM circumvention
- Light theme (dark-only v1)
- Python/UnityPy fallback
- Cpp2IL (use Il2CppDumper for v1)

## Total Estimate
~18-26 days for full MVP (single dev). Phases 06/07/08 parallelizable saves ~5-8 days.

## Notable Risks
- AssetRipper version churn — pin and document upgrade procedure (phase 04)
- WebView2 missing on older Win10 — Tauri auto-installs but flag in BUILD.md (phase 10)
- IL2CPP decompile accuracy ~70% — surface confidence in UI inspector (phase 07)
- Large asset trees (>20k nodes) — virtualize via TanStack Virtual (phase 06)
- GPL infection — AssetRipper subprocess only, never link (phase 04, phase 10)

## Open Questions (post-plan)
- Final app name? Codename Unwrap — defer to user.
- Asset cache TTL / location preference? Default `%LOCALAPPDATA%\Unwrap\cache` — confirm.
- Should support drop-by-archive-file (.zip of game) in v1 or v2? Wireframe shows it; defer concrete impl to phase 05 stretch.
- IL2CPP-only sidecar lazy-download policy: bundle or download on first detection? Plan as deferred-download in phase 04.
