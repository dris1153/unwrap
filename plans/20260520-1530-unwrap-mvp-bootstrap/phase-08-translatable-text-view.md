# Phase 08: Translatable Text View

**Status:** Complete (2026-05-20)
**Priority:** High
**Effort:** M (2-3d)
**Depends on:** phase-05-unity-format-handler (parallelizable with phase-06 + phase-07)

## Context Links
- Wireframe: `docs/wireframe/04-translate.html` (translate toolbar, EN→VI columns, status dots, progress bar)
- Tech: `docs/tech-stack.md` (Non-Goals: editing/repacking v1 — translations stored in SQLite, not repacked)
- Arch: `docs/system-architecture.md` (Workflow #4 Translate Text)

## Overview
Detect translatable text/locale assets across an opened Unity project, present a side-by-side translation editor (EN source → target locale), persist user-entered translations in SQLite (read-only viewer constraint — no repack to game files in v1). Matches `04-translate.html` visually. Supports per-string status (translated/pending/review), progress meter, locale switcher, filter, and CSV/JSON export.

## Key Insights
- Translatable strings live in 3 source types in Unity builds:
  1. **TextAsset** files (`.txt` / `.json`) — extracted by AssetRipper to `Assets/.../*.txt`.
  2. **Localization package ScriptableObjects** — `StringTable` and `StringTableCollection` assets exposed as `.asset` YAML by AR.
  3. **Hardcoded strings in DLLs** — extracted via `mono.cecil` or grep — deferred to v2 (mention in inspector as "not indexed").
- Locale detection: file path patterns `**/Locales/<lang>/...` or `**/Localization/<lang>/...` or filename `*_<lang>.json`. Common langs: `en`, `vi`, `fr`, `de`, `ja`, `zh`, `ko`, `es`, `pt`, `ru`.
- Each detected string row gets a stable key = `sha256(source_path + key_in_file)[..16]`.
- Translations stored in SQLite, **never** modify extracted asset files.
- Auto-translate button is a stretch goal — wire as no-op for v1 (placeholder modal "AI translate — coming v2").

## Requirements

### Functional
1. Backend scans the asset tree for translatable items when project opens (lightweight pass, no full file read).
2. New IPC command `list_translatable(handle) -> Vec<TranslatableRow>` exposes rows lazily.
3. Each row: `{ id, source_path, key, locale, source_text, status }`.
4. Sidebar rail's "Strings" tab activates Translate view (replaces center pane content).
5. Translate view matches wireframe 04 lines 412-... :
   - Tab bar showing `localization/strings.json` (mono).
   - Translate toolbar (lines 469-520):
     - Locale source picker (EN), target picker (e.g., VI).
     - Progress bar with `translated/total` count + percentage.
     - Status legend (translated/pending/review).
     - Filter dropdown (All / Translated / Pending / Review).
     - Export dropdown (CSV / JSON / .po).
     - Auto-translate button (cosmetic v1; opens "coming v2" modal).
   - Column header row (key/EN/target/menu — 28px).
   - Row list — virtualized; per row:
     - Status dot (translated=success, pending=warning, review=info).
     - Key + comment.
     - Source EN text (read-only mono).
     - Target translation `<textarea>` editable.
     - Char count + last-edited timestamp.
     - Row actions (mark reviewed, copy source, jump-to-source-file).
6. Editing target text autosaves to SQLite (debounced 500ms).
7. Locale picker fetches available target locales from project (parsed from filename suffixes / dir names) + falls back to ISO-639 list.
8. Progress recomputed on every change.
9. Export to CSV/JSON writes to user-chosen path via Tauri dialog.
10. Translation data isolated per `project_id` — switching projects loads correct dataset.

### Non-Functional
- 5k strings rendered smoothly (virtualized).
- Save latency <50ms per keystroke (UI optimistic update + async DB write).
- Re-open project: prior translations restored within 200ms.

## Architecture / Approach

**Backend additions:**

```
src-tauri/src/
├── commands/
│   ├── translate.rs           # list_translatable, save_translation, list_locales, export
└── handlers/unity/
    └── strings/
        ├── mod.rs
        ├── scan.rs            # walk AssetTree, identify translatable nodes
        ├── parse_textasset.rs # parse .json / .txt / .csv into key→text rows
        ├── parse_stringtable.rs # parse Localization package YAML
        └── locale_detect.rs   # extract locale code from path/filename
```

**Translatable domain model:**

```rust
pub struct TranslatableRow {
    pub id: String,             // stable hash
    pub project_id: String,
    pub source_path: String,    // 'Assets/Localization/en/strings.json'
    pub key: String,            // 'menu.title'
    pub comment: Option<String>,
    pub locale: String,         // 'en'
    pub source_text: String,
}

pub struct TranslationEntry {
    pub row_id: String,
    pub target_locale: String,  // 'vi'
    pub text: String,
    pub status: TranslationStatus,
    pub updated_at: i64,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum TranslationStatus {
    Pending,
    Translated,
    Review,
}
```

**SQLite schema additions (migrated to v2):**

```sql
PRAGMA user_version = 2;

CREATE TABLE translatable_rows (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  source_path TEXT NOT NULL,
  key TEXT NOT NULL,
  comment TEXT,
  source_locale TEXT NOT NULL,
  source_text TEXT NOT NULL,
  detected_at INTEGER NOT NULL
);
CREATE INDEX idx_tr_project ON translatable_rows(project_id);

CREATE TABLE translations (
  row_id TEXT NOT NULL REFERENCES translatable_rows(id) ON DELETE CASCADE,
  target_locale TEXT NOT NULL,
  text TEXT NOT NULL,
  status TEXT NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (row_id, target_locale)
);
```

**Scan strategy:**

```rust
pub async fn scan_translatable(handle: &ProjectHandle, ctx: &HandlerCtx) -> Result<()> {
    // Walk AssetTree for nodes with kind == Text or named StringTable*.asset
    // For each, run parse_textasset or parse_stringtable
    // Bulk-insert into translatable_rows (transaction)
}
```

**Frontend additions:**

```
src/features/project/translate/
├── translate-view.tsx         # main container; replaces center pane when rail = 'strings'
├── translate-toolbar.tsx
├── locale-picker.tsx
├── progress-strip.tsx
├── status-dot.tsx
├── translate-row.tsx
├── translate-list.tsx         # virtualized list of rows
└── use-translate-store.ts     # local Zustand: targetLocale, filter, dirty buffer
```

**`useTranslateStore`:**

```ts
type TranslateState = {
  targetLocale: string;        // 'vi'
  filter: 'all'|'translated'|'pending'|'review';
  dirty: Record<string, string>;   // rowId -> draft text
  setLocale: (l: string) => void;
  setFilter: (f) => void;
  updateDraft: (rowId, text) => void;
  flush: () => Promise<void>;
};
```

**Autosave debounce flow:**

```ts
const debouncedSave = useDebouncedCallback(async (rowId, text) => {
  await ipc.saveTranslation({ rowId, targetLocale, text, status: text.length ? 'Translated' : 'Pending' });
}, 500);

const onChange = (rowId, text) => {
  updateDraft(rowId, text);
  debouncedSave(rowId, text);
};
```

**Render row matching wireframe 04:**

```tsx
<div class="flex border-b border-default min-h-[76px]">
  <KeyCell row={row} status={status}/>
  <SourceCell text={row.source_text}/>
  <TargetCell text={draft} onChange={onChange}/>
  <RowActions row={row}/>
</div>
```

## Files to Modify / Create
- CREATE `src-tauri/src/commands/translate.rs`
- CREATE `src-tauri/src/handlers/unity/strings/mod.rs`
- CREATE `src-tauri/src/handlers/unity/strings/scan.rs`
- CREATE `src-tauri/src/handlers/unity/strings/parse_textasset.rs`
- CREATE `src-tauri/src/handlers/unity/strings/parse_stringtable.rs`
- CREATE `src-tauri/src/handlers/unity/strings/locale_detect.rs`
- MODIFY `src-tauri/src/db/schema.rs` — add v2 migration with translatable_rows + translations.
- CREATE `src-tauri/src/db/repo/translations.rs`
- MODIFY `src-tauri/src/lib.rs` — register translate commands.
- MODIFY `src-tauri/Cargo.toml` — add `csv = "1"`, `yaml-rust2 = "0.10"` (or `serde_yaml`).
- CREATE `src/features/project/translate/translate-view.tsx`
- CREATE `src/features/project/translate/translate-toolbar.tsx`
- CREATE `src/features/project/translate/locale-picker.tsx`
- CREATE `src/features/project/translate/progress-strip.tsx`
- CREATE `src/features/project/translate/translate-row.tsx`
- CREATE `src/features/project/translate/translate-list.tsx`
- CREATE `src/stores/use-translate-store.ts`
- CREATE `src/lib/use-debounced-callback.ts` (small util; or `pnpm add use-debounce`)
- MODIFY `src/features/project/project-route.tsx` — route center pane content by rail tab (`'files'` → tabs+preview, `'strings'` → translate view).
- MODIFY `src/lib/ipc.ts` — add `listTranslatable`, `saveTranslation`, `listLocales`, `exportTranslations`.

## Implementation Steps
1. Add `csv` + YAML deps to Cargo. Add `pnpm add use-debounce` (or write tiny hook).
2. Implement DB migration v2 in `db/schema.rs` — add new tables; gate on `PRAGMA user_version`.
3. Implement `db/repo/translations.rs` — CRUD for `translatable_rows` + `translations`; bulk insert helper.
4. Implement `locale_detect::extract_locale(path)`:
   - Regex `/Locales?/([a-z]{2,3}(?:-[A-Z]{2})?)/`.
   - Filename suffix `_([a-z]{2,3})\.(json|csv|txt)$`.
   - Fallback to `en`.
5. Implement `parse_textasset.rs`:
   - JSON: read top-level object `{ "menu.title": "Start Game", ... }` → flat key/value rows.
   - Nested JSON: flatten with dot-joined keys.
   - CSV: first column = key, second = text (skip header).
   - .txt with key=value lines.
6. Implement `parse_stringtable.rs`:
   - Reads YAML emitted by AssetRipper for Unity Localization package's StringTable.asset.
   - Extracts `m_TableEntries` array → rows.
7. Implement `scan::scan_translatable(handle, ctx)`:
   - Walk handle's `AssetTree.nodes`.
   - For each node where `kind == Text` AND filename suggests locale, dispatch to parser.
   - Bulk insert rows in a transaction.
   - Idempotent: ON CONFLICT(id) skip.
8. Add Tauri commands:
   - `list_translatable(handle_id, source_locale) -> Vec<TranslatableRow>` (filters by source_locale).
   - `save_translation({ rowId, targetLocale, text, status })`.
   - `list_locales(handle_id) -> Vec<{ code, count }>`.
   - `export_translations({ handle_id, target_locale, format: 'csv'|'json', dest_path })`.
9. Call `scan_translatable` lazily on first IPC `list_translatable` call (cache flag in `projects` table column `translatable_scanned INTEGER`).
10. Frontend: implement `<TranslateView>` as root of strings tab.
11. Implement `<TranslateToolbar>` matching wireframe 04 lines 469-520. Use real locale list from `listLocales`. Progress strip computed from `translated/total`.
12. Implement `<LocalePicker>` — popover with searchable locale list (ISO-639 + project-detected).
13. Implement `<TranslateList>` virtualized via TanStack Virtual.
14. Implement `<TranslateRow>`:
    - Status dot color by status.
    - Key cell + comment.
    - Source EN cell read-only.
    - Target cell `<textarea>` with `onChange` debounced save.
    - Char count display.
    - Actions: mark reviewed (toggles status), copy source, jump-to-source (calls `useTabStore.openTab` on source file).
15. Wire optimistic updates: `useTranslateStore.dirty` overrides displayed value; on flush, query cache updated.
16. Implement export flow:
    - On Export click, open Tauri dialog → user picks path + format.
    - IPC `export_translations` writes file in Rust (avoids serializing thousands of rows to JS).
17. Wire `<ProjectRoute>` to switch center pane between `<TabContent>` (files view) and `<TranslateView>` (strings view) based on `useUiStore.activeRail`.
18. Auto-translate button → cosmetic-only — opens modal "AI translation — coming in v0.2"; do NOT call any external API in v1.
19. Verify: `cargo clippy -- -D warnings` passes.
20. Verify: `pnpm tsc --noEmit` passes.
21. Verify: fixture project with at least one Localization JSON parses → rows appear in translate view.
22. Verify: typing into target cell saves to DB (close + reopen app → translations persist).
23. Verify: export CSV produces valid UTF-8 file readable by Excel + LibreOffice.
24. Verify: 5k rows scroll smoothly.
25. Pixel-match wireframe 04 visual review.

## Todo List
- [x] DB v2 migration with translatable_rows + translations
- [x] `db/repo/translations.rs` CRUD + bulk insert
- [x] `locale_detect` path/filename matching
- [x] `parse_textasset` for JSON / nested JSON / CSV / .txt
- [x] `parse_stringtable` for Unity Localization YAML
- [x] `scan_translatable` walks tree + bulk inserts
- [x] IPC: list_translatable / save_translation / list_locales / export_translations
- [x] Lazy scan flag on `projects` table
- [x] `<TranslateView>` container
- [x] `<TranslateToolbar>` with locale + progress + filter + export + auto-translate
- [x] `<LocalePicker>` popover with ISO-639 + detected locales
- [x] `<TranslateList>` virtualized
- [x] `<TranslateRow>` with status dot, source, editable target, char count, actions
- [x] Debounced autosave (500ms) with optimistic UI
- [x] `useTranslateStore` Zustand slice
- [x] Export CSV / JSON via Tauri dialog + Rust writer
- [x] `<ProjectRoute>` switch center pane by active rail
- [x] Auto-translate "coming v0.2" modal
- [x] `cargo clippy -- -D warnings` passes
- [x] `pnpm tsc --noEmit` passes
- [x] Fixture round-trip: scan → edit → reopen → persisted
- [x] Pixel-match wireframe 04

## Success Criteria
- Wireframe 04 rendered side-by-side — match within 2% for translate view.
- Project with a Localization JSON populates translate rows on first rail-tab open.
- Editing a target cell persists across app restart.
- Progress strip + count reflect live edits.
- Export CSV opens cleanly in Excel; export JSON is valid `{ "key": "translation", ... }`.
- 5k rows scroll at 60fps.
- Filter narrows rows correctly.
- `cargo clippy` + `pnpm tsc` clean.

## Risk Assessment
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| AssetRipper YAML for StringTable varies by Unity Localization version | High | Medium | Defensive parsing; skip unparseable rows; log warning |
| User-entered translations lost on db corruption | Low | High | DB WAL mode (already in v1); document periodic backup in BUILD.md |
| Auto-translate button raises expectation of working AI integration | High | Low | Explicit "Coming v0.2" modal; gray-out is sufficient signal |
| Encoding issues on non-UTF-8 TextAssets | Medium | Medium | Use `encoding_rs` to detect+convert; default to UTF-8 with replacement chars |
| 5k+ row virtualization with textarea inside causes scroll jank | Medium | Medium | Use plain `<input>` instead of textarea for v1; expand on click; measured against fixture |
| Export of very large datasets blocks Rust thread | Low | Low | Stream writer; spawn_blocking; max 100k rows expected |
| Locale codes ambiguous (`zh` vs `zh-CN` vs `zh-TW`) | Medium | Low | Show full code from detection; let user choose via picker |

## Security Considerations
- DB queries parameterized (rusqlite `?` placeholders); no SQL injection.
- Export path validated as writable + outside cache dir if user chooses.
- Auto-translate intentionally NOT wired to any external API in v1 — no PII leak risk.
- TextAsset reads bounded to 16MB per file; reject larger.
- YAML parser configured for safe-load only; no custom tags / object deserialization.

## Implementation Notes
DB v2 migration with translatable_rows + translations tables. 4 new strings parsers: textasset (JSON/CSV/txt), stringtable (YAML), locale_detect, scan. 4 new Tauri commands: list_translatable, save_translation, list_locales, export_translations. Frontend: TranslateView + Toolbar + LocalePicker + List + Row with debounced autosave (500ms). useTranslateStore Zustand slice. Virtualized translation list handling 5k+ rows. Status dots for translated/pending/review. Export to CSV/JSON. 76 Rust unit tests passing. Locale detection via path patterns + filename suffixes.

## Next Steps
- Unblocks phase-09 (palette can search translatable keys via FTS5).
- v2 will add repack-to-game flow (write translated strings back into asset bundle via AssetRipper export).
