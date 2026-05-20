# Building Unwrap

## Prerequisites

| Requirement | Version | Notes |
|---|---|---|
| Windows | 10 / 11 x64 | Only supported platform for v0.1 |
| Rust toolchain | stable MSVC | `rustup default stable-x86_64-pc-windows-msvc` |
| Windows 10 SDK | 10.0.19041+ | Required by Tauri MSVC target |
| Node.js | 20+ | |
| pnpm | 9+ | `npm install -g pnpm` |
| .NET 8 Runtime | 8.0.x | Required by ILSpyCmd sidecar at runtime |
| WebView2 Runtime | any | Pre-installed on Windows 11; Win10 needs manual install |

Rust toolchain is pinned in `rust-toolchain.toml` to `stable-x86_64-pc-windows-msvc`.
Run `rustup show` to confirm the active toolchain.

## Sidecar binaries

Unwrap ships two prebuilt sidecars under `src-tauri/binaries/`. These are committed to the
repository for reproducible builds. For a fresh checkout they are already present.

If you need to re-acquire them (e.g. after a version bump):

**AssetRipper** (GPL-3.0)
1. Download the Windows x64 release from https://github.com/AssetRipper/AssetRipper/releases/tag/1.3.14
2. Rename the executable to `AssetRipper-x86_64-pc-windows-msvc.exe`
3. Place at `src-tauri/binaries/AssetRipper-x86_64-pc-windows-msvc.exe`

**ILSpyCmd** (MIT)
1. Install via .NET tool or download from NuGet: `ilspycmd` v9.1.0.7988
2. Rename the executable to `ilspycmd-x86_64-pc-windows-msvc.exe`
3. Place at `src-tauri/binaries/ilspycmd-x86_64-pc-windows-msvc.exe`

**Il2CppDumper** (MIT) — not bundled; downloaded on first IL2CPP detection.

## Development

Install frontend dependencies and start the dev server with hot-reload:

```
pnpm install
pnpm tauri dev
```

The first `cargo build` compiles rusqlite, symphonia, and three — expect 3-10 minutes on a
cold cache. Subsequent builds are incremental.

The Vite dev server runs on `http://localhost:1420`. Tauri will open a native window
pointing at that URL.

## Testing

Run all Rust unit and integration tests:

```
cargo test --manifest-path src-tauri/Cargo.toml
```

Run all frontend Vitest tests:

```
pnpm test
```

Run both together:

```
cargo test --manifest-path src-tauri/Cargo.toml && pnpm test
```

Run Rust linter (must be clean before committing):

```
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

## Production build (release MSI)

```
pnpm tauri build
```

Output artifact: `src-tauri/target/release/bundle/msi/Unwrap_0.1.0_x64_en-US.msi`

The MSI bundles the app binary, WebView2 bootstrapper, sidecar binaries, and NOTICE.txt.

Note: v0.1 ships unsigned. Windows SmartScreen will warn on first run. Signing is planned
for v0.2. Document this in release notes.

## Troubleshooting

**First `cargo build` hangs or takes very long**
Expected — rusqlite, symphonia, and three have many transitive crates. Allow 10 minutes on
first build. Subsequent incremental builds are fast.

**`pnpm tauri dev` exits immediately with "WebView2 not found"**
Install the WebView2 Evergreen Runtime from
https://developer.microsoft.com/en-us/microsoft-edge/webview2/

**ILSpyCmd fails to decompile with "No .NET runtime"**
Install the .NET 8 Desktop Runtime from https://dotnet.microsoft.com/download/dotnet/8.0

**AssetRipper crashes on extract**
Check the tracing logs (`%APPDATA%\Unwrap\logs\`). The most common cause is a version
mismatch between the bundled AssetRipper binary and the Unity project's asset format.
Update the sidecar to a newer AssetRipper release.

**`cargo clippy` reports `items_after_test_module`**
Ensure all `#[cfg(test)] mod tests { ... }` blocks are placed at the very end of their
source file, after all production functions.

**pnpm peer dependency warnings for vitest / @vitest/coverage-v8**
These are informational only and do not affect test correctness. Both packages are pinned
to `^1.6.1` to stay aligned.
