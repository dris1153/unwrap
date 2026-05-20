# Phase 03: Rust Backend Core

**Status:** Complete (2026-05-20)
**Priority:** Critical
**Effort:** L (3-4d)
**Depends on:** phase-02-design-system-and-shell

## Context Links
- Tech: `docs/tech-stack.md` (Plugin Architecture, Format Detection Pipeline)
- Arch: `docs/system-architecture.md` (Core Domain Models, IPC / Communication Patterns)

## Overview
Stand up the Rust backend skeleton: Tauri command surface, the `FormatHandler` async trait + registry, format-detection pipeline (magic/ext/structural), domain models (Project, AssetTree, AssetNode, PreviewPayload), SQLite project cache via `rusqlite`, IPC event channel for progress streaming, and a generic sidecar process manager (spawn / cancel / progress streaming) — without any concrete handler implementation yet (Unity handler is phase-05).

## Key Insights
- All async I/O on `tokio`. All Tauri commands are `async fn` returning `Result<T, AppError>` where `AppError: serde::Serialize`.
- `FormatHandler` is dyn-compatible only when wrapped in `Arc<dyn FormatHandler>`; registry stores `Vec<Arc<dyn FormatHandler>>`.
- SQLite via `rusqlite` (no async wrapper — wrap in `tokio::task::spawn_blocking` at call sites; the db is small).
- Project cache schema versioned via `PRAGMA user_version`; migration is "drop and recreate" for v0.1.
- Sidecar process manager: own crate-local module, returns a `SidecarProcess` handle with `kill()` + `progress_rx` (tokio mpsc).
- Detection pipeline runs `detect()` of every registered handler in parallel via `tokio::join_all`, ranks by confidence, picks max if >0.8 else returns Vec for chooser UI.

## Requirements

### Functional
1. Tauri commands defined and exposed (stubs return mock/error for now):
   - `detect_project(path: String) -> DetectionResult`
   - `open_project(path: String) -> ProjectHandle`
   - `get_project(id: String) -> Project`
   - `list_recents() -> Vec<RecentEntry>`
   - `tree(handle: String) -> AssetTree`
   - `preview(handle: String, node_id: String) -> PreviewPayload`
   - `export(handle: String, node_id: String, dest: String) -> ()`
   - `cancel(operation_id: String) -> ()`
2. `FormatHandler` trait defined per `docs/tech-stack.md` §Plugin Architecture.
3. `FormatHandlerRegistry` holds `Vec<Arc<dyn FormatHandler>>`, exposes `register()` + `detect_all(path)`.
4. Detection pipeline runs magic-bytes -> extension -> structural fallback; results scored 0.0-1.0.
5. SQLite schema created on first run at `%LOCALAPPDATA%\Unwrap\cache.db`:
   - `projects(id TEXT PK, root_path TEXT, format_id TEXT, engine_version TEXT, scripting_backend TEXT, created_at INT, last_opened INT)`
   - `recents(project_id TEXT PK FK, pinned INT DEFAULT 0)`
   - `asset_trees(project_id TEXT PK FK, source_hash TEXT, json_blob BLOB, built_at INT)`
6. IPC event channel: backend emits `progress` events with `{ operation_id, phase, percent, message }`.
7. Sidecar process manager:
   - `spawn(bin_name, args, cancellation_token) -> SidecarProcess`
   - Streams stdout line-by-line, parses JSON if line starts with `{`, emits raw text otherwise.
   - `kill()` sends `taskkill /T /F` on Windows.
   - 5-minute default timeout, configurable.

### Non-Functional
- All commands return errors via typed `AppError` enum; no `unwrap`/`expect` in production paths.
- `cargo clippy -- -D warnings` passes.
- DB writes wrapped in transactions where appropriate.
- Sidecar manager allocates max one tokio task per process (stdout reader); stdin not held.

## Architecture / Approach

**Module layout (`src-tauri/src/`):**

```
src/
├── main.rs
├── lib.rs                # pub mod ...; build()
├── commands/
│   ├── mod.rs            # tauri::generate_handler! macro
│   ├── project.rs        # detect_project, open_project, get_project, list_recents
│   ├── tree.rs           # tree
│   ├── preview.rs        # preview
│   ├── export.rs         # export
│   └── ops.rs            # cancel
├── domain/
│   ├── mod.rs
│   ├── project.rs        # Project, ScriptingBackend, RecentEntry
│   ├── tree.rs           # AssetTree, AssetNode, AssetKind, NodeId
│   ├── preview.rs        # PreviewPayload, ModelFormat
│   ├── detection.rs      # DetectionResult, DetectionCandidate
│   └── error.rs          # AppError (Serialize)
├── handlers/
│   ├── mod.rs            # FormatHandler trait + Registry
│   └── README.md         # how to add a new handler (phase-05 will add unity/)
├── detection/
│   ├── mod.rs            # run_pipeline(path, registry) -> DetectionResult
│   ├── magic.rs          # known magic-byte signatures
│   └── structural.rs     # directory pattern checks
├── sidecar/
│   ├── mod.rs            # SidecarManager, SidecarProcess
│   ├── spawn.rs          # tokio::process::Command wrapper
│   └── progress.rs       # JSON-line parser
├── db/
│   ├── mod.rs            # connect(), migrate(), with_conn()
│   ├── schema.rs         # CREATE TABLE statements
│   └── repo/
│       ├── projects.rs   # CRUD for Project
│       ├── recents.rs    # CRUD for RecentEntry
│       └── asset_trees.rs# get/put AssetTree blob
└── events.rs             # emit_progress(app, payload), event types
```

**`FormatHandler` trait (frozen from `docs/tech-stack.md`):**

```rust
#[async_trait]
pub trait FormatHandler: Send + Sync {
    fn id(&self) -> &str;
    fn display_name(&self) -> &str;
    async fn detect(&self, path: &Path) -> DetectionResult;
    async fn open(&self, path: &Path, ctx: HandlerCtx) -> Result<ProjectHandle>;
    async fn tree(&self, handle: &ProjectHandle) -> Result<AssetTree>;
    async fn preview(&self, handle: &ProjectHandle, node: &NodeId) -> Result<PreviewPayload>;
    async fn export(&self, handle: &ProjectHandle, node: &NodeId, dest: &Path) -> Result<()>;
}

pub struct HandlerCtx {
    pub app: tauri::AppHandle,
    pub operation_id: String,
    pub cancel: CancellationToken,
    pub cache_dir: PathBuf,
    pub sidecar: Arc<SidecarManager>,
}
```

**Detection pipeline:**

```rust
pub async fn detect(path: &Path, reg: &FormatHandlerRegistry) -> DetectionResult {
    let candidates: Vec<DetectionCandidate> = futures::future::join_all(
        reg.handlers().iter().map(|h| async {
            let r = h.detect(path).await;
            DetectionCandidate { handler_id: h.id().into(), result: r }
        })
    ).await;
    candidates.sort_by(|a, b| b.result.confidence.partial_cmp(&a.result.confidence).unwrap());
    if candidates.first().map(|c| c.result.confidence >= 0.8).unwrap_or(false) {
        DetectionResult::Auto(candidates.into_iter().next().unwrap())
    } else {
        DetectionResult::Chooser(candidates)
    }
}
```

**Sidecar process manager interface:**

```rust
pub struct SidecarManager { /* registry of running pids by operation_id */ }

impl SidecarManager {
    pub fn spawn(&self, spec: SidecarSpec) -> Result<SidecarHandle>;
    pub fn kill(&self, op_id: &str) -> Result<()>;
}

pub struct SidecarSpec {
    pub operation_id: String,
    pub bin: String,                          // e.g., "AssetRipper.exe"
    pub args: Vec<String>,
    pub workdir: PathBuf,
    pub timeout: Duration,                    // default 5min
    pub progress_parser: ProgressParser,      // JsonLines | Regex(pat) | Stderr
}

pub struct SidecarHandle {
    pub op_id: String,
    pub stdout_rx: mpsc::Receiver<SidecarMsg>,
    pub exit: oneshot::Receiver<ExitStatus>,
}

pub enum SidecarMsg {
    Progress { percent: f32, message: String },
    Log(String),
    Done,
    Error(String),
}
```

**IPC event payload (Rust → React):**

```rust
#[derive(Serialize, Clone)]
pub struct ProgressPayload {
    pub operation_id: String,
    pub phase: String,         // "extracting" | "indexing" | "decompiling" | ...
    pub percent: Option<f32>,
    pub message: String,
}
```

**SQLite schema v1:**

```sql
PRAGMA user_version = 1;
PRAGMA journal_mode = WAL;

CREATE TABLE projects (
  id TEXT PRIMARY KEY,
  root_path TEXT NOT NULL,
  format_id TEXT NOT NULL,
  engine_version TEXT,
  scripting_backend TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  last_opened INTEGER NOT NULL
);

CREATE TABLE recents (
  project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
  pinned INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE asset_trees (
  project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
  source_hash TEXT NOT NULL,
  json_blob BLOB NOT NULL,
  built_at INTEGER NOT NULL
);

CREATE INDEX idx_recents_pinned ON recents(pinned DESC);
```

## Files to Modify / Create
- MODIFY `src-tauri/Cargo.toml` — add deps: `tokio = { version = "1", features = ["full"] }`, `serde = "1"`, `serde_json = "1"`, `async-trait = "0.1"`, `anyhow = "1"`, `thiserror = "1"`, `rusqlite = { version = "0.32", features = ["bundled"] }`, `futures = "0.3"`, `tokio-util = { version = "0.7", features = ["rt"] }`, `directories = "5"`, `sha2 = "0.10"`, `tracing = "0.1"`, `tracing-subscriber = "0.3"`.
- MODIFY `src-tauri/src/main.rs` — call `lib::run()`.
- CREATE `src-tauri/src/lib.rs` — `pub fn run()`, registers commands, initializes db + sidecar manager + handler registry into Tauri State.
- CREATE `src-tauri/src/commands/mod.rs`, `project.rs`, `tree.rs`, `preview.rs`, `export.rs`, `ops.rs`.
- CREATE `src-tauri/src/domain/{mod,project,tree,preview,detection,error}.rs`.
- CREATE `src-tauri/src/handlers/{mod,README}.rs`.
- CREATE `src-tauri/src/detection/{mod,magic,structural}.rs`.
- CREATE `src-tauri/src/sidecar/{mod,spawn,progress}.rs`.
- CREATE `src-tauri/src/db/{mod,schema}.rs`, `src-tauri/src/db/repo/{projects,recents,asset_trees}.rs`.
- CREATE `src-tauri/src/events.rs`.
- CREATE `src/lib/ipc.ts` — TS wrapper around `@tauri-apps/api/core::invoke` with typed command signatures.
- CREATE `src/lib/types.ts` — TS mirrors of Rust domain types (manually kept in sync; codegen deferred to phase-10).

## Implementation Steps
1. Update `Cargo.toml` deps as listed.
2. Define `AppError` enum with `thiserror::Error` derive + `serde::Serialize` impl (manual; thiserror doesn't auto-serde).
3. Implement domain types (`Project`, `AssetTree`, `AssetNode`, `AssetKind`, `PreviewPayload`, `ScriptingBackend`, `NodeId(String)`, `ProjectId(String)`).
4. Implement `FormatHandler` trait + `FormatHandlerRegistry` (in `handlers/mod.rs`).
5. Implement detection pipeline in `detection/mod.rs` — `run_pipeline(path, registry) -> DetectionResult`.
6. Implement `magic.rs` with known signatures (UnityFS, PE/MZ, ELF, PK/zip). Each returns `Option<&'static str>` handler hint.
7. Implement `structural.rs` — directory pattern checks (deferred to Unity handler in phase-05; stub now).
8. Implement `db/mod.rs` — `connect(app_dir) -> Result<Connection>`, `migrate(&Connection)`, `with_conn(state, |conn| ...)`.
9. Implement repos (`projects.rs`, `recents.rs`, `asset_trees.rs`) — CRUD via `rusqlite` + `serde_json` for blob columns.
10. Implement `events.rs` — `emit_progress(app, payload)`.
11. Implement `sidecar/mod.rs` — `SidecarManager` holding `HashMap<OpId, ChildHandle>`; `spawn` uses `tokio::process::Command`, redirects stdout, spawns reader task that pushes parsed `SidecarMsg` into mpsc.
12. Implement `sidecar/progress.rs` — `JsonLines` parser: if line starts with `{`, parse as `{percent, message}`; else `SidecarMsg::Log`.
13. Implement command stubs in `commands/*.rs`. All return `AppError::NotImplemented` for now except `list_recents` (real query) and `detect_project` (real detection pipeline; will return empty `Chooser` until Unity handler exists).
14. Register commands in `lib::run()` via `tauri::generate_handler![...]`.
15. Initialize state: `app.manage(Arc::new(SidecarManager::new()))`, `app.manage(Arc::new(FormatHandlerRegistry::new()))`, `app.manage(Arc::new(Mutex::new(db_conn)))`.
16. Create `%LOCALAPPDATA%\Unwrap\` dirs on startup via `directories` crate; initialize SQLite.
17. Write `src/lib/ipc.ts` with typed `invoke` wrappers — one fn per command. Mirror types in `src/lib/types.ts`.
18. Verify: `cargo clippy --all-targets -- -D warnings` passes.
19. Verify: `cargo test` runs (unit tests for `detection::magic` only — write 3 sanity tests).
20. Verify: `pnpm tsc --noEmit` passes with imported `src/lib/ipc.ts`.
21. Verify: Open dev app — `list_recents()` invoked from React returns `[]`, db file created at `%LOCALAPPDATA%\Unwrap\cache.db`.

## Todo List
- [x] Add Cargo deps (tokio, rusqlite, async-trait, thiserror, etc.)
- [x] Implement `AppError` with Serialize impl
- [x] Implement domain types (Project, AssetTree, PreviewPayload, etc.)
- [x] Implement `FormatHandler` trait + Registry
- [x] Implement detection pipeline with parallel `detect()` fanout
- [x] Implement magic-byte signatures module
- [x] Implement structural check stub
- [x] Implement SQLite migrations + repos (projects, recents, asset_trees)
- [x] Implement IPC event emit helper
- [x] Implement SidecarManager with kill + timeout + progress mpsc
- [x] Implement JsonLines progress parser
- [x] Wire Tauri command handlers (stubs return NotImplemented except list_recents + detect)
- [x] Register commands + state in `lib::run()`
- [x] Init `%LOCALAPPDATA%\Unwrap\` on startup
- [x] Write typed `src/lib/ipc.ts` wrappers
- [x] Write `src/lib/types.ts` TS mirrors
- [x] `cargo clippy -- -D warnings` passes
- [x] `cargo test` passes (magic-byte tests)
- [x] `pnpm tsc --noEmit` passes
- [x] Dev app boots, db file created, `list_recents` returns `[]`

## Implementation Notes
- Cooked: 2026-05-20 (single dev, --auto bootstrap)
- Verification: `cargo build` pass, `cargo clippy -D warnings` clean, `cargo test` 7/7 pass, `pnpm tsc --noEmit` pass
- Deviations: none — spec fully met
- Files shipped: 28 Rust source files (src-tauri/src/) + 2 TS files (src/lib/ipc.ts, src/lib/types.ts). Includes domain models, FormatHandler trait + registry, detection pipeline with magic-byte signatures, SQLite schema v1, SidecarManager, 8 Tauri commands, JsonLines progress parser

## Success Criteria
- All 8 Tauri commands callable from React with full typing — wrong arg type fails at compile-time.
- DB file `cache.db` exists at `%LOCALAPPDATA%\Unwrap\` after first launch.
- DB schema includes `projects`, `recents`, `asset_trees` tables (verify via `sqlite3` CLI).
- `cargo clippy --all-targets -- -D warnings` exits 0.
- 3+ unit tests pass for `detection::magic` signature checks.
- `list_recents` round-trips through IPC and returns `[]` on first run.
- No `unwrap()`/`panic!` in `src-tauri/src/` outside of tests.

## Risk Assessment
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| `rusqlite` bundled feature pulls full SQLite — slow first build | High | Low | Document; cache target dir |
| `async-trait` adds Box<Future> overhead on hot paths | Low | Low | Detection runs once per project open — overhead negligible |
| Tauri State sharing across commands has lifetime issues | Medium | Medium | Use `Arc<Mutex<_>>` wrappers; document patterns |
| TS types drift from Rust types over time | High | Medium | Add `ts-rs = "9"` dep + codegen task in phase-10; manual sync v1 |
| SidecarManager leaks zombie processes on app crash | Medium | Medium | Register cleanup hook on `RunEvent::Exit` to kill all tracked PIDs |

## Security Considerations
- All file paths from React validated as existing + canonicalized before passing to handlers (prevents path traversal).
- Sidecar binary paths resolved from Tauri sidecar bin dir only — never trust caller-provided binary names (phase-04 reinforces this).
- Subprocess args sanitized: no shell interpolation; pass as `Vec<String>` to `tokio::process::Command` directly.
- SQLite file permissions: rely on Windows ACL of `%LOCALAPPDATA%` (user-scoped).
- No commands accept arbitrary SQL — all queries are static prepared statements.

## Next Steps
- Unblocks phase-04 (sidecar manager + handler registry are the integration points for bundling AssetRipper etc.).
- Unblocks phase-05 (UnityHandler implements `FormatHandler` trait defined here).
- TS IPC wrappers in `src/lib/ipc.ts` are the integration surface for phases 06/07/08.
