# Phase 01: Project Scaffolding

**Status:** Complete (2026-05-20)
**Priority:** Critical
**Effort:** S (1-2d)
**Depends on:** none

## Context Links
- Tech: `docs/tech-stack.md` (Stack Summary, Directory Layout)
- Arch: `docs/system-architecture.md` (High-Level Layers)
- Design: `docs/design-guidelines.md` (Tauri-specific, Color tokens)
- Brand: `docs/branding/brand-marks.md` (mark SVG, wordmark, favicon)

## Overview
Stand up the empty Tauri 2 + React 18 + Vite + TS + Tailwind 4 project. Wire brand assets (mark SVG, favicon, fonts). Verify a hello-world window opens with 0 lint/type errors. This phase is dead-simple but it freezes every tool version downstream — no scope creep.

## Key Insights
- Tauri 2 ships `create-tauri-app` with React+TS+Vite preset — use it, do not hand-roll.
- Tailwind 4 uses `@import "tailwindcss"` syntax (not v3 `@tailwind` directives) — config-as-CSS via `@theme`.
- Fonts must be self-hosted (no CDN at runtime) for offline-first guarantee per status bar "Offline mode".
- pnpm chosen for lockfile determinism + Tauri compatibility.

## Requirements

### Functional
1. `pnpm tauri dev` opens a Tauri window showing brand mark + "Unwrap" wordmark.
2. Fonts (Satoshi 450/500/700, JetBrains Mono 400/500/700) load locally.
3. Phosphor icons load via `@phosphor-icons/react` package (not CDN).
4. App icon (taskbar/exe) is `unwrap-mark.ico` generated from the SVG.
5. Cargo workspace builds clean with `cargo check`.
6. Frontend type-checks clean with `pnpm tsc --noEmit`.

### Non-Functional
- Cold start under 2s on dev build (smoke check, not strict).
- Bundle size budget noted in README (target: <15MB release).
- No CDN dependencies at runtime.

## Architecture / Approach

**Directory tree to create (matches `docs/tech-stack.md`):**

```
Unwrap/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs              # tauri::Builder::default().run()
│   │   └── lib.rs               # placeholder for future modules
│   ├── icons/                   # generated from unwrap-mark.svg
│   ├── Cargo.toml               # rusqlite, tokio, serde, anyhow stubs (commented out)
│   ├── tauri.conf.json          # title "Unwrap", min 1280x800, dark titlebar
│   └── build.rs
├── src/
│   ├── main.tsx
│   ├── App.tsx                  # renders <BrandHello/>
│   ├── components/
│   │   └── brand-hello.tsx      # mark + wordmark, font smoke test
│   ├── styles/
│   │   ├── globals.css          # @import "tailwindcss"; font-face declarations
│   │   └── fonts/               # Satoshi + JetBrains Mono .woff2
│   └── assets/
│       ├── unwrap-mark.svg
│       ├── unwrap-wordmark.svg
│       └── favicon.svg
├── public/                      # static favicon
├── package.json
├── pnpm-lock.yaml
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.ts           # placeholder (real tokens in phase-02)
└── README.md                    # short stub; full README in phase-10
```

**Pinned versions (do not deviate):**

| Package | Version |
|---|---|
| `@tauri-apps/cli` | `^2.1.0` |
| `@tauri-apps/api` | `^2.1.0` |
| `react`, `react-dom` | `^18.3.1` |
| `vite` | `^5.4.0` |
| `typescript` | `^5.6.0` |
| `tailwindcss` | `^4.0.0` (alpha-stable channel ok) |
| `@phosphor-icons/react` | `^2.1.7` |
| `tauri` (Rust crate) | `2.x` |
| `serde`, `serde_json` | `1.x` |
| `tokio` | `1.x` (features = ["full"]) |
| `anyhow` | `1.x` |

## Files to Modify / Create
- CREATE `package.json`
- CREATE `pnpm-lock.yaml` (via install)
- CREATE `tsconfig.json`
- CREATE `vite.config.ts`
- CREATE `tailwind.config.ts` (skeleton)
- CREATE `src/main.tsx`, `src/App.tsx`
- CREATE `src/components/brand-hello.tsx`
- CREATE `src/styles/globals.css`
- CREATE `src/styles/fonts/*.woff2` (download from Fontshare + Google Fonts; bundle)
- CREATE `src/assets/unwrap-mark.svg`, `unwrap-wordmark.svg`, `favicon.svg`
- CREATE `src-tauri/Cargo.toml`
- CREATE `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- CREATE `src-tauri/tauri.conf.json`
- CREATE `src-tauri/build.rs`
- CREATE `src-tauri/icons/` (32/128/256/icon.ico/icon.png via `tauri icon`)
- CREATE `README.md` (stub: name, tagline, "see plans/")
- CREATE `.gitignore` (node_modules, dist, target, .DS_Store, *.pdb)

## Implementation Steps
1. Run `pnpm dlx create-tauri-app@latest unwrap --template react-ts --manager pnpm` in a temp dir; copy the generated files into the repo root.
2. Pin all versions in `package.json` to the table above; run `pnpm install`.
3. Add Tailwind 4: `pnpm add -D tailwindcss@^4 @tailwindcss/vite`; wire `@tailwindcss/vite` plugin in `vite.config.ts`.
4. Add Phosphor: `pnpm add @phosphor-icons/react`.
5. Replace generated SVGs in `src-tauri/icons/` with brand mark — generate full icon set via `pnpm tauri icon src/assets/unwrap-mark.svg`.
6. Download Satoshi (450/500/700) `.woff2` files from Fontshare; download JetBrains Mono (400/500/700) `.woff2` files from Google Fonts; place in `src/styles/fonts/`.
7. In `globals.css`: declare `@font-face` for both families; `@import "tailwindcss";` at top.
8. Write `brand-hello.tsx`: centered mark SVG (32px) + "Unwrap" in Satoshi-700 32px + "Hello, build." in JetBrains Mono 14px below.
9. Set `tauri.conf.json` window: `title: "Unwrap"`, `width: 1440`, `height: 900`, `minWidth: 1280`, `minHeight: 800`, `decorations: true` (custom titlebar comes in phase-02), `theme: "Dark"`.
10. Verify: run `cargo check` in `src-tauri/`. Must pass with zero warnings.
11. Verify: run `pnpm tsc --noEmit`. Must pass with zero errors.
12. Verify: run `pnpm tauri dev`. Window must open showing brand mark + wordmark + "Hello, build."

## Todo List
- [x] Scaffold via `create-tauri-app` and merge into repo
- [x] Pin package versions in `package.json`
- [x] Install + wire Tailwind 4 via `@tailwindcss/vite`
- [x] Install `@phosphor-icons/react`
- [x] Drop brand SVGs into `src/assets/`
- [x] Generate Tauri icon set from `unwrap-mark.svg`
- [x] Self-host Satoshi + JetBrains Mono woff2 in `src/styles/fonts/`
- [x] Wire `@font-face` + `@import "tailwindcss"` in `globals.css`
- [x] Write `brand-hello.tsx` smoke component
- [x] Configure `tauri.conf.json` window settings
- [x] `cargo check` passes
- [x] `pnpm tsc --noEmit` passes
- [x] `pnpm tauri dev` opens window with brand

## Implementation Notes
- Cooked: 2026-05-20 (single dev, --auto bootstrap)
- Verification: `cargo check` pass, `pnpm tsc --noEmit` pass, `pnpm tauri dev` window renders
- Deviations: none — spec fully met
- Files shipped: Tauri scaffold + React 18 + Vite + Tailwind 4 + brand SVGs + self-hosted fonts (Satoshi, JetBrains Mono) in `src/styles/fonts/` + icon set in `src-tauri/icons/`

## Success Criteria
- `pnpm tauri dev` opens a window 1440x900, dark bg, with brand mark + wordmark rendered using self-hosted Satoshi.
- `cargo check` exits 0 with no warnings on Windows.
- `pnpm tsc --noEmit` exits 0.
- App icon in Windows taskbar matches `unwrap-mark.svg`.
- No HTTP requests to fonts.googleapis.com or fontshare.com at runtime (verified via DevTools Network tab).

## Risk Assessment
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Tailwind 4 still alpha — breaking API | Medium | High | Pin minor version; document escape hatch to v3 in README |
| Tauri 2 cargo cache misses on first build (slow) | High | Low | Document expected 5-10min first build in README |
| Satoshi license restricts redistribution | Low | Medium | Verify Fontshare license permits app bundling (it does for desktop apps) |
| `tauri icon` fails on complex SVG | Low | Medium | Provide pre-rasterized PNG fallback in `src/assets/icon-fallback.png` |

## Security Considerations
- All assets bundled; no network fetches at startup.
- `tauri.conf.json` `app.security.csp` set to strict default (no inline scripts except Vite HMR in dev).
- No Tauri commands defined yet — IPC surface is empty.

## Next Steps
- Unblocks phase-02 (design system tokens replace placeholder Tailwind config).
- No Rust modules created yet; phase-03 introduces commands, handlers, db.
