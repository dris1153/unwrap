# Unwrap

<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" fill="none" width="128" height="128" role="img" aria-label="Unwrap" style="display: block; margin: 16px 0;">
  <title>Unwrap</title>
  <!-- Back bracket -->
  <path d="M62 18 Q48 18 48 32 Q48 45 36 50 Q48 55 48 68 Q48 82 62 82"
        stroke="#10B981" stroke-width="6.5"
        stroke-linecap="round" stroke-linejoin="round" opacity="0.22"/>
  <!-- Middle bracket -->
  <path d="M56 18 Q42 18 42 32 Q42 45 30 50 Q42 55 42 68 Q42 82 56 82"
        stroke="#10B981" stroke-width="6.5"
        stroke-linecap="round" stroke-linejoin="round" opacity="0.5"/>
  <!-- Front bracket -->
  <path d="M50 18 Q36 18 36 32 Q36 45 24 50 Q36 55 36 68 Q36 82 50 82"
        stroke="#10B981" stroke-width="6.5"
        stroke-linecap="round" stroke-linejoin="round"/>
  <!-- Closing chevron echo -->
  <path d="M62 38 L72 50 L62 62"
        stroke="#10B981" stroke-width="6.5"
        stroke-linecap="round" stroke-linejoin="round" opacity="0.85"/>
</svg>

> Decode anything you own. Windows-first desktop reverse engineering tool for Unity games.

## Status

**v0.1 MVP — all 10 phases complete**

## What it does (v0.1)

- Drop a Unity build folder — tree of extracted assets populates instantly
- Preview textures, audio, 3D meshes, text files, and raw hex
- Decompile C# scripts via Mono and IL2CPP backends
- Side-by-side translation editor with autosave
- Cmd+K command palette to jump anywhere
- Read-only viewer (no edit or repack in v0.1)

## Supported

- Windows 10 / 11 (x64)
- Unity 3.5 through 6000.x
- Mono and IL2CPP scripting backends

## Install

Download the latest MSI from [Releases](../../releases) and run it.
WebView2 Runtime is required (pre-installed on Windows 11; the installer prompts on Windows 10).

## Tech Stack

- **Desktop:** Tauri 2, React 18, TypeScript, Vite, Tailwind 4
- **Backend:** Rust 1.95+ with tokio async runtime
- **Storage:** SQLite via rusqlite
- **Sidecars:** AssetRipper (GPL-3.0, subprocess), ILSpyCmd (MIT), Il2CppDumper (MIT, on-demand)
- **UI:** Zustand, TanStack Router, TanStack Virtual, Monaco Editor

## Prerequisites (development)

- Windows 10 / 11
- Rust stable MSVC toolchain: `rustup default stable-x86_64-pc-windows-msvc`
- Node.js 20+ and pnpm 9+
- .NET 8 Runtime (required by ILSpyCmd sidecar)
- WebView2 Runtime

## Develop

```
pnpm install
pnpm tauri dev
```

First `cargo build` takes 3-10 minutes while Rust compiles rusqlite, three, and symphonia.

## Test

```
cargo test --manifest-path src-tauri/Cargo.toml
pnpm test
```

## Build

```
pnpm tauri build
```

Output: `src-tauri/target/release/bundle/msi/Unwrap_0.1.0_x64_en-US.msi`

See [BUILD.md](BUILD.md) for full setup instructions including sidecar binary acquisition.

## Bundled tools

Unwrap bundles or downloads the following third-party tools:

| Tool | License | How bundled |
|---|---|---|
| AssetRipper | GPL-3.0 | Shipped binary, invoked as subprocess |
| ILSpyCmd | MIT | Shipped binary, invoked as subprocess |
| Il2CppDumper | MIT | Downloaded on first IL2CPP detection |

See [src-tauri/NOTICE.txt](src-tauri/NOTICE.txt) for full license texts and source URLs.
Unwrap invokes each tool only as a separate process and does not link any of their code.

## Roadmap

- **v0.2** — Light theme, asset repack to bundle, AI-assisted translation, signed installer
- **v0.3** — Plugin SDK, custom decompiler adapters, Linux / macOS support

## Documentation

- [BUILD.md](BUILD.md) — Full build and sidecar setup
- [docs/keyboard-shortcuts.md](docs/keyboard-shortcuts.md) — Keyboard reference
- [docs/development-roadmap.md](docs/development-roadmap.md) — Phase progress
- [docs/project-changelog.md](docs/project-changelog.md) — Version history

## License

MIT — see [LICENSE](LICENSE).
Bundled tools retain their original licenses; see [src-tauri/NOTICE.txt](src-tauri/NOTICE.txt).
