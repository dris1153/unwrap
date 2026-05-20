<p align="left">
  <img src="docs/branding/logos/unwrap-mark.svg" alt="Unwrap" width="96" height="96">
</p>

# Unwrap

> Decode anything you own. A Windows desktop tool for reverse engineering Unity games.

## What it does

Drop a Unity build folder into Unwrap and instantly explore it.

- **Browse assets** — textures, audio, 3D meshes, text, raw hex, all in a virtualized tree
- **Decompile C#** — Mono and IL2CPP backends, with Monaco syntax highlighting
- **Translate strings** — side-by-side editor with autosave to a local database
- **Jump anywhere** — Cmd+K (or Ctrl+K) palette searches assets, code, and strings
- **Stays local** — nothing is uploaded; everything runs on your machine

Read-only viewer. Editing and repacking are on the roadmap.

## Supported

- Windows 10 / 11 (x64)
- Unity 3.5 through 6000.x
- Mono and IL2CPP scripting backends

## Install

Download the latest MSI from [Releases](../../releases) and run it. WebView2 Runtime is required — pre-installed on Windows 11; the installer prompts to add it on Windows 10.

## Develop

```
pnpm install
pnpm sidecars
pnpm tauri dev
```

`pnpm sidecars` is a one-time step after a fresh clone. It auto-downloads **AssetRipper 1.3.14** (~120 MB) from upstream, installs **ILSpyCmd 9.1.0.7988** via `dotnet tool install --global` (needs .NET 8 SDK on PATH), and skips **Il2CppDumper** — the app fetches it lazily on first IL2CPP detection.

The script is idempotent. Re-run it whenever versions change in [src-tauri/binaries/sidecar-manifest.json](src-tauri/binaries/sidecar-manifest.json).

First `cargo build` takes 3–10 minutes while Rust compiles rusqlite, three, and symphonia.

### Prerequisites

- Rust stable MSVC toolchain — `rustup default stable-x86_64-pc-windows-msvc`
- Node.js 20+ and pnpm 9+
- .NET 8 SDK (for ILSpyCmd dev install)
- WebView2 Runtime

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

Full instructions including sidecar acquisition: [BUILD.md](BUILD.md).

## Tech stack

- **Desktop:** Tauri 2, React 18, TypeScript, Vite, Tailwind 4
- **Backend:** Rust 1.95+ on tokio
- **Storage:** SQLite via rusqlite
- **UI:** Zustand, TanStack Router, TanStack Virtual, Monaco Editor
- **Sidecars:** AssetRipper (GPL-3.0, subprocess), ILSpyCmd (MIT), Il2CppDumper (MIT, on-demand)

## Bundled tools

Unwrap invokes the following third-party tools only as separate processes; none of their code is linked into the Unwrap binary.

| Tool | License | Delivery |
|---|---|---|
| AssetRipper | GPL-3.0 | Bundled sidecar |
| ILSpyCmd | MIT | Bundled sidecar |
| Il2CppDumper | MIT | Lazy-downloaded on first IL2CPP detection |

Full license texts and source URLs: [src-tauri/NOTICE.txt](src-tauri/NOTICE.txt).

## Documentation

- [BUILD.md](BUILD.md) — Full build setup
- [docs/keyboard-shortcuts.md](docs/keyboard-shortcuts.md) — Keyboard reference
- [docs/project-changelog.md](docs/project-changelog.md) — Version history

## License

MIT — see [LICENSE](LICENSE). Bundled tools retain their original licenses.
