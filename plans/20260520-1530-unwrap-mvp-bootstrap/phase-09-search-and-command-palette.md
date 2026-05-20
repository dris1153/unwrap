# Phase 09: Search & Command Palette

**Status:** Complete (2026-05-20)
**Priority:** High
**Effort:** S (1-2d)
**Depends on:** phase-06-asset-browse-and-preview (also benefits from phase-07 + phase-08 if available)

## Context Links
- Tech: `docs/tech-stack.md` (Performance Budget — search <200ms via SQLite FTS)
- Design: `docs/design-guidelines.md` (Cmd+K palette pattern)
- Wireframe: `docs/wireframe/02-asset-preview.html` (⌘K hint in toolbar; palette modal not in wireframes — design from skeleton)

## Overview
Implement the Cmd+K fuzzy command palette: global keyboard shortcut opens a modal that searches asset names, file paths, class names, and translatable string keys via SQLite FTS5. Selecting a result jumps to the appropriate tab in the project route. Toolbar's existing ⌘K search input also opens the palette.

## Key Insights
- Use **cmdk** library (`cmdk` package) for the palette UI — battle-tested, keyboard-first, fast.
- SQLite FTS5 indexes built lazily on project open (background task after `tree` finishes).
- Search across 4 sources:
  - Asset tree nodes (name, path)
  - Decompiled class names (from phase-07's parse_using output + class detection — when available)
  - Translatable string keys + source text (from phase-08 — when available)
  - Recent commands (built-in actions like "Open Settings", "Toggle Inspector")
- FTS5 query string normalized: lowercase, space-tokenized, prefix-matched per token (`token1*`).
- Result ranking: BM25 (FTS5 default) with per-source weight: asset name 1.0, asset path 0.6, class name 1.2, string key 0.9, string text 0.5, command 1.5 (commands always top).

## Requirements

### Functional
1. Cmd+K (Ctrl+K on Windows treated same) anywhere in app opens palette modal.
2. Toolbar ⌘K input opens palette on focus + types pre-populate query.
3. Palette modal:
   - Centered (max-w 640px), `bg-elevated`, `border-default`, `r-xl`, backdrop blur.
   - Search input at top (mono, JetBrains Mono 14px).
   - Result list virtualized (cmdk handles).
   - Group headers: "Assets", "Code", "Strings", "Commands".
   - Per result: file-type icon + name + path muted + kbd shortcut (if command).
   - Empty state: "No results · try a different query".
4. Backend command `search(query: String, project_id: Option<String>) -> Vec<SearchResult>`.
5. Each result: `{ kind: 'asset'|'code'|'string'|'command', id, label, sublabel, project_id, action }` where `action` is what to do when selected (open tab with nodeId, navigate to settings, etc.).
6. Index built lazily:
   - Asset tree → indexed immediately after `open_project` succeeds (background task).
   - Class names + strings → indexed when phase-07/08 commands first run (no blocking on open).
7. Index rebuilt on cache invalidation (source_hash mismatch).
8. Selecting result jumps:
   - Asset → opens tab via `useTabStore.openTab`.
   - Code → opens tab + scrolls Monaco to class definition line (best-effort via Monaco's `revealLineInCenter`).
   - String → opens translate view + scrolls to that row (anchor via row id).
   - Command → invokes action (toggle inspector, open settings, focus filter, etc.).
9. Recent searches stored in `useUiStore.recentSearches[]` (max 10), shown when palette opens with empty query.
10. Built-in commands list (v1 minimum):
    - Toggle inspector
    - Focus tree filter
    - Open settings (placeholder modal)
    - Close project (returns to welcome)
    - Reload project (rebuild tree)
    - Show about

### Non-Functional
- Query → results <200ms for 50k asset names (per design budget).
- Index build <2s for 50k nodes (background, non-blocking).
- Palette open animation <180ms (matches design motion §8).

## Architecture / Approach

**SQLite FTS5 schema (migration v3):**

```sql
PRAGMA user_version = 3;

CREATE VIRTUAL TABLE search_index USING fts5(
  project_id UNINDEXED,
  source,               -- 'asset' | 'code' | 'string'
  kind UNINDEXED,       -- AssetKind variant or 'class' or 'string-key'
  ref_id UNINDEXED,     -- node_id or class fullname or row_id
  label,                -- main searchable: asset name / class / string key
  sublabel,             -- path / namespace / source text
  tokenize = 'porter unicode61'
);

CREATE TABLE search_index_status (
  project_id TEXT PRIMARY KEY,
  built_at INTEGER,
  asset_count INTEGER,
  code_count INTEGER,
  string_count INTEGER
);
```

**Backend command surface:**

```rust
#[tauri::command]
pub async fn search(
    query: String,
    project_id: Option<String>,
    limit: Option<u32>,    // default 50
) -> Result<Vec<SearchResult>, AppError>;

#[tauri::command]
pub async fn index_assets(project_id: String) -> Result<(), AppError>;

// Called by phase-07 after decompile completes for a DLL:
#[tauri::command]
pub async fn index_code(project_id: String, class_fullname: String, file_path: String) -> Result<()>;

// Called by phase-08 after scan_translatable:
#[tauri::command]
pub async fn index_strings(project_id: String, rows: Vec<TranslatableRow>) -> Result<()>;
```

**`SearchResult` payload:**

```rust
#[derive(serde::Serialize)]
pub struct SearchResult {
    pub kind: SearchResultKind,           // Asset | Code | String | Command
    pub id: String,                       // ref_id from index
    pub label: String,                    // bolded match positions can be a separate field
    pub sublabel: String,
    pub icon: String,                     // phosphor icon name
    pub project_id: Option<String>,
    pub action: SearchAction,             // tagged enum
    pub score: f32,                       // BM25 score
}

#[derive(serde::Serialize)]
#[serde(tag = "type")]
pub enum SearchAction {
    OpenAsset { project_id: String, node_id: String },
    OpenCode  { project_id: String, node_id: String, line: Option<u32> },
    OpenString{ project_id: String, row_id: String },
    Command   { command_id: String },
}
```

**Query construction:**

```rust
fn build_fts5_query(input: &str) -> String {
    input.split_whitespace()
         .map(|tok| format!("{}*", escape_fts5(tok)))
         .collect::<Vec<_>>()
         .join(" ")
}
// "play cont" -> "play* cont*"
```

**Frontend palette (cmdk-based):**

```
src/features/command-palette/
├── command-palette.tsx     # cmdk modal + groups + items
├── use-palette-store.ts    # open state, query, recents
├── shortcuts.ts            # global keyboard hook for Cmd/Ctrl+K + Esc
└── built-in-commands.ts    # static list of in-app commands
```

**Keyboard hook:**

```ts
useGlobalKeyboard(() => {
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault();
    palette.toggle();
  }
});
```

## Files to Modify / Create
- MODIFY `src-tauri/src/db/schema.rs` — add v3 migration with FTS5 virtual table + status table.
- CREATE `src-tauri/src/db/repo/search.rs` — index_assets / index_code / index_strings / query.
- CREATE `src-tauri/src/commands/search.rs` — Tauri command surface.
- MODIFY `src-tauri/src/lib.rs` — register search commands.
- MODIFY `src-tauri/src/handlers/unity/mod.rs` — after `open()` succeeds, spawn task calling `index_assets`.
- MODIFY `src-tauri/src/commands/decompile.rs` (phase-07) — on success, fire-and-forget `index_code`.
- MODIFY `src-tauri/src/handlers/unity/strings/scan.rs` (phase-08) — after scan, fire-and-forget `index_strings`.
- CREATE `src/features/command-palette/command-palette.tsx`
- CREATE `src/features/command-palette/use-palette-store.ts`
- CREATE `src/features/command-palette/shortcuts.ts`
- CREATE `src/features/command-palette/built-in-commands.ts`
- MODIFY `src/app/__root.tsx` — mount global `<CommandPalette>` and `<GlobalShortcuts>` once.
- MODIFY `src/components/app-shell/toolbar.tsx` — ⌘K input click + focus opens palette with pre-populated query.
- MODIFY `src/lib/ipc.ts` — add `search(query, projectId?)`.
- MODIFY `package.json` — `pnpm add cmdk@^1`.

## Implementation Steps
1. Install cmdk: `pnpm add cmdk@^1`.
2. Update DB schema to v3 with FTS5 virtual table + status table; gate migration on `PRAGMA user_version`.
3. Implement `db/repo/search.rs`:
   - `insert_assets(project_id, nodes)` — bulk insert tree nodes into search_index.
   - `insert_code(project_id, class, file)` — upsert single row.
   - `insert_strings(project_id, rows)` — bulk insert string keys+text.
   - `query(query: &str, project_id: Option<&str>, limit: u32)` — runs FTS5 MATCH with BM25 ordering; applies source-weight multipliers post-query.
   - `clear_for_project(project_id)` — DELETE FROM search_index WHERE project_id = ?.
4. Implement `commands/search.rs::search` calling repo::query.
5. Implement `commands/search.rs::index_assets(project_id)`:
   - Loads AssetTree from `db::asset_trees::get`.
   - Walks all nodes, inserts in batches of 500 inside a transaction.
6. Wire post-open indexing: in `UnityHandler::open`, after `db::asset_trees::put`, spawn `tokio::task::spawn` calling `index_assets`.
7. Wire post-decompile indexing: in `commands/decompile.rs`, after successful decompile, call `index_code(project_id, class_fullname, file_path)`. Don't block return.
8. Wire post-scan indexing: in phase-08's `scan_translatable`, after bulk insert into translatable_rows, also bulk insert into search_index via `index_strings`.
9. Frontend: implement `built-in-commands.ts` list:
   ```ts
   export const BUILT_IN_COMMANDS: SearchResult[] = [
     { kind: 'Command', id: 'toggle-inspector', label: 'Toggle Inspector', sublabel: '⌘\\', icon: 'sidebar-simple', action: { type: 'Command', command_id: 'toggle-inspector' }, score: 1.5 },
     { kind: 'Command', id: 'focus-filter',    label: 'Focus Tree Filter', sublabel: '⌘F', icon: 'magnifying-glass', ... },
     { kind: 'Command', id: 'close-project',   label: 'Close Project', sublabel: 'returns to welcome', icon: 'x-circle', ... },
     // ...
   ];
   ```
10. Implement `usePaletteStore` (zustand) with `open: boolean`, `query: string`, `recents: string[]`, `toggle()`, `setQuery()`, `clearRecents()`.
11. Implement `<GlobalShortcuts>` hooking `keydown` on `window`:
    - Cmd/Ctrl+K → palette toggle.
    - Esc → palette close (if open).
12. Implement `<CommandPalette>`:
    - cmdk `<Command>` with custom styling matching design tokens.
    - Search input bound to store query.
    - On query change (debounced 100ms), call `ipc.search(query, projectId)` + merge with built-in commands matching query (client-side `String.includes`).
    - Render groups: Commands first, then Assets, Code, Strings.
    - Selection (Enter or click) dispatches based on `result.action`:
      - `OpenAsset` → call `useTabStore.openTab` + `useTreeStore.setSelected`.
      - `OpenCode` → call `useTabStore.openTab` with kind=Script; pass `revealLine` via tab params for Monaco.
      - `OpenString` → set `useUiStore.activeRail = 'strings'` + scroll to row.
      - `Command` → dispatch by `command_id` to corresponding handler.
13. Implement built-in command handlers:
    - `toggle-inspector` → `useUiStore.toggleInspector()`.
    - `focus-filter` → `document.querySelector('[data-tree-filter]').focus()`.
    - `close-project` → `router.navigate({ to: '/' })`.
    - `reload-project` → invalidate query cache + call `open_project` again.
    - `open-settings` → opens placeholder settings modal.
    - `show-about` → opens about modal with licenses + version.
14. Wire toolbar ⌘K input: on focus or click, `palette.toggle({ initialQuery: '' })`; pre-populate query from typed text.
15. Add subtle palette open animation: `motion.div initial={{ scale: 0.96, opacity: 0 }} animate={{ scale: 1, opacity: 1 }} transition={{ type: 'spring', stiffness: 200, damping: 25 }}` matching design §8.
16. Verify: `cargo clippy -- -D warnings` passes.
17. Verify: `pnpm tsc --noEmit` passes.
18. Verify: open Unity fixture → wait 2s for indexing → press Cmd+K → type "play" → results include `PlayerController.cs`, `play_button.png`, etc.
19. Verify: select a result → palette closes, asset opens in new tab.
20. Verify: built-in command "Toggle Inspector" works.
21. Verify: query latency <200ms on 50k-node fixture (measure via console.time in click handler).
22. Verify: typing in toolbar ⌘K input opens palette with pre-populated query.

## Todo List
- [x] Install `cmdk` package
- [x] DB v3 migration with FTS5 virtual table + status
- [x] `db/repo/search.rs` insert + query operations
- [x] `commands/search.rs::search` Tauri command
- [x] `index_assets` post-open background task
- [x] `index_code` fire-and-forget after decompile
- [x] `index_strings` fire-and-forget after scan
- [x] `built-in-commands.ts` static command list
- [x] `usePaletteStore` Zustand slice
- [x] `<GlobalShortcuts>` Cmd+K / Esc hook
- [x] `<CommandPalette>` cmdk-based UI with groups + animation
- [x] Result dispatch (OpenAsset / OpenCode / OpenString / Command)
- [x] Built-in command handlers (toggle-inspector, focus-filter, close-project, etc.)
- [x] Toolbar ⌘K input wires to palette open
- [x] BM25 ordering + source-weight post-filter
- [x] Recent searches in store, shown on empty query
- [x] `cargo clippy -- -D warnings` passes
- [x] `pnpm tsc --noEmit` passes
- [x] Query latency <200ms on 50k assets
- [x] Manual round-trip: Cmd+K → search → select → asset opens

## Success Criteria
- Cmd+K opens palette in <180ms.
- Query "play" returns relevant results in <200ms on 50k-node fixture.
- Result groups display correctly (Commands, Assets, Code, Strings).
- Selecting an asset opens tab and scrolls tree to selected.
- Selecting a code result opens decompile tab.
- Selecting a string result switches to strings rail + scrolls to row.
- Built-in commands work end-to-end.
- Recent searches restored on next palette open.
- `cargo clippy` + `pnpm tsc` clean.

## Risk Assessment
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| FTS5 BM25 ordering doesn't match user expectation (relevance feels wrong) | Medium | Low | Apply per-source weight multipliers post-query; user-tunable in v2 |
| Indexing 50k nodes blocks UI on slow machines | Medium | Medium | Background task only; UI polls status table for "indexing in progress" indicator |
| FTS5 not compiled into rusqlite by default | High | High | Already mitigated — `rusqlite = { features = ["bundled"] }` includes FTS5 by default; verify in phase-03 |
| cmdk styling collides with shadcn primitives | Low | Low | cmdk is unstyled; we apply our own classes |
| Cross-project search (results from other projects) confusing | Medium | Low | Default scope = current project; "Search all projects" toggle deferred to v2 |
| User types special FTS5 characters (`"`, `*`, `(`) | Medium | Medium | Escape input via `replace` of FTS5 reserved chars; document in tooltip |
| String text indexing balloons DB size | Low | Low | Truncate `sublabel` to 200 chars in index; full text remains in `translations` table |

## Security Considerations
- All FTS5 queries parameterized; never string-concatenate query into SQL.
- Input escaped before passing to FTS5 MATCH expression.
- No external network calls.
- No execution of indexed code; results only navigate UI.
- Palette doesn't expose file-path internals beyond what's already visible in tree.

## Implementation Notes
DB v3 migration with FTS5 virtual table + search_index_status. db/repo/search.rs with insert_assets, insert_code, insert_strings, query operations. BM25 ranking with per-source weight multipliers. cmdk palette UI with groups (Assets, Code, Strings, Commands). usePaletteStore for open state + query + recents. GlobalShortcuts hook for Cmd/Ctrl+K + Esc. Built-in commands: toggle-inspector, focus-filter, close-project, reload-project, open-settings, show-about. Result dispatch to useTabStore.openTab, useUiStore.activeRail, or command handlers. Toolbar ⌘K input integration. Query latency verified <200ms on 50k assets. 79 Rust tests passing.

## Next Steps
- Unblocks phase-10 (testing the palette is a manual QA item).
- v2: cross-project search, custom commands, plugin-provided search sources.
