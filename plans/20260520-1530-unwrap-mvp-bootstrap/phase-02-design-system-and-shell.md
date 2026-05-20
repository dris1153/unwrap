# Phase 02: Design System & Shell

**Status:** Complete (2026-05-20)
**Priority:** Critical
**Effort:** M (2-3d)
**Depends on:** phase-01-project-scaffolding

## Context Links
- Design: `docs/design-guidelines.md` (Color System, Typography, Spacing, Layout — App Shell, Components, Motion)
- Wireframe: `docs/wireframe/01-welcome.html` (Welcome screen ground truth)
- Wireframe: `docs/wireframe/02-asset-preview.html` (App shell ground truth — TitleBar/Toolbar/Sidebar/Inspector/StatusBar)
- Brand: `docs/branding/brand-marks.md`

## Overview
Translate the design tokens from `design-guidelines.md` into a Tailwind 4 config + shadcn primitives + a complete app shell skeleton. After this phase the Welcome route renders pixel-matched to `01-welcome.html` and the app shell (TitleBar / Toolbar / Sidebar / Inspector / StatusBar) renders empty but structurally complete to `02-asset-preview.html`.

## Key Insights
- Design uses tokens (`bg-base`, `bg-surface`, `bg-elevated`, etc.) — encode via Tailwind 4 `@theme` block, not custom CSS vars.
- Title bar is custom (`decorations: false`) — Windows control buttons rendered in React, drag region via `data-tauri-drag-region`.
- shadcn-style primitives kept minimal: only Button, Input, Tooltip, Dialog, ScrollArea, Tabs, Separator. Other shapes are app-specific components (no shadcn copy-paste).
- TanStack Router file-based routing: `/` -> WelcomeRoute, `/project/:id` -> ProjectRoute (built in phase-06).
- Zustand `useUiStore` holds sidebar width, inspector collapsed state, active rail tab.

## Requirements

### Functional
1. Tailwind config exposes every token from `design-guidelines.md` §2 (colors), §3 (typography), §4 (spacing), §6 (radius).
2. Welcome route (`/`) renders pixel-matched to `01-welcome.html`: hero glow, drop zone (visually only — no real drop handler yet), 2x2 recents grid (static mock data), status bar.
3. App shell components exist and render with no data:
   - `<TitleBar>` 36px, custom Windows controls (min/max/close), brand mark + project breadcrumb dropdown.
   - `<Toolbar>` 44px, nav arrows + Recents button + breadcrumb + ⌘K search input (no behavior) + bell/settings.
   - `<Sidebar>` 280px = 44px rail + 236px tree panel, header "Assets" + filter input + empty tree placeholder.
   - `<Inspector>` 320px, collapsible to 0 via `useUiStore`, "Inspector" header + placeholder.
   - `<StatusBar>` 24px, backend dot + version dots + offline mode.
4. Custom title bar drag region works (window draggable from title bar area, not from buttons).
5. Window controls (min/max/close) call Tauri `app.window.{minimize|toggleMaximize|close}` via `@tauri-apps/api/window`.
6. Sidebar + Inspector resizable via drag handles, persisted in Zustand store.
7. Drop zone on welcome screen accepts folder drag-over visually (pulse border via CSS keyframes) but defers actual file handling to phase-05.

### Non-Functional
- All rendered sizes within 2px of wireframes.
- No CLS (cumulative layout shift) on initial load.
- `prefers-reduced-motion: reduce` disables shimmer + pulse animations.
- Keyboard focus visible via 2px accent-glow ring on every interactive element.

## Architecture / Approach

**Tailwind 4 `@theme` block in `globals.css`:**

```css
@import "tailwindcss";

@theme {
  --color-base: #09090B;
  --color-surface: #111114;
  --color-elevated: #18181B;
  --color-overlay: #27272A;
  --color-border-subtle: rgba(63,63,70,0.35);
  --color-border-default: #27272A;
  --color-border-strong: #3F3F46;
  --color-primary: #F4F4F5;
  --color-secondary: #A1A1AA;
  --color-tertiary: #71717A;
  --color-disabled: #52525B;
  --color-accent: #10B981;
  --color-accent-strong: #059669;
  --color-accent-muted: rgba(16,185,129,0.12);
  --color-accent-glow: rgba(16,185,129,0.20);
  --color-success: #22C55E;
  --color-warning: #F59E0B;
  --color-danger: #EF4444;
  --color-info: #0EA5E9;
  --color-tex: #F472B6;
  --color-aud: #A78BFA;
  --color-mesh: #FBBF24;
  --color-script: #10B981;
  --color-txt: #60A5FA;
  --color-bin: #71717A;
  --font-sans: "Satoshi", ui-sans-serif, system-ui;
  --font-mono: "JetBrains Mono", ui-monospace, monospace;
  --tracking-tightest: -0.02em;
  --tracking-tighter: -0.015em;
  --tracking-tightish: -0.01em;
  --radius-sm: 4px;
  --radius-md: 6px;
  --radius-lg: 8px;
  --radius-xl: 12px;
  --radius-2xl: 16px;
}
```

**Zustand `useUiStore` interface:**

```ts
type UiState = {
  sidebarWidth: number;        // 200..400, default 280
  inspectorWidth: number;      // 0 (collapsed) or 240..480, default 320
  activeRail: 'files'|'search'|'xref'|'scripts'|'strings'|'bookmarks';
  setSidebarWidth: (w: number) => void;
  toggleInspector: () => void;
  setActiveRail: (r: UiState['activeRail']) => void;
};
```

**Component tree (Welcome route):**

```
<AppShell>
  <TitleBar mode="welcome"/>
  <main class="hero-glow flex-1">
    <WelcomeHero/>
    <RecentProjectsGrid items={mockData}/>
  </main>
  <StatusBar mode="welcome"/>
</AppShell>
```

**Component tree (Project route, populated in phase-06):**

```
<AppShell>
  <TitleBar mode="project" projectName="..."/>
  <Toolbar/>
  <div class="flex flex-1 min-h-0">
    <Sidebar/>
    <CenterPane> <!-- empty in phase-02, tabs+preview added later --> </CenterPane>
    <Inspector/>
  </div>
  <StatusBar mode="project"/>
</AppShell>
```

**Routing (TanStack Router file-based):**

```
src/app/
├── __root.tsx          # AppShell + Outlet
├── index.tsx           # WelcomeRoute (renders WelcomeHero)
└── project/
    └── $projectId.tsx  # ProjectRoute (empty layout, scaffolded for phase-06)
```

## Files to Modify / Create
- MODIFY `src/styles/globals.css` — replace placeholder with full `@theme` block, font-face, keyframes (drop-pulse, shimmer, status-pulse).
- DELETE `tailwind.config.ts` — Tailwind 4 uses CSS-based config; no JS file needed.
- CREATE `src/components/app-shell/title-bar.tsx`
- CREATE `src/components/app-shell/toolbar.tsx`
- CREATE `src/components/app-shell/sidebar.tsx`
- CREATE `src/components/app-shell/inspector.tsx`
- CREATE `src/components/app-shell/status-bar.tsx`
- CREATE `src/components/app-shell/app-shell.tsx`
- CREATE `src/components/app-shell/window-controls.tsx` (min/max/close buttons)
- CREATE `src/components/app-shell/resize-handle.tsx`
- CREATE `src/components/ui/button.tsx` (shadcn-derived, primary/secondary/ghost/danger variants)
- CREATE `src/components/ui/input.tsx`
- CREATE `src/components/ui/tooltip.tsx`
- CREATE `src/components/ui/scroll-area.tsx`
- CREATE `src/components/ui/separator.tsx`
- CREATE `src/components/ui/kbd.tsx` (keyboard hint pill `⌘K` style from wireframe)
- CREATE `src/components/ui/status-dot.tsx`
- CREATE `src/features/welcome/welcome-hero.tsx`
- CREATE `src/features/welcome/recent-projects-grid.tsx`
- CREATE `src/features/welcome/drop-zone.tsx` (visual only, real handler in phase-05)
- CREATE `src/stores/use-ui-store.ts`
- CREATE `src/app/__root.tsx`
- CREATE `src/app/index.tsx` (WelcomeRoute)
- CREATE `src/app/project/$projectId.tsx` (empty ProjectRoute scaffold)
- CREATE `src/app/router.tsx` (router instance)
- MODIFY `src/main.tsx` — wrap in `<RouterProvider/>`
- MODIFY `src/App.tsx` — DELETE (router replaces it) OR strip to nothing
- MODIFY `src-tauri/tauri.conf.json` — `decorations: false` (custom titlebar)
- CREATE `src/lib/cn.ts` (classname merge util)
- CREATE `src/lib/mock-recents.ts` (4 entries matching wireframe — Hollow Vale / Ferrocity Demo / KOTD / Wandersmith)

## Implementation Steps
1. Install deps: `pnpm add @tanstack/react-router zustand class-variance-authority clsx tailwind-merge @radix-ui/react-tooltip @radix-ui/react-scroll-area @radix-ui/react-separator`.
2. Install dev deps for router codegen: `pnpm add -D @tanstack/router-vite-plugin @tanstack/router-cli`.
3. Wire router-vite-plugin in `vite.config.ts`.
4. Rewrite `globals.css` with `@theme` block above + `@font-face` + keyframes (`@keyframes dropPulse`, `@keyframes shimmer`, `@keyframes statusPulse`).
5. Create `src/lib/cn.ts` (`clsx + tailwind-merge`).
6. Create shadcn-style primitives in `src/components/ui/` — copy from wireframe `01-welcome.html` button + input markup; abstract into CVA-based components.
7. Create `useUiStore` in `src/stores/use-ui-store.ts`.
8. Build `<WindowControls>` calling `getCurrentWindow().minimize()`, `.toggleMaximize()`, `.close()` from `@tauri-apps/api/window`.
9. Build `<TitleBar>` with `data-tauri-drag-region` on outer container; brand mark + breadcrumb + window controls. Two modes: `welcome` (just brand) and `project` (brand + project dropdown).
10. Build `<Toolbar>` matching wireframe 02 lines 134-191: nav arrows + Recents button + breadcrumb + ⌘K search (no behavior) + bell/settings.
11. Build `<Sidebar>` matching wireframe 02 lines 197-454: 44px rail (6 buttons + bookmarks) + 236px tree panel (header + filter input + empty body + footer mini-stats).
12. Build `<Inspector>` matching wireframe 02 lines 733-911: 36px header + scrollable body with placeholder copy "Select an asset to inspect."
13. Build `<StatusBar>` matching wireframes — two variants by `mode` prop.
14. Build `<ResizeHandle>` (1px hot strip, 8px hit area, cursor `col-resize`) — emits delta to `useUiStore`.
15. Build `<WelcomeHero>` (left 60%) matching wireframe 01 lines 132-209: eyebrow + display headline + sub body + drop zone + supports line.
16. Build `<DropZone>` — visual only, uses `drop-pulse` animation. Accepts file/folder drag events but only `console.log`s — real handler in phase-05.
17. Build `<RecentProjectsGrid>` (right 40%) matching wireframe 01 lines 212-388: header + 2x2 grid + empty row + tip footer. Use `mock-recents.ts` for data.
18. Wire routes via TanStack Router; `__root.tsx` renders `<AppShell>` + `<Outlet>`.
19. Set `tauri.conf.json` `app.windows[0].decorations = false`.
20. Verify: `pnpm tsc --noEmit` passes.
21. Verify: `pnpm tauri dev` opens window; Welcome route matches `01-welcome.html` within 2px on all axis. Compare side-by-side.
22. Verify: navigate to `/project/test-id` manually — empty ProjectRoute renders TitleBar/Toolbar/Sidebar/Inspector/StatusBar shell.
23. Verify: Inspector toggle in `useUiStore` collapses inspector to 0 width.
24. Verify: sidebar resize via drag handle, persisted in store.

## Todo List
- [x] Install router + zustand + radix + cva deps
- [x] Write Tailwind 4 `@theme` block with all design tokens
- [x] Define keyframes (dropPulse, shimmer, statusPulse)
- [x] Build shadcn primitives (Button, Input, Tooltip, ScrollArea, Separator, Kbd, StatusDot)
- [x] Build `useUiStore`
- [x] Build `<WindowControls>` with Tauri window API
- [x] Build `<TitleBar>` (welcome + project modes)
- [x] Build `<Toolbar>`
- [x] Build `<Sidebar>` (rail + tree panel skeleton)
- [x] Build `<Inspector>` (header + placeholder)
- [x] Build `<StatusBar>` (welcome + project modes)
- [x] Build `<ResizeHandle>`
- [x] Build `<AppShell>` composing the above
- [x] Build `<WelcomeHero>` + `<DropZone>` + `<RecentProjectsGrid>`
- [x] Wire TanStack Router routes
- [x] Disable Tauri default decorations
- [x] `pnpm tsc --noEmit` passes
- [x] Welcome route pixel-matches `01-welcome.html` within 2px
- [x] Project route renders empty shell matching `02-asset-preview.html` chrome

## Implementation Notes
- Cooked: 2026-05-20 (single dev, --auto bootstrap)
- Verification: `pnpm tsc --noEmit` pass, `pnpm build` pass, visual review against wireframes complete
- Deviations: 1 icon substitution — FolderNotchOpen → FolderSimplePlus due to package version mismatch; tailwind.config.ts not deleted (harmless under Tailwind 4)
- Files shipped: 7 UI primitives (button, input, tooltip, scroll-area, separator, kbd, status-dot) + 8 app-shell components (title-bar, toolbar, sidebar, inspector, status-bar, window-controls, resize-handle, app-shell) + welcome features (welcome-hero, drop-zone, recent-projects-grid) + TanStack Router file-based routing + Zustand useUiStore with persist middleware

## Success Criteria
- Welcome page screenshot diff vs `01-welcome.html` rendered at 1440x900: <2% pixel diff (manual review).
- Project shell layout matches `02-asset-preview.html` chrome (excluding tree/preview/inspector content).
- All 7 rail buttons render with correct Phosphor icons and hover states.
- Window controls (min/max/close) work via Tauri API.
- Window draggable from title bar background.
- Sidebar resizes via drag handle, width persisted in Zustand.
- Inspector collapses via Zustand toggle.
- `prefers-reduced-motion: reduce` disables drop-pulse animation.
- `pnpm tsc --noEmit` passes, no any-types in component props.

## Risk Assessment
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Custom titlebar drag region breaks on Windows snap | Medium | Medium | Test snap-to-edge gestures; document if broken; fallback `decorations: true` flag |
| Tailwind 4 `@theme` directive instability | Medium | High | Lock to specific minor version; keep CSS-vars fallback ready |
| TanStack Router file-routing codegen flakiness | Low | Medium | Commit `routeTree.gen.ts` to repo; regenerate on build |
| Phosphor icon weights don't match wireframe (regular vs fill) | Medium | Low | Use `weight="regular"` default; `fill` only where wireframe explicitly shows fill |
| Welcome hero glow gradient renders differently across WebView2 versions | Low | Low | Test on Win11 + Win10 minimum WebView2; document anomalies |

## Security Considerations
- `data-tauri-drag-region` zones must not overlap clickable elements (security: clickjacking surface).
- Window control buttons must NOT have `data-tauri-drag-region` — would freeze close button.
- CSP unchanged from phase-01.
- No new Tauri commands; IPC surface still empty.

## Next Steps
- Unblocks phase-03 (Rust backend can now invoke commands wired to UI placeholders).
- Phase-06 will replace placeholder Sidebar tree content with virtualized real tree.
- Phase-09 will wire ⌘K shortcut → palette modal (no behavior yet here).
