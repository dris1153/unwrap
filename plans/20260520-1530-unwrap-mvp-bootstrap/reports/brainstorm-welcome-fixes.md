# Brainstorm — Welcome Screen Fixes

**Date:** 2026-05-20
**Trigger:** User ran `pnpm tauri dev` against shipped v0.1.0. 3 functional gaps surfaced.
**Mode:** Brainstorm only; no implementation yet.

---

## Problem Statement

`pnpm tauri dev` opens Welcome screen với visual đúng wireframe nhưng 3 interaction gaps:

1. Window controls (min/max/close) ở title bar không có phản ứng khi click
2. "Browse files…" + "Open archive (.zip)" buttons không có phản ứng
3. Recent projects (4 tiles bên phải) là mockup data — không reflect DB thực

App được build qua /ck:bootstrap + /ck:cook 10 phases, tất cả tests pass, nhưng những "wire-up cuối cùng" ở welcome screen bị miss.

---

## Root Cause Analysis

### Issue 1: Window controls broken
- File: `src-tauri/capabilities/default.json:7-8`
- Capability chỉ có `core:default` + `opener:default`
- Tauri 2 model: `core:default` granted `core:window:default` nhưng default này **chỉ** include event listeners (`allow-emit`, `allow-listen`, …) — KHÔNG include window control verbs
- Frontend `<WindowControls>` ở `src/components/app-shell/window-controls.tsx:10-32` gọi `getCurrentWindow().minimize/toggleMaximize/close` đúng API nhưng IPC bị denied silently (chỉ thấy lỗi console)
- Bonus: title bar `data-tauri-drag-region` cần `core:window:allow-start-dragging` để drag được

### Issue 2: Browse + Open archive handlers no-op
- File: `src/features/welcome/drop-zone.tsx:167,175`
- Cả 2 handlers literal `console.log(...)` từ phase-02 scaffold
- Phase-05 đã wire native `onDragDropEvent` (drag-drop folder work) NHƯNG click button thì chưa
- Thiếu hoàn toàn: `@tauri-apps/plugin-dialog` (npm) + `tauri-plugin-dialog` (Rust crate) + plugin init ở `lib.rs` + capability `dialog:allow-open`

### Issue 3: Recent projects mockup
- File: `src/features/welcome/recent-projects-grid.tsx:12,149-163`
- Component import `MOCK_RECENTS` directly từ `lib/mock-recents.ts`
- Phase-03 đã build `ipc.listRecents()` thật (SQLite `recents` JOIN `projects`) nhưng frontend chưa swap
- Backend `RecentEntry` shape có thể chưa đủ field cho UI (`openedLabel`, `iconTheme`, `chip` variant) — cần verify khi plan

---

## Decisions (user-approved)

| Question | Choice |
|---|---|
| Fix strategy | **C: Bundle 1+2 (Tauri permissions theme), 3 separate** |
| .zip support | **Defer to v0.2 — button shows "Coming soon" tooltip** |
| Empty state | **"No recent projects yet — drop a folder to start"** |

Rationale:
- 1+2 grouped vì cả 2 đụng vào `capabilities/default.json` + Tauri permission layer; tách ra phải edit cùng file 2 lần
- Issue 3 pure frontend swap (mock → IPC + empty state), không động Rust
- .zip wire thật cần unzip pipeline + temp dir mgmt — không scope cho hotfix, defer

---

## Solution Design

### PR 1 — Tauri Permissions + Dialog Plugin (Issues 1 & 2)

**Backend changes:**
- `src-tauri/Cargo.toml` — add `tauri-plugin-dialog = "2"`
- `src-tauri/src/lib.rs` — `.plugin(tauri_plugin_dialog::init())` trong Tauri builder
- `src-tauri/capabilities/default.json` — append 5 permissions:
  - `core:window:allow-minimize`
  - `core:window:allow-toggle-maximize`
  - `core:window:allow-close`
  - `core:window:allow-start-dragging`
  - `dialog:allow-open`

**Frontend changes:**
- `package.json` — add `@tauri-apps/plugin-dialog`
- `src/features/welcome/drop-zone.tsx`:
  - Replace `console.log` browse handler với:
    ```ts
    import { open } from "@tauri-apps/plugin-dialog";
    const selected = await open({ directory: true, multiple: false, title: "Select Unity build folder" });
    if (typeof selected === "string") void handleOpen(selected);
    ```
  - Replace `console.log` archive handler với Tooltip wrapper:
    - Button disabled visual state
    - On hover: tooltip "Coming in v0.2 — archive extraction"
    - Alternative: button vẫn clickable nhưng mở modal "Coming soon" (consistent với phase-08 auto-translate pattern)
  - Add `⌘O` keyboard shortcut → trigger same flow (already hinted ở UI)

**Verification:**
- `cargo check` + `cargo clippy` clean
- `pnpm tsc` + `pnpm build` clean
- `pnpm tauri dev` manual:
  - Click min/max/close → all work
  - Drag title bar → window draggable
  - Browse button → native folder picker → select → opens project route
  - Open archive → tooltip/modal "Coming v0.2"

### PR 2 — Real Recents via IPC + Empty State (Issue 3)

**Backend check (may need extension):**
- Verify `RecentEntry` shape returned by `list_recents()` has fields: `id`, `name` (derived from `root_path` basename), `root_path`, `engine_version`, `scripting_backend`, `last_opened`, `pinned`, `asset_count` (from cached `asset_trees` row count if available)
- Nếu thiếu, extend `commands/project.rs::list_recents` SQL JOIN

**Frontend changes:**
- `src/features/welcome/recent-projects-grid.tsx`:
  - Remove `MOCK_RECENTS` import
  - Use `useQuery(["recents"], () => ipc.listRecents())`
  - Map `RecentEntry` → UI shape (`RecentProject`):
    - `iconTheme`: derive from `scripting_backend` (Mono → `script`, IL2CPP → `mesh`, Unknown → `tex`)
    - `chip`: pinned → `pinned`, else `scripting_backend` lowercased
    - `openedLabel`: `Intl.RelativeTimeFormat` từ `last_opened` timestamp
  - Loading state: 4 skeleton tiles (shimmer animation đã có ở globals.css)
  - **Empty state** (DB rỗng):
    - 2×2 grid biến mất
    - Single card empty state: dashed border, icon (`ph-clock-counter-clockwise`), copy "No recent projects yet", subtitle "Drop a Unity build folder on the left to begin", "Clear history" button bị ẩn
  - Refresh button → `queryClient.invalidateQueries(["recents"])`
  - Click tile → call `ipc.openProject(root_path)` + navigate to `/project/$projectId`
- `src/lib/mock-recents.ts`:
  - Keep `RecentProject` type export
  - Remove `MOCK_RECENTS` constant (hoặc rename → `__MOCK_RECENTS_DEV_ONLY` cho potential storybook reuse)

**Verification:**
- pnpm tsc + build clean
- `pnpm tauri dev` manual:
  - Empty DB → empty state hiển thị
  - Drop folder → mới reopen → tile xuất hiện ở recents grid
  - Click recent tile → opens project route
  - Refresh button updates list

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `tauri-plugin-dialog` v2 API khác với plugin v1 docs | Low | Medium | Cross-check `docs.rs/tauri-plugin-dialog` trước implement |
| `RecentEntry` shape không đủ field cho UI | Medium | Medium | PR 2 step 1 = verify shape; nếu thiếu extend backend trước frontend swap |
| Tooltip primitive chưa support disabled-button hover (Radix quirk) | Medium | Low | Fallback: wrap button trong span + tooltip on span |
| Empty state copy nghe khô | Low | Low | Tweak với designer review trong v0.1.1 |
| `core:window:allow-start-dragging` không tồn tại — Tauri 2 đã rename | Low | Medium | Verify capability schema trước commit |

---

## Success Metrics

- Window control clicks → 100% success rate (no console errors)
- Title bar drag → window moves
- Browse button → native dialog → select folder → project opens
- Open archive → tooltip/modal explains v0.2
- Recents grid: empty DB → empty state; populated DB → real entries với accurate timestamps
- All existing 141 tests still pass
- No new clippy warnings, no new tsc errors
- Bundle size delta < 50KB (dialog plugin is tiny)

---

## Out of Scope (defer)

- .zip archive extraction pipeline → v0.2
- Multi-project recents (>10) pagination → v0.2
- Pin/unpin recents from UI → v0.2 (DB column exists, no UI action yet)
- Drag-to-reorder recents → v0.2

---

## Next Steps

User to confirm proceed sang `/ck:plan` để break PR 1 và PR 2 thành phase files với task lists detailed.

Alternatively: skip plan + cook ngay vì scope nhỏ (mỗi PR ~5-10 file changes).
