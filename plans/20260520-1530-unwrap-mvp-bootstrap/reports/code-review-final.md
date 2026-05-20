# Code Review — Unwrap v0.1

**Date:** 2026-05-20
**Verdict:** SHIP-WITH-CAVEATS
**Quality score:** 7.5/10

Sampled 13 files across security / reliability / frontend hotspots. Code is well-organized, error types are typed end-to-end, zip-slip is consciously addressed, and the FTS5 query builder properly escapes user input. The caveats below are real production risks that should be fixed in v0.1.x, but none are exploit-grade against a local-only desktop tool.

## P1 Critical (block)

### 1. Downloaded sidecar executes without checksum verification
- `src-tauri/binaries/sidecar-manifest.json:31` — `"sha256": "TBD"` for `il2cpp-dumper`
- `src-tauri/src/sidecar/manifest.rs:119-122` and `:157-158` — `verify_sha256*` short-circuits to `Ok(true)` when value is `"TBD"`
- **Impact:** `installer.rs:98-103` calls `verify_sha256_bytes` → returns true → arbitrary bytes from `download_url` get extracted and executed. The HTTPS-only check (`installer.rs:47`) is the *only* integrity gate. A GitHub release-asset MITM or a takeover of the upstream tag would result in arbitrary code execution under the user account.
- **Fix:** Either commit a real SHA-256 before shipping, or refuse to install when `sha256 == "TBD"`. The skip-on-placeholder behavior is a dev convenience that must not survive into release.

### 2. Sidecar cancellation is a no-op
- `src-tauri/src/sidecar/mod.rs:50-53` — PID is recorded as `0` placeholder; comment says "deferred to phase-04" but we're at phase-10
- `src-tauri/src/sidecar/mod.rs:74-78` — `kill()` on PID=0 logs and returns `Ok(())` without killing anything
- `src-tauri/src/sidecar/mod.rs:106` — `cleanup_all()` skips PID=0 entries
- **Impact:** Frontend `cancel` IPC and app-shutdown cleanup do nothing. A long-running AssetRipper / IL2CPP dump cannot be cancelled. On app close, child processes may be orphaned (mitigated only by `kill_on_drop(true)` in `spawn.rs:71`, which fires when the parent's `Child` is dropped — but the `Child` lives inside the tokio task at `spawn.rs:90`, so it survives `SidecarManager::cleanup_all`).
- **Fix:** Capture `child.id()` before consuming `child` into the tokio task; pass real PID back to `SidecarManager`. Alternatively, hand back a `tokio::sync::oneshot::Sender<()>` for cooperative cancel that the reader task observes alongside `lines.next_line()`.

## P2 High (fix in v0.1.x)

### 3. `read_file_chunk` allows arbitrary file read
- `src-tauri/src/commands/read.rs:18-47` — canonicalizes whatever path the frontend sends and reads `len ≤ 1 MiB` bytes
- `src-tauri/capabilities/default.json:5-9` — only `core:default` + `opener:default`; no fs scope
- **Impact:** Any compromised renderer (XSS via untrusted asset content rendered through Monaco/markdown — unlikely but possible) can exfiltrate any user file via IPC. The 1 MiB cap is a rate-limit, not a scope check.
- **Fix:** Restrict to paths whose canonical form has the project cache dir or active project root as a prefix. Store opened-project roots in app state and validate on every call.

### 4. Search-on-type race in command palette
- `src/features/command-palette/command-palette.tsx:134-145` — `useEffect` calls `ipc.search` without an AbortController or sequence guard
- **Impact:** Fast typing fires N overlapping FTS calls. Out-of-order responses overwrite newer results with stale ones, producing flickering and incorrect dropdowns.
- **Fix:** Track a request sequence id (or use `useRef` for a "latest query" sentinel) and discard responses whose query no longer matches the current input. Better: AbortController wired through `ipc.search`.

### 5. Debounced save captures stale `targetLocale`
- `src/features/project/translate/translate-row.tsx:37-51` — `useDebouncedCallback` closes over `targetLocale` but it isn't in deps and the debounce isn't flushed/cancelled on change
- **Impact:** User types in row, switches target locale within 500 ms → pending save fires against the **previous** locale, silently misfiling the translation.
- **Fix:** Include `targetLocale` in `useDebouncedCallback`'s `deps` arg, and `debouncedSave.flush()` (or `.cancel()`) on `targetLocale` change. Also cancel in a `useEffect` cleanup on unmount.

### 6. Unbounded download buffer
- `src-tauri/src/sidecar/installer.rs:204-211` — `body.extend_from_slice(&chunk)` accumulates the full download into RAM with no ceiling, no `Content-Length` sanity check
- **Impact:** Compromised or misconfigured `download_url` returning a huge body OOMs the app. Bundled tools are unaffected; only applies to the il2cpp-dumper download path.
- **Fix:** Read `response.content_length()` (line 203) and reject before streaming if it exceeds, say, 200 MiB. While streaming, also abort if `downloaded` crosses the cap.

### 7. Blocking canonicalize on async path
- `src-tauri/src/handlers/unity/mod.rs:75` — `std::fs::canonicalize` directly in `async fn open`
- `src-tauri/src/handlers/unity/cache_key.rs:14` — `compute_project_id` also uses `std::fs::canonicalize` and is called from async contexts
- **Impact:** Stalls the tokio runtime worker. Worst case is a network-mounted project path that takes seconds to resolve, blocking other commands and event emission for that interval.
- **Fix:** Wrap in `tokio::task::spawn_blocking` or use `tokio::fs::canonicalize` (already used in `read.rs:27` — be consistent).

### 8. DB mutex held across background indexing
- `src-tauri/src/handlers/unity/mod.rs:219-228` — fire-and-forget task takes `self.db.lock().await` then does `block_in_place(|| index_assets_sync(...))`
- **Impact:** All other DB-touching commands serialize behind asset-indexing for the duration. On a multi-thousand-node tree, no other reads can land. Failures are silently logged, never surfaced.
- **Fix:** Either yield the mutex in chunks (commit-per-batch and re-acquire) or run indexing on a dedicated single-threaded executor that owns the connection.

## P3 Medium (track for v0.2)

### 9. `extract_zip` zip-slip check covers entry name but not join target
- `src-tauri/src/sidecar/installer.rs:283-290` — checks `raw_name`, then writes to `dest_dir.join(entry_name)` where `entry_name` is *caller-provided* (manifest's `install_subpath`)
- Manifest is trusted, but defense-in-depth: also canonicalize `out_path` and assert it starts with `dest_dir`.

### 10. `extract_zip` matches first hit only
- `src-tauri/src/sidecar/installer.rs:266-307` — first matching entry wins; doesn't detect duplicate basenames
- A crafted zip with benign + malicious files sharing a basename could pivot on entry order. Bundled use case is OK, but is fragile if manifest entries ever expand.

### 11. AppError messages leak filesystem paths to frontend
- `src-tauri/src/domain/error.rs:14-30` + `:56-63` — variants embed raw paths/error strings into `message`, sent to the renderer as-is
- For a local-only tool, low risk; for screen-recordings / crash reports, user paths (which contain Windows usernames) leak.
- Consider stripping `cache_dir` prefix and replacing with a sentinel in the serialized `message`, keep full path in the `tracing` log only.

### 12. FTS5 prefix queries have no minimum length
- `src-tauri/src/db/repo/search.rs:111-122` — `"a"` → `"a"*` triggers a near-full-index scan
- Command palette debounce is 120 ms (`command-palette.tsx:119`); single-letter queries will hit. Add `if tok.len() < 2 { continue; }` or set a minimum at the call site.

### 13. Theme registration ref is per-component instance
- `src/features/project/preview/code-preview/monaco-host.tsx:22-30` — `themeRegistered` is a `useRef`; if two `MonacoHost`s mount in the same window (split view, future), each re-registers. Idempotent today, but should hoist to module-level boolean.

### 14. Command-palette dispatch casts through `Parameters<typeof …>`
- `src/features/command-palette/command-palette.tsx:170-185` — type-erases tree-node payload by chaining function-parameter introspection
- If the store signature ever changes the assertion compiles but mis-shapes node data. Define an explicit `TreeNode` type and import it from the store module.

## Strengths

- Typed end-to-end error model — `AppError` with stable `code` + serialize impl (`src-tauri/src/domain/error.rs`).
- Conscious zip-slip handling with a dedicated test (`installer.rs:354-361`).
- Streaming SHA-256 in 64 KiB chunks for bundled binary verification (`manifest.rs:130-141`) — won't blow memory on large tools.
- Source-weighted BM25 with proper FTS5 quote-escaping (`search.rs:111-122`).
- Atomic batch inserts behind transactions in all three `insert_*` paths in `search.rs` — no torn indexes.
- `kill_on_drop(true)` on child process (`spawn.rs:71`) — partial mitigation for the orphan-child issue.
- Frontend lazy-loads Monaco (~2.5 MB) via dynamic import — keeps welcome-screen bundle lean.
- HTTPS-only download enforcement (`installer.rs:47-52`).

## Recommendation

Ship v0.1 internal / beta builds with caveats. Before any public release:
1. Resolve P1 #1 (commit real `il2cpp-dumper` SHA-256) — single-line manifest fix.
2. Resolve P1 #2 (wire real PID into `SidecarManager`) — ~30-line refactor in `spawn.rs` + `mod.rs`.
3. Address P2 #3 (scope `read_file_chunk`) — must be done before any untrusted-content rendering ships.
4. Schedule remaining P2 for v0.1.1 in a single hardening PR.

P3 items are quality-of-life and can be folded into v0.2 planning.

## Unresolved Questions

- Is the il2cpp-dumper SHA-256 truly TBD because the upstream artifact is not yet pinned, or because the dev convenience was forgotten? Affects whether fix #1 is a build-pipeline change or a manifest commit.
- Where does the "active project root" live for use by a scoped `read_file_chunk` (P2 #3) — is there already a registry in `tab-store` / handler context, or does it need to be created?
- Cancellation UX: when `cancel` fires and the sidecar truly dies mid-`extract`, what's the recovery path on disk? `tree_builder::walk` would still see partial output.
