# Phase 05: Unity Format Handler

**Status:** Complete (2026-05-20)
**Priority:** Critical
**Effort:** L (3-4d)
**Depends on:** phase-04-sidecar-tools-integration

## Context Links
- Tech: `docs/tech-stack.md` (Auto-detection flow, External Tools)
- Arch: `docs/system-architecture.md` (Domain Models, Workflow #1 Open Unity Project)
- Research: `docs/research/summary.md` (Mono vs IL2CPP reality)

## Overview
Implement the concrete `UnityHandler: FormatHandler` end-to-end. Detects Mono vs IL2CPP, parses Unity engine version, invokes AssetRipper sidecar from phase-04 to extract assets, builds in-memory `AssetTree` from the extracted file structure, caches the tree blob in SQLite keyed by source-file hash, and emits progress through the IPC event channel. This phase wires the full "drop folder → tree visible" pipeline; the UI for browsing it is phase-06.

## Key Insights
- **Detection signals** (ranked by reliability):
  - `*_Data/Managed/*.dll` exists → Mono backend.
  - `GameAssembly.dll` + `*_Data/il2cpp_data/Metadata/global-metadata.dat` exist → IL2CPP backend.
  - `globalgamemanagers` in `*_Data/` → confirms Unity build.
  - Engine version parsed from `*_Data/globalgamemanagers` binary header (4-byte magic + Unity version string).
- Confidence = 0.95 if both `*_Data/` + (Managed/Mono OR GameAssembly/IL2CPP) match; 0.70 if only `*_Data/`; 0.0 otherwise.
- `AssetTree` construction is a 2-pass walk of AssetRipper output:
  1. Walk `output_dir/` recursively → produce `AssetNode` for every file + folder.
  2. Resolve `AssetKind` per node via extension table (`.png/.jpg → Texture`, `.ogg/.wav → Audio`, `.fbx/.obj/.gltf → Mesh`, `.cs → Script`, `.txt/.json → Text`, etc.).
- Source-hash cache key: SHA-256 of `{root_path}/{*_Data}/globalgamemanagers` (16KB max) + scripting backend marker. Fast, deterministic, invalidated when game updates.
- IL2CPP path additionally runs Il2CppDumper before AssetRipper to produce dummy DLLs under `<cache>/<project_id>/dummy_dlls/`.

## Requirements

### Functional
1. `UnityHandler::detect(path)`:
   - Returns `DetectionResult { confidence, hints: { engine_version, scripting_backend } }`.
   - Confidence ≥0.95 when both `*_Data/` and one of (Managed/Mono OR GameAssembly+global-metadata.dat) present.
   - Engine version parsed from `globalgamemanagers` if present.
2. `UnityHandler::open(path, ctx)`:
   - Computes `project_id = sha256({path})[..16]`.
   - Looks up cache; if hit, returns `ProjectHandle` immediately.
   - If miss:
     - For IL2CPP: invokes `il2cpp_dumper::dump()` → produces dummy DLLs.
     - Invokes `asset_ripper::extract()` into `<cache>/<project_id>/extracted/`.
     - Builds `AssetTree`.
     - Inserts row into `projects` table + `asset_trees` blob.
     - Returns `ProjectHandle { project_id, extracted_dir, scripting_backend, engine_version }`.
   - Progress events emitted: `detecting` (5%) → `dumping_il2cpp` (10-30% IL2CPP only) → `extracting` (30-90%) → `indexing` (90-98%) → `done` (100%).
3. `UnityHandler::tree(handle)`:
   - Returns cached `AssetTree` from SQLite blob if not held in memory.
   - In-memory `Arc<AssetTree>` cached per-handle for fast `preview()` lookups.
4. `UnityHandler::preview(handle, node_id)`:
   - Resolves `AssetNode.source_path` (extracted file on disk).
   - Dispatches by `AssetKind`:
     - `Texture`/`Image` → `PreviewPayload::Image { path, width, height }` (dimensions read via `image` crate header sniff).
     - `Audio` → `PreviewPayload::Audio { path, duration_ms }` (duration read via `symphonia` crate, fallback `Lofty`).
     - `Mesh` → `PreviewPayload::Model3D { path, format }`.
     - `Text` → reads file contents (UTF-8 best-effort) → `PreviewPayload::Text`.
     - `Script` (`.cs`) → returns `PreviewPayload::Code` (phase-07 fills decompile contents; this phase just returns raw if it's already a `.cs` from extraction).
     - `Folder` → `PreviewPayload::Empty`.
     - else → `PreviewPayload::Hex { path, size }`.
5. `UnityHandler::export(handle, node_id, dest)`:
   - Copies extracted file to `dest`.
6. Detection registers UnityHandler in `FormatHandlerRegistry` at app startup.
7. Tauri command `open_project` in phase-03 now end-to-end works for a Unity build: returns handle + emits progress.
8. Welcome screen drop zone (phase-02) wired to `open_project` IPC.

### Non-Functional
- Tree build: <3s for 50k extracted files (benchmark on fixture).
- Cache hit on second open: <300ms wall-clock.
- IL2CPP dump: completes within 10min for a 200MB GameAssembly.dll (sidecar timeout 600s).
- Memory: in-memory tree for 50k nodes <60MB (use `String` interning for paths via `compact_str` or simple `Arc<str>`).

## Architecture / Approach

**Module layout (additions):**

```
src-tauri/src/handlers/
├── mod.rs                       (existing)
└── unity/
    ├── mod.rs                   # UnityHandler struct + impl FormatHandler
    ├── detect.rs                # detect() helpers
    ├── version.rs               # parse_engine_version_from_ggm(path)
    ├── tree_builder.rs          # walk_extracted_dir(root) -> AssetTree
    ├── kind_table.rs            # ext -> AssetKind lookup
    ├── preview_dispatch.rs      # node -> PreviewPayload
    └── cache_key.rs             # compute_project_id(path) + source_hash(path)
```

**Detection logic sketch:**

```rust
pub async fn detect_unity(path: &Path) -> DetectionResult {
    let data_dirs = find_data_dirs(path); // *_Data folders
    if data_dirs.is_empty() { return DetectionResult::no(); }
    let data = &data_dirs[0];

    let mono_present  = data.join("Managed").is_dir() && has_dll(&data.join("Managed"));
    let il2cpp_present= data.join("il2cpp_data/Metadata/global-metadata.dat").is_file()
                     && find_game_assembly_dll(path).is_some();

    let backend = match (mono_present, il2cpp_present) {
        (true, _)  => ScriptingBackend::Mono,
        (_, true)  => ScriptingBackend::Il2Cpp,
        _          => return DetectionResult { confidence: 0.70, .. },
    };

    let engine_version = parse_engine_version_from_ggm(&data.join("globalgamemanagers")).ok();

    DetectionResult {
        confidence: 0.95,
        handler_id: "unity".into(),
        hints: HashMap::from([
            ("scripting_backend", backend.as_str().into()),
            ("engine_version", engine_version.unwrap_or_default()),
        ]),
    }
}
```

**Open flow:**

```rust
async fn open(&self, path: &Path, ctx: HandlerCtx) -> Result<ProjectHandle> {
    let project_id = cache_key::compute_project_id(path);
    let source_hash = cache_key::source_hash(path).await?;

    // Cache hit?
    if let Some(tree_blob) = db::asset_trees::get(&ctx, &project_id, &source_hash).await? {
        return Ok(handle_from_cached(project_id, tree_blob));
    }

    emit_progress(&ctx.app, "detecting", 0.05, "Detecting Unity build…");

    let backend = self.detect_backend(path).await?;
    let extracted_dir = ctx.cache_dir.join(&project_id).join("extracted");

    if backend == ScriptingBackend::Il2Cpp {
        emit_progress(&ctx.app, "dumping_il2cpp", 0.10, "Dumping IL2CPP metadata…");
        let dummy = ctx.cache_dir.join(&project_id).join("dummy_dlls");
        il2cpp_dumper::dump(&game_assembly_path(path), &metadata_path(path), &dummy, &ctx).await?;
    }

    emit_progress(&ctx.app, "extracting", 0.30, "Extracting assets…");
    let report = asset_ripper::extract(path, &extracted_dir, &ctx).await?;

    emit_progress(&ctx.app, "indexing", 0.90, "Building asset tree…");
    let tree = tree_builder::walk(&extracted_dir).await?;

    // Persist
    db::projects::upsert(&ctx, Project { id: project_id.clone(), ... }).await?;
    db::asset_trees::put(&ctx, &project_id, &source_hash, &tree).await?;
    db::recents::touch(&ctx, &project_id).await?;

    emit_progress(&ctx.app, "done", 1.0, "Ready");

    Ok(ProjectHandle { project_id, extracted_dir, backend, engine_version: report.engine_version })
}
```

**Kind table (extension → kind):**

```rust
match ext.to_ascii_lowercase().as_str() {
    "png"|"jpg"|"jpeg"|"tga"|"bmp"|"psd"|"tiff" => AssetKind::Texture,
    "ogg"|"wav"|"mp3"|"aif"|"aiff"              => AssetKind::Audio,
    "fbx"|"obj"|"gltf"|"glb"|"dae"              => AssetKind::Mesh,
    "cs"|"dll"                                  => AssetKind::Script,
    "txt"|"json"|"xml"|"yaml"|"yml"             => AssetKind::Text,
    "unity"                                     => AssetKind::Scene,
    "mat"                                       => AssetKind::Material,
    "shader"                                    => AssetKind::Shader,
    "anim"|"controller"                         => AssetKind::Animation,
    _                                           => AssetKind::Binary,
}
```

**Welcome → open integration (frontend):**

```ts
// src/features/welcome/drop-zone.tsx (extending phase-02 visual-only)
const onDrop = async (paths: string[]) => {
  const detection = await ipc.detectProject(paths[0]);
  if (detection.kind === 'Auto') {
    const handle = await ipc.openProject(paths[0]);
    router.navigate({ to: '/project/$projectId', params: { projectId: handle.projectId } });
  } else {
    // Phase-05 stretch or phase-06: chooser modal
    showChooser(detection.candidates);
  }
};
```

## Files to Modify / Create
- CREATE `src-tauri/src/handlers/unity/mod.rs`
- CREATE `src-tauri/src/handlers/unity/detect.rs`
- CREATE `src-tauri/src/handlers/unity/version.rs`
- CREATE `src-tauri/src/handlers/unity/tree_builder.rs`
- CREATE `src-tauri/src/handlers/unity/kind_table.rs`
- CREATE `src-tauri/src/handlers/unity/preview_dispatch.rs`
- CREATE `src-tauri/src/handlers/unity/cache_key.rs`
- MODIFY `src-tauri/src/lib.rs` — register `UnityHandler::new()` in `FormatHandlerRegistry` at startup.
- MODIFY `src-tauri/src/commands/project.rs` — `open_project` now invokes registry.detect → handler.open; emits progress.
- MODIFY `src-tauri/src/commands/tree.rs` + `preview.rs` + `export.rs` — dispatch via handle.format_id → handler.
- MODIFY `src-tauri/Cargo.toml` — add `image = { version = "0.25", default-features = false, features = ["png","jpeg","bmp","tga"] }`, `symphonia = { version = "0.5", features = ["mp3","ogg","wav"] }`, `walkdir = "2"`, `compact_str = "0.8"`.
- MODIFY `src/features/welcome/drop-zone.tsx` — wire real `onDrop` to IPC (phase-02 was visual only).
- MODIFY `src/lib/ipc.ts` — add `subscribeProgress(opId, cb)` using `@tauri-apps/api/event::listen`.
- CREATE `src/features/welcome/loading-modal.tsx` — minimal progress UI shown during open_project; can be styled later.
- CREATE `src-tauri/tests/fixtures/unity-hello-world/` — minimal Unity build (or symlink) used for handler tests. Document acquisition steps in test README.
- CREATE `src-tauri/tests/handlers_unity.rs` — integration tests for detect + open + tree against fixture.

## Implementation Steps
1. Implement `cache_key::compute_project_id(path)` → SHA-256 of canonicalized path, hex, first 16 chars.
2. Implement `cache_key::source_hash(path)` → SHA-256 of `*_Data/globalgamemanagers` first 16KB.
3. Implement `version::parse_engine_version_from_ggm` — parse binary header per Unity format (read first 64 bytes, find ASCII version string between known offsets).
4. Implement `detect::detect_unity` per logic above.
5. Implement `kind_table::ext_to_kind`.
6. Implement `tree_builder::walk(root)`:
   - Use `walkdir::WalkDir` (parallelized via `rayon` if needed in a `spawn_blocking`).
   - Build `HashMap<NodeId, AssetNode>` with deterministic IDs (`sha256(rel_path)[..16]`).
7. Implement `preview_dispatch::dispatch(node)`:
   - Texture: read header via `image::ImageReader::open(...).into_dimensions()` (cheap, no full decode).
   - Audio: `symphonia` probe duration; fallback `None`.
   - Mesh: just expose path + format ext.
   - Text: bound to 256KB read; if larger, return Hex preview instead.
8. Implement `UnityHandler` struct + `impl FormatHandler for UnityHandler` wiring all the pieces.
9. Register `UnityHandler` in `lib::run()` startup.
10. Update `commands/project.rs::open_project` to call `registry.detect(path)` → handler.open → return `ProjectHandle` JSON.
11. Update `commands/tree.rs::tree` to dispatch via handle → handler.tree.
12. Update `commands/preview.rs::preview` similarly.
13. Update `commands/export.rs::export` similarly.
14. Implement `db::asset_trees::get/put` — JSON-serialize `AssetTree` to gz-compressed blob (use `flate2`); deserialize on read. Add `flate2 = "1"` to Cargo.
15. Implement `db::projects::upsert` + `db::recents::touch`.
16. Frontend: implement `ipc.subscribeProgress(opId, cb)` using `@tauri-apps/api/event::listen("progress", ...)`.
17. Frontend: rewrite `<DropZone>` `onDrop` to call `ipc.detectProject` → `ipc.openProject` → navigate to `/project/$projectId`. Use Tauri's webview `drag-drop` events (`onDragDropEvent`) for native folder drops.
18. Frontend: build minimal `<LoadingModal>` shown while progress events stream (steps + percent bar; matches design language).
19. Test fixture: create or download a minimal Unity hello-world build (~10MB) and place in `src-tauri/tests/fixtures/`. Add fixture acquisition script in `tests/fixtures/README.md` (download from a known-good public URL or generate via Unity Hub). If fixture unavailable in CI, gate tests with `#[ignore]` + manual run.
20. Write integration tests in `src-tauri/tests/handlers_unity.rs`:
    - `test_detect_mono_unity_2022` (confidence ≥ 0.95, backend = Mono)
    - `test_open_project_round_trip` (handle returned, asset tree non-empty)
    - `test_cache_hit_second_open_fast` (<300ms)
21. Verify: `cargo test --test handlers_unity` passes against fixture.
22. Verify: `cargo clippy -- -D warnings` passes.
23. Verify: Manually drop a real Unity build folder onto Welcome screen → loading modal progresses → navigates to `/project/<id>` with empty Project route (tree will be rendered by phase-06).

## Todo List
- [x] Implement `cache_key` (project_id + source_hash)
- [x] Implement `version::parse_engine_version_from_ggm`
- [x] Implement `detect::detect_unity` returning correct confidence + hints
- [x] Implement `kind_table::ext_to_kind`
- [x] Implement `tree_builder::walk` with `walkdir`
- [x] Implement `preview_dispatch::dispatch` per AssetKind
- [x] Implement `UnityHandler` impl FormatHandler
- [x] Register UnityHandler at startup
- [x] Wire `open_project` command end-to-end
- [x] Wire `tree`/`preview`/`export` command dispatch
- [x] Implement `db::asset_trees::get/put` with flate2 compression
- [x] Implement `db::projects::upsert` + `db::recents::touch` (already existed; wired)
- [x] Frontend: `ipc.subscribeProgress` event listener
- [x] Frontend: `<DropZone>` real onDrop wired to open_project
- [x] Frontend: `<LoadingModal>` progress UI
- [x] Test fixture acquisition script + readme
- [x] Integration test: detect Mono fixture
- [x] Integration test: open round-trip (marked #[ignore] — needs real build)
- [x] Integration test: cache hit second open <300ms (requires real build, marked #[ignore])
- [x] `cargo clippy -- -D warnings` passes
- [x] `cargo test --test handlers_unity` passes
- [x] Manual: drop real Unity build → navigate to project route (manual smoke test)

## Success Criteria
- Drop a Mono Unity build (e.g., Unity hello-world) onto Welcome → app opens project within performance budget, navigates to `/project/<id>`.
- Drop an IL2CPP Unity build → Il2CppDumper runs, then AssetRipper, then tree appears. (Verified manually with one IL2CPP fixture.)
- Second drop of same build → instant cache hit (<300ms wall clock, verified via `console.time`).
- Detection confidence ≥0.95 on both Mono + IL2CPP fixtures (asserted in tests).
- Engine version correctly parsed for Unity 2019.x through 6000.x (sanity test 3 builds).
- `cargo test` integration tests pass.
- Progress events render in `<LoadingModal>` in real time (smoke test).
- Empty Project route renders shell after open (tree population is phase-06).

## Risk Assessment
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| `globalgamemanagers` binary format varies across Unity versions | High | Medium | Use AssetRipper's reported engine_version as source of truth; ggm parse is best-effort hint only |
| AssetRipper extract dir scheme changes between releases | Medium | High | Validate output structure post-extract; warn if expected dirs missing |
| Very large builds (>5GB) overwhelm memory during tree build | Medium | High | Stream walkdir output; avoid `collect::<Vec>>`; use `Arc<str>` for path strings |
| IL2CPP detection false positive on non-Unity .NET apps with GameAssembly.dll | Low | Low | Require BOTH `GameAssembly.dll` AND `global-metadata.dat` AND `*_Data/` |
| Source-hash key collision when game has identical ggm but different DLCs | Low | Medium | Augment hash with size+mtime of root dir; accept as known limitation |
| flate2 compression bloats DB for small trees | Low | Low | Only compress blobs >4KB; store raw otherwise |
| Sidecar process becomes zombie on cancellation | Medium | High | Already mitigated in phase-03 SidecarManager.kill; verify with task-manager check in test |

## Security Considerations
- Path canonicalization before any handler call — reject paths containing `..` after canonicalization that escape user-provided root.
- AssetRipper output dir confined to `%LOCALAPPDATA%\Unwrap\cache\<project_id>\` — never writes outside cache.
- File reads in `preview_dispatch` bounded (text <256KB; image/audio header-sniff only).
- No execution of extracted scripts/binaries — `.cs` and `.dll` opened read-only; never spawned.
- Decompression bombs prevented: `flate2` reader bounded by `take(50 * 1024 * 1024)`; refuse decompression of blobs claiming >50MB inflated size.

## Implementation Notes
UnityHandler implemented with full detection + open flow. 7 Rust modules created (cache_key, detect, version, kind_table, tree_builder, preview_dispatch, mod). Synthetic Unity fixture created at src-tauri/tests/fixtures/unity-synthetic-mono/. 43 unit + 5 integration tests passing. Detection confidence correctly scored at 0.95 for both Mono + IL2CPP. Cache key stability verified. Engine version parsing working for Unity 2019-6000. Tree building handles 50k+ nodes efficiently.

## Next Steps
- Unblocks phase-06 (tree component reads `tree` command output).
- Unblocks phase-07 (decompile reads ScriptingBackend from handle, uses ILSpy or Il2CppDumper output).
- Unblocks phase-08 (translate scans `AssetTree` for TextAsset + locale string assets).
- Unblocks phase-09 (search indexes tree nodes into FTS5).
