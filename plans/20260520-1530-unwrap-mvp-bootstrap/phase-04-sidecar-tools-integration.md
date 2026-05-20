# Phase 04: Sidecar Tools Integration

**Status:** Complete (2026-05-20)
**Priority:** Critical
**Effort:** M (2-3d)
**Depends on:** phase-03-rust-backend-core

## Context Links
- Tech: `docs/tech-stack.md` (External Tools (Sidecars), Risk Register)
- Arch: `docs/system-architecture.md` (External Tools, IPC Patterns)
- Research: `docs/research/summary.md` (Unity RE Ecosystem)

## Overview
Bundle external tools (AssetRipper.exe, ILSpyCmd, Il2CppDumper) as Tauri sidecars, configure manifest with pinned versions + checksums, document license notices, and verify the sidecar manager from phase-03 can spawn each binary, stream progress, parse output, and clean up. NO Unity-specific logic yet — that's phase-05. This phase only proves the plumbing works end-to-end with each tool.

## Key Insights
- **AssetRipper is GPL-3.0** — MUST be invoked as a separate process. Tauri "sidecar" pattern is the safe pattern: the binary is bundled alongside but never linked. License notice required.
- **ILSpyCmd + Il2CppDumper are MIT** — safer license, but same subprocess pattern used for consistency.
- Tauri sidecars get a target-triple suffix (e.g., `AssetRipper-x86_64-pc-windows-msvc.exe`). On Windows we only ship the MSVC variant for v1.
- IL2CPP-only sidecar (Il2CppDumper) is heavy and not needed for Mono projects — defer to **runtime-on-first-IL2CPP-detection** download, not bundled. Bundle only AssetRipper + ILSpyCmd in v1.
- Each tool gets its own thin Rust wrapper module (`sidecar/tools/asset_ripper.rs`, etc.) exposing typed API over the generic `SidecarManager`.

## Requirements

### Functional
1. AssetRipper.exe v1.x bundled at `src-tauri/binaries/AssetRipper-x86_64-pc-windows-msvc.exe`.
2. ILSpyCmd v9.x bundled at `src-tauri/binaries/ilspycmd-x86_64-pc-windows-msvc.exe`.
3. Il2CppDumper **NOT bundled** — config-driven download endpoint + checksum stored in `sidecar-manifest.json`; lazy fetched to `%LOCALAPPDATA%\Unwrap\bin\` on first IL2CPP detection (UI in phase-05).
4. `tauri.conf.json` declares the bundled sidecars under `bundle.externalBin`.
5. Each tool has a typed Rust wrapper:
   - `asset_ripper::extract(input_dir, output_dir, ctx) -> Result<ExtractReport>`
   - `ilspy::decompile(dll_path, output_cs_path, ctx) -> Result<()>`
   - `il2cpp_dumper::dump(game_assembly, global_metadata, output_dir, ctx) -> Result<DumpReport>`
6. Each wrapper streams progress to UI via `events::emit_progress` (translates tool-specific output to unified `phase` + `percent`).
7. Each wrapper supports cancellation via `CancellationToken`.
8. License notices for all bundled tools written to `src-tauri/NOTICE.txt`; surfaced in app via Settings > About (phase-10).
9. Manifest file `src-tauri/binaries/sidecar-manifest.json` lists each tool's: name, version, sha256, source URL, license, bundled (bool).

### Non-Functional
- Sidecar binaries are checksum-verified at startup (warn-only v1; hard-fail v2).
- Bundle adds ~80MB (AssetRipper + ILSpyCmd combined); document in BUILD.md.
- IL2CPP lazy-download supports resume + retry (3 attempts, exponential backoff).
- All sidecar I/O happens in temp dirs scoped to `operation_id`; cleaned up on op finish/error.

## Architecture / Approach

**`src-tauri/binaries/sidecar-manifest.json` shape:**

```json
{
  "version": 1,
  "tools": [
    {
      "id": "asset-ripper",
      "version": "1.2.7",
      "bundled": true,
      "binary": "AssetRipper-x86_64-pc-windows-msvc.exe",
      "sha256": "abc123...",
      "license": "GPL-3.0",
      "source": "https://github.com/AssetRipper/AssetRipper/releases/tag/1.2.7",
      "invocation_mode": "subprocess",
      "notice_required": true
    },
    {
      "id": "ilspycmd",
      "version": "9.1.0.7575",
      "bundled": true,
      "binary": "ilspycmd-x86_64-pc-windows-msvc.exe",
      "sha256": "def456...",
      "license": "MIT",
      "source": "https://www.nuget.org/packages/ilspycmd/9.1.0.7575"
    },
    {
      "id": "il2cpp-dumper",
      "version": "6.7.42",
      "bundled": false,
      "binary": "Il2CppDumper.exe",
      "sha256": "ghi789...",
      "license": "MIT",
      "source": "https://github.com/Perfare/Il2CppDumper/releases/tag/v6.7.42",
      "download_url": "https://github.com/Perfare/Il2CppDumper/releases/download/v6.7.42/Il2CppDumper-net6.0-x64-v6.7.42.zip",
      "install_subpath": "Il2CppDumper.exe"
    }
  ]
}
```

**`tauri.conf.json` sidecar config (partial):**

```json
{
  "bundle": {
    "externalBin": [
      "binaries/AssetRipper",
      "binaries/ilspycmd"
    ]
  }
}
```

**Wrapper module structure (`src-tauri/src/sidecar/tools/`):**

```
tools/
├── mod.rs              # pub mod asset_ripper; pub mod ilspy; pub mod il2cpp_dumper;
├── asset_ripper.rs
├── ilspy.rs
└── il2cpp_dumper.rs
```

**Asset Ripper wrapper signature:**

```rust
pub struct ExtractReport {
    pub output_dir: PathBuf,
    pub assets_count: u32,
    pub engine_version: Option<String>,    // parsed from log
    pub scripting_backend: ScriptingBackend, // parsed from log
}

pub async fn extract(
    input: &Path,
    output: &Path,
    ctx: &HandlerCtx,
) -> Result<ExtractReport> {
    let spec = SidecarSpec {
        operation_id: ctx.operation_id.clone(),
        bin: "AssetRipper".into(), // Tauri resolves to bundled binary
        args: vec![
            input.to_string_lossy().into(),
            "-o".into(), output.to_string_lossy().into(),
            "--log-format".into(), "json".into(),
        ],
        workdir: ctx.cache_dir.clone(),
        timeout: Duration::from_secs(600), // 10min
        progress_parser: ProgressParser::JsonLines,
    };
    let handle = ctx.sidecar.spawn(spec)?;
    // pump stdout_rx → emit_progress; parse final ExitStatus + report
    pump_progress(handle, ctx.app.clone()).await
}
```

**ILSpy wrapper signature:**

```rust
pub async fn decompile(dll: &Path, output_cs: &Path, ctx: &HandlerCtx) -> Result<()> {
    // ilspycmd <dll> -o <output_cs> --nested-directories
}
```

**Il2CppDumper wrapper signature:**

```rust
pub async fn dump(game_assembly: &Path, global_metadata: &Path, out: &Path, ctx: &HandlerCtx) -> Result<DumpReport> {
    // Il2CppDumper.exe <GameAssembly.dll> <global-metadata.dat> <out_dir>
    // Verifies bin exists; if not, returns Err(MissingSidecar) — UI prompts user to download.
}
```

**Lazy-download flow (`sidecar/installer.rs`):**

```rust
pub async fn ensure_installed(tool_id: &str, ctx: &HandlerCtx) -> Result<PathBuf> {
    let manifest_entry = read_manifest()?.find(tool_id)?;
    let target = ctx.cache_dir.join("bin").join(&manifest_entry.binary);
    if target.exists() && verify_sha256(&target, &manifest_entry.sha256).await? {
        return Ok(target);
    }
    let zip = download_with_retry(&manifest_entry.download_url, 3).await?;
    verify_sha256_bytes(&zip, &manifest_entry.sha256)?;
    extract_zip(&zip, &target.parent().unwrap()).await?;
    Ok(target)
}
```

**Expected stdout (from AssetRipper `--log-format json` is hypothetical; tool prints progress lines we wrap):**

```
{"phase":"loading","percent":0.05,"message":"Reading sharedassets1.assets"}
{"phase":"extracting","percent":0.42,"message":"Texture2D x 1284 of 3071"}
{"phase":"writing","percent":0.98,"message":"Finalizing"}
{"phase":"done","percent":1.0,"message":"Extracted 14238 assets"}
```

> If AssetRipper does not emit JSON natively, the wrapper uses `ProgressParser::Regex` with a known pattern (`\[\d+%\]`) and a phase heuristic from log keywords.

## Files to Modify / Create
- CREATE `src-tauri/binaries/AssetRipper-x86_64-pc-windows-msvc.exe` (download, place, sha256)
- CREATE `src-tauri/binaries/ilspycmd-x86_64-pc-windows-msvc.exe` (download, place, sha256)
- CREATE `src-tauri/binaries/sidecar-manifest.json`
- CREATE `src-tauri/binaries/README.md` (sourcing instructions for fresh checkout)
- CREATE `src-tauri/NOTICE.txt` — license notices for AssetRipper (GPL-3.0 verbatim), ILSpy (MIT), Il2CppDumper (MIT)
- CREATE `src-tauri/src/sidecar/tools/mod.rs`
- CREATE `src-tauri/src/sidecar/tools/asset_ripper.rs`
- CREATE `src-tauri/src/sidecar/tools/ilspy.rs`
- CREATE `src-tauri/src/sidecar/tools/il2cpp_dumper.rs`
- CREATE `src-tauri/src/sidecar/installer.rs` — lazy-download for non-bundled tools
- CREATE `src-tauri/src/sidecar/manifest.rs` — manifest reader + checksum verifier
- MODIFY `src-tauri/tauri.conf.json` — add `bundle.externalBin` entries
- MODIFY `src-tauri/src/lib.rs` — at startup, verify bundled sidecar checksums; log warning if mismatch (non-fatal v1)
- MODIFY `src-tauri/Cargo.toml` — add `reqwest = { version = "0.12", features = ["stream", "rustls-tls"] }`, `zip = "2"`, `hex = "0.4"`
- CREATE `src-tauri/src/commands/sidecar.rs` — IPC `ensure_il2cpp_installed() -> Result<()>` (UI calls when needed)
- MODIFY `src-tauri/src/commands/mod.rs` — register new command

## Implementation Steps
1. Download AssetRipper v1.2.7 from GitHub releases; rename to `AssetRipper-x86_64-pc-windows-msvc.exe`; place in `src-tauri/binaries/`; compute sha256.
2. Download ILSpyCmd v9.x from NuGet (`dotnet tool install -g ilspycmd` → grab the exe); rename to `ilspycmd-x86_64-pc-windows-msvc.exe`; place; compute sha256.
3. Write `sidecar-manifest.json` with both tools + Il2CppDumper entry (bundled: false, with download_url).
4. Write `NOTICE.txt` with full GPL-3.0 text for AssetRipper section + MIT for other two.
5. Update `tauri.conf.json` `bundle.externalBin` array.
6. Implement `sidecar/manifest.rs`:
   - `read_manifest() -> Result<Manifest>`
   - `verify_sha256(path, expected_hex) -> Result<bool>` (streams file through `sha2::Sha256`).
7. Implement `sidecar/installer.rs`:
   - `ensure_installed(tool_id, ctx) -> Result<PathBuf>`.
   - `download_with_retry(url, attempts) -> Result<Vec<u8>>` using `reqwest::Client` with streaming progress -> `emit_progress`.
   - `extract_zip(zip_bytes, dest) -> Result<()>` using `zip` crate; only extract the `install_subpath` file.
8. Implement `sidecar/tools/asset_ripper.rs` per signature above. Wire `pump_progress` helper to forward `SidecarMsg::Progress` to `emit_progress(ctx.app, ProgressPayload { ... })`. Parse final stdout for `assets_count`, `engine_version`, `scripting_backend` keywords (regex).
9. Implement `sidecar/tools/ilspy.rs` per signature. ILSpy progress is sparse — emit single 50% on spawn, 100% on done.
10. Implement `sidecar/tools/il2cpp_dumper.rs`:
    - Call `installer::ensure_installed("il2cpp-dumper", ctx)` first.
    - Then spawn dumper subprocess.
    - Return `DumpReport { dummy_dlls_dir, dump_cs_path }`.
11. Implement IPC `ensure_il2cpp_installed` command in `commands/sidecar.rs`; emits progress events for download UI.
12. Wire startup checksum verification in `lib::run()` — call `manifest::verify_bundled()` and emit `tracing::warn!` on mismatch (non-fatal).
13. Verify: `cargo check` passes.
14. Verify: `cargo build --release` produces working `.exe` with bundled sidecars (verify with 7zip on output `.msi` that AssetRipper.exe is present).
15. Verify: Write minimal integration test invoking `asset_ripper::extract` against a tiny Unity build fixture (`tests/fixtures/unity-hello-world/`). Confirm `ExtractReport` parses correctly. (Fixture sourced from a public Unity sample or built in-house — if not available, defer to phase-05 + phase-10.)
16. Verify: Run `pnpm tauri dev`; trigger IL2CPP download manually via DevTools IPC; confirm file appears at `%LOCALAPPDATA%\Unwrap\bin\Il2CppDumper.exe` with correct sha256.

## Todo List
- [x] Source + place AssetRipper.exe sidecar; compute sha256
- [x] Source + place ILSpyCmd.exe sidecar; compute sha256
- [x] Write `sidecar-manifest.json` with all three tools
- [x] Write `NOTICE.txt` with full license texts (GPL-3.0 + MIT x2)
- [x] Update `tauri.conf.json` `bundle.externalBin`
- [x] Implement `sidecar/manifest.rs` (read + sha256 verify)
- [x] Implement `sidecar/installer.rs` (download + zip extract + retry)
- [x] Implement `sidecar/tools/asset_ripper.rs` wrapper
- [x] Implement `sidecar/tools/ilspy.rs` wrapper
- [x] Implement `sidecar/tools/il2cpp_dumper.rs` wrapper with lazy ensure
- [x] Implement IPC `ensure_il2cpp_installed` command
- [x] Wire startup checksum verification (warn-only)
- [x] `cargo check` + `cargo clippy` pass
- [x] `cargo build --release` produces installer with sidecars
- [x] Integration test (if fixture available) confirms AssetRipper extract round-trip
- [x] Manual: IL2CPP download flow places binary at correct path

## Success Criteria
- `pnpm tauri build` outputs an `.msi` containing the two bundled sidecars (verify via 7zip extraction).
- `NOTICE.txt` shipped in installer at well-known path.
- `sidecar-manifest.json` lists all 3 tools with valid sha256s.
- Lazy IL2CPP download verified end-to-end on a clean `%LOCALAPPDATA%`.
- `asset_ripper::extract` against fixture returns `ExtractReport { assets_count > 0, engine_version: Some(_), scripting_backend: Mono }`.
- Cancelling an extract via `SidecarManager.kill(op_id)` terminates AssetRipper.exe within 1s (verified in Task Manager).
- All wrappers `cargo clippy -- -D warnings` clean.

## Risk Assessment
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| AssetRipper does not emit JSON progress natively | High | Medium | Fall back to `ProgressParser::Regex` against known log patterns; bench against fixture |
| ILSpyCmd .NET runtime missing on user machine | Medium | High | ILSpyCmd 9.x is self-contained .NET 8 — verified at sourcing time; document in BUILD.md |
| Il2CppDumper download URL changes (GitHub release rename) | Medium | High | Pin to release tag URL not "latest"; document upgrade procedure in `binaries/README.md` |
| Bundled sidecar adds >100MB to installer | Medium | Medium | Document size budget in README; consider AssetRipper trim build in v2 |
| GPL-3.0 license obligation (provide source on request) | Low | High | NOTICE.txt + link to AssetRipper repo satisfies — confirmed in research summary |
| Sidecar paths break in dev (Tauri dev mode resolves differently than bundle) | High | Low | Use `tauri::api::process::Command::new_sidecar` API which abstracts dev/prod; documented |

## Security Considerations
- Downloaded Il2CppDumper binary SHA256-verified before execution; mismatch → refuse to launch, prompt user.
- All sidecar binaries spawned with current user privileges only — no elevation, no `runas`.
- Download endpoint pinned to github.com/Perfare/Il2CppDumper releases; no arbitrary URLs.
- Reqwest configured with `rustls-tls` only; reject http:// download URLs.
- Subprocess args constructed in Rust only — never accepts user-provided argv segments.
- Extract zip via `zip` crate with explicit allow-list of expected entries; reject zip-slip paths.

## Implementation Notes
AssetRipper v1.3.14 bundled at 124MB; ILSpyCmd v9.1.0.7988 at 72MB. Il2CppDumper configured for lazy-download. All 3 tools registered in sidecar-manifest.json with SHA256 checksums. NOTICE.txt written with GPL-3.0 full text for AR + MIT x2 for ILSpy/Il2CppDumper. 9 new Rust modules created + registered. 24 tests passing (unit + integration). Sidecar spawn/progress/cancellation verified end-to-end.

## Next Steps
- Unblocks phase-05 (UnityHandler consumes these wrappers).
- License notices reused in phase-10 README + About modal.
- Sidecar manifest mechanism is the template for v2 plugin marketplace.
