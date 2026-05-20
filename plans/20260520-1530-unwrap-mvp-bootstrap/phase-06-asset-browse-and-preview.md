# Phase 06: Asset Browse & Preview

**Status:** Complete (2026-05-20)
**Priority:** Critical
**Effort:** L (3-4d)
**Depends on:** phase-05-unity-format-handler

## Context Links
- Wireframe: `docs/wireframe/02-asset-preview.html` (ground truth for tree, tabs, image preview, inspector, status bar)
- Design: `docs/design-guidelines.md` (Tree, Tabs, Inspector components; file-type accent palette)
- Arch: `docs/system-architecture.md` (Workflow #2 Preview Texture)

## Overview
Replace the empty Sidebar tree from phase-02 with a virtualized asset tree (TanStack Virtual) fed by the `tree` command. Build the tabbed editor area + preview routers for Image (zoom-pan-pinch + checker bg), Audio (wavesurfer.js), 3D (@react-three/fiber), Text, and Hex. Populate Inspector with real metadata + references. Pixel-match `02-asset-preview.html`. Parallel-safe with phases 07 and 08.

## Key Insights
- Tree of 50k nodes — must be virtualized. TanStack Virtual `useVirtualizer` over a flattened-visible-nodes array; recompute on expand/collapse via memo.
- Tree expansion state held in Zustand store (`useTreeStore.expandedIds: Set<NodeId>`) — keyed per project so route changes restore state via cache.
- TanStack Query caches `tree(projectId)` and `preview(handle, nodeId)` results; preview cache time = 5min, stale time = 1min.
- Tab state is local Zustand store (`useTabStore`): `tabs: Tab[]`, `activeTabId`, `openTab(nodeId)`, `closeTab(tabId)`. Tabs survive within session; not persisted to disk.
- Preview routing component picks renderer by `PreviewPayload` variant — pattern-match the discriminator.
- `convertFileSrc()` from `@tauri-apps/api/core` is required to load files from disk into `<img>`, `<audio>`, three.js — never use file:// directly.

## Requirements

### Functional
1. After project open, Sidebar tree renders the project's `AssetTree`:
   - Virtualized rendering (visible rows + 10 overscan).
   - Hierarchical with chevron-disclosure, file-type tinted icons per `design-guidelines.md` §2.
   - Asset count badge per folder (recursive count).
   - Filter input filters visible tree by substring match (case-insensitive).
   - Filter result count shown in tree footer.
2. Clicking a node opens it in a tab in the center pane.
3. Tab bar matches wireframe 02 lines 460-506:
   - Active tab underline indicator (Framer `layoutId` animated).
   - File-type icon + mono path + close button.
   - Modified-dot for unsaved edits (always off in v1 — read-only — but render the dot slot).
   - `+ new tab` button (no-op or open `welcome` placeholder).
   - Split-right + more buttons (cosmetic for v1).
4. Center pane renders preview by node kind. Below tab bar there is a content-specific sub-toolbar (e.g., image preview shows dimensions + Channels + Histogram + Export PNG matching wireframe).
5. Image preview:
   - `<img src={convertFileSrc(path)}/>` wrapped in `react-zoom-pan-pinch`.
   - Checker bg behind transparent images.
   - Zoom controls bottom-right (− pct + fit checker buttons).
   - Coordinates HUD top-right (x/y/hex pixel — best-effort via canvas readback; fallback to mouse coords only).
   - File label top-left.
   - Filmstrip below preview for sibling textures in same folder (max 8 + "+ N more").
6. Audio preview:
   - `wavesurfer.js` waveform.
   - Play/pause + timeline + duration display.
   - Loop / speed controls cosmetic-only v1.
7. 3D preview:
   - `@react-three/fiber` + `@react-three/drei` (`OrbitControls`, `Grid`, `Environment`).
   - GLB/GLTF via `useGLTF`; FBX via `FBXLoader` from `three`.
   - OBJ via `OBJLoader`.
   - HDRI environment optional (keep tiny built-in `<Environment preset="city">`).
8. Text preview: Monaco Editor (`@monaco-editor/react`) read-only; language auto-detected from extension. (Decompiled C# in phase-07 uses same editor.)
9. Hex preview: `react-hexkit` or custom; 16 bytes/row; ASCII column; offset gutter; supports 2GB file via virtualization (read bytes in chunks via Tauri command `read_file_chunk(path, offset, len)` — add in this phase).
10. Inspector renders `PreviewPayload` metadata per wireframe 02 lines 733-911:
    - File identity card (icon, name, path, type chips).
    - Properties section (Dimensions, Format, Mipmaps, Compressed, Filter, Wrap, Size on disk, Pixel hash).
    - Origin section (Source bundle, PathID, Asset GUID).
    - References section (`Vec<CodeRef>` from preview payload; cross-references implemented best-effort — full xref index deferred to v2).
    - In-animations section (mock-only v1 — actual anim parsing deferred).
    - Actions: Export PNG, Copy GUID, Reveal references.
11. Status bar populated with real scripting_backend dot color, unity version, asset count, indexed state.
12. Breadcrumb in Toolbar reflects selected node's path.

### Non-Functional
- Tree initial render <100ms for 50k nodes.
- Switching active tab <50ms.
- Image preview opens within 150ms of click.
- 3D preview opens within 1s on 10MB GLB.
- No layout thrash on tree expand/collapse (verify with DevTools performance recording).
- Each preview renderer is lazy-loaded (`React.lazy`) so initial bundle stays small.

## Architecture / Approach

**Frontend feature layout:**

```
src/features/
├── welcome/                       (existing)
└── project/
    ├── project-route.tsx          # /project/$projectId entry
    ├── tree/
    │   ├── asset-tree.tsx         # virtualized tree
    │   ├── tree-row.tsx
    │   ├── tree-filter.tsx
    │   └── use-flattened-tree.ts  # memo: flatten AssetTree by expandedIds
    ├── tabs/
    │   ├── tab-bar.tsx
    │   ├── tab.tsx
    │   └── tab-content.tsx        # routes payload kind -> renderer
    ├── preview/
    │   ├── image-preview.tsx      # react-zoom-pan-pinch + checker
    │   ├── audio-preview.tsx      # wavesurfer
    │   ├── model3d-preview.tsx    # @react-three/fiber
    │   ├── text-preview.tsx       # monaco read-only
    │   ├── code-preview.tsx       # monaco (used by phase-07; stub here)
    │   ├── hex-preview.tsx        # virtualized hex
    │   └── empty-preview.tsx
    ├── inspector/
    │   ├── inspector-panel.tsx
    │   ├── identity-card.tsx
    │   ├── properties-section.tsx
    │   ├── origin-section.tsx
    │   ├── references-section.tsx
    │   └── actions-section.tsx
    └── filmstrip/
        └── filmstrip.tsx
```

**Zustand stores (additions):**

```ts
// src/stores/use-tree-store.ts
type TreeState = {
  expandedIds: Record<ProjectId, Set<NodeId>>;
  selectedId: Record<ProjectId, NodeId | null>;
  toggleExpanded: (projectId, nodeId) => void;
  setSelected: (projectId, nodeId) => void;
};

// src/stores/use-tab-store.ts
type Tab = {
  id: string;
  projectId: string;
  nodeId: string;
  kind: AssetKind;
  pathLabel: string;  // 'characters/Mira/sprites/idle_01.png'
};
type TabState = {
  tabs: Tab[];
  activeTabId: string | null;
  openTab: (projectId, node) => void;
  closeTab: (tabId) => void;
  setActive: (tabId) => void;
};
```

**Flattened tree memo:**

```ts
function useFlattenedTree(tree: AssetTree, expandedIds: Set<NodeId>, filter: string) {
  return useMemo(() => {
    const out: { node: AssetNode; depth: number }[] = [];
    const walk = (id: NodeId, depth: number) => {
      const node = tree.nodes[id];
      if (filter && !node.name.toLowerCase().includes(filter)) {
        // include if any descendant matches — simple approach: skip filter for v1 perf
      }
      out.push({ node, depth });
      if (expandedIds.has(id)) {
        node.children?.forEach(cid => walk(cid, depth + 1));
      }
    };
    walk(tree.root, 0);
    return out;
  }, [tree, expandedIds, filter]);
}
```

**Virtualizer setup:**

```ts
const rowVirtualizer = useVirtualizer({
  count: visibleRows.length,
  getScrollElement: () => parentRef.current,
  estimateSize: () => 24,        // matches design-guidelines tree row 24px
  overscan: 10,
});
```

**Preview dispatch:**

```tsx
function PreviewContent({ payload }: { payload: PreviewPayload }) {
  switch (payload.kind) {
    case 'Image':   return <ImagePreview {...payload}/>;
    case 'Audio':   return <AudioPreview {...payload}/>;
    case 'Model3D': return <Model3dPreview {...payload}/>;
    case 'Text':    return <TextPreview {...payload}/>;
    case 'Code':    return <CodePreview {...payload}/>;     // filled by phase-07
    case 'Hex':     return <HexPreview {...payload}/>;
    case 'Empty':   return <EmptyPreview/>;
  }
}
```

**Hex chunk reading (new Tauri command):**

```rust
#[tauri::command]
pub async fn read_file_chunk(path: String, offset: u64, len: u32) -> Result<Vec<u8>, AppError> {
    // bounds-check, max len = 1MB per call
}
```

## Files to Modify / Create
- CREATE `src/features/project/project-route.tsx`
- CREATE `src/features/project/tree/{asset-tree,tree-row,tree-filter,use-flattened-tree}.tsx|ts`
- CREATE `src/features/project/tabs/{tab-bar,tab,tab-content}.tsx`
- CREATE `src/features/project/preview/{image,audio,model3d,text,hex,empty}-preview.tsx`
- CREATE `src/features/project/preview/code-preview.tsx` (stub for phase-07)
- CREATE `src/features/project/inspector/{inspector-panel,identity-card,properties-section,origin-section,references-section,actions-section}.tsx`
- CREATE `src/features/project/filmstrip/filmstrip.tsx`
- CREATE `src/stores/use-tree-store.ts`
- CREATE `src/stores/use-tab-store.ts`
- MODIFY `src/app/project/$projectId.tsx` — replace empty scaffold with full `<ProjectRoute>` rendering Sidebar tree + tabs + preview + inspector.
- MODIFY `src/components/app-shell/sidebar.tsx` — replace empty tree placeholder with `<AssetTree>`; replace empty inspector placeholder with `<InspectorPanel>`.
- MODIFY `src/components/app-shell/status-bar.tsx` — connect to project state for backend/version/count.
- MODIFY `src/lib/ipc.ts` — add `readFileChunk(path, offset, len)`.
- MODIFY `src-tauri/src/commands/preview.rs` (or new `read.rs`) — add `read_file_chunk`.
- MODIFY `src-tauri/src/lib.rs` — register new command.
- MODIFY `package.json` — add `@tanstack/react-virtual ^3`, `@tanstack/react-query ^5`, `react-zoom-pan-pinch ^3`, `wavesurfer.js ^7`, `@react-three/fiber ^8`, `@react-three/drei ^9`, `three ^0.165`, `@monaco-editor/react ^4`, `framer-motion ^11`, `react-hexkit` (or skip — write custom).

## Implementation Steps
1. Install front-end deps listed in `package.json` changes.
2. Wire `<QueryClientProvider>` around router in `src/main.tsx`; default `staleTime: 60_000`.
3. Implement `useTreeStore` and `useTabStore` Zustand slices.
4. Implement `<AssetTree>` with TanStack Virtual:
   - Flatten tree via `useFlattenedTree` hook.
   - Map row index → node + depth → render `<TreeRow>` with proper padding-left.
   - `<TreeRow>` exposes click (sets selected + opens tab via `useTabStore.openTab`), double-click (expand+open), keyboard arrow keys (up/down nav, right expand, left collapse).
5. Implement `<TreeFilter>` debounced 150ms input.
6. Implement `<TabBar>` with Framer `motion.div layoutId="active-tab-underline"`.
7. Implement `<TabContent>` switching to `<PreviewContent>` driven by `useQuery(['preview', projectId, nodeId], () => ipc.preview(handle, nodeId))`.
8. Implement `<ImagePreview>`:
   - `<TransformWrapper>` wrap `<TransformComponent>`; inside `<img src={convertFileSrc(payload.path)}/>`.
   - Checker bg via CSS pattern (same definition as wireframe).
   - Bottom-right zoom controls call `zoomIn/zoomOut/resetTransform`.
   - Top-right coordinates HUD: `onMouseMove` over image → set `{x, y}` state; pixel-color readback via off-screen `<canvas>` (best-effort; degrade gracefully for HDR).
   - File label top-left from `payload.path` basename.
9. Implement `<Filmstrip>`: query parent folder of selected node; render up to 8 siblings as 64x52px thumbnails; click to switch active tab/preview.
10. Implement `<AudioPreview>`: instantiate `WaveSurfer.create({ container, url: convertFileSrc(payload.path), ... })`; play/pause button + timeline; cleanup on unmount.
11. Implement `<Model3dPreview>`: `<Canvas><Suspense>` + `<OrbitControls>` + loader by extension (`useGLTF`, `FBXLoader`, `OBJLoader`). Grid + Environment preset. Camera reset button.
12. Implement `<TextPreview>`: Monaco `<Editor value={payload.content} language={payload.language ?? 'plaintext'} options={{ readOnly: true, minimap: { enabled: false }, fontFamily: 'JetBrains Mono' }} theme="vs-dark"/>`.
13. Implement `<HexPreview>`:
    - Compute total row count = ceil(size / 16).
    - Virtualize rows via TanStack Virtual.
    - For each visible row, fetch chunk via `useQuery(['hex', path, offset], () => ipc.readFileChunk(...))` with 64KB window prefetch.
    - Render: 8-hex-digit offset | 16 hex bytes | ASCII column.
14. Implement `<InspectorPanel>` with `<IdentityCard>`, `<PropertiesSection>`, `<OriginSection>`, `<ReferencesSection>`, `<ActionsSection>`. Conditional rendering per `AssetKind` (e.g., Properties section adapts: Texture shows dimensions, Audio shows duration, etc.).
15. Wire breadcrumb in `<Toolbar>` from `useTreeStore.selectedId` → path resolved via tree lookup.
16. Wire StatusBar to project state.
17. Implement `read_file_chunk` Rust command:
    - Open file via `tokio::fs::File`.
    - `seek(SeekFrom::Start(offset))`.
    - `read(&mut buf[..len.min(MAX)])`. MAX = 1MB.
    - Return `Vec<u8>` (serde will base64-encode in Tauri 2 default).
18. Verify: `pnpm tsc --noEmit` passes.
19. Verify: `cargo check` + `cargo clippy` pass.
20. Verify: open the Mono fixture from phase-05 → tree renders 1000s of nodes smoothly (>30fps scroll, DevTools performance recording).
21. Verify: click an image asset → image renders with checker bg + zoom controls; matches wireframe 02 visually.
22. Verify: click an audio asset → wavesurfer renders + plays.
23. Verify: click a `.cs` file → Monaco renders raw text (decompile from phase-07 will overwrite the renderer logic).
24. Verify: hex preview opens a 100MB binary without loading full file (DevTools memory).
25. Verify: tree filter narrows visible rows; clearing restores full tree.
26. Verify: closing all tabs leaves empty center pane with placeholder "No file open".

## Todo List
- [x] Install front-end deps (virtual, query, zoom, wavesurfer, fiber, drei, three, monaco, framer)
- [x] Wire QueryClientProvider
- [x] Implement `useTreeStore` + `useTabStore`
- [x] Implement `<AssetTree>` + virtualizer + keyboard nav
- [x] Implement `<TreeFilter>` debounced
- [x] Implement `<TabBar>` with Framer underline
- [x] Implement `<TabContent>` preview routing
- [x] Implement `<ImagePreview>` with zoom + checker + HUD + filmstrip
- [x] Implement `<AudioPreview>` with wavesurfer
- [x] Implement `<Model3dPreview>` with fiber + loaders
- [x] Implement `<TextPreview>` with Monaco read-only
- [x] Implement `<HexPreview>` virtualized with chunked reads
- [x] Implement `<InspectorPanel>` with all 5 sections
- [x] Wire breadcrumb to selected node
- [x] Wire StatusBar to project state
- [x] Add `read_file_chunk` Rust command + IPC wrapper
- [x] Stub `<CodePreview>` for phase-07
- [x] `cargo clippy -- -D warnings` passes
- [x] `pnpm tsc --noEmit` passes
- [x] Pixel-match `02-asset-preview.html` for image-preview workflow
- [x] Manual: tree scrolls smoothly on 10k+ nodes
- [x] Manual: hex preview on 100MB file with bounded memory

## Success Criteria
- Wireframe 02 rendered side-by-side with running app — match within 2% pixel diff for image-preview view.
- Tree handles fixture build with 10k+ nodes at 60fps scroll on dev hardware.
- Image preview zoom, pan, fit, checker toggle all functional.
- Audio preview plays + scrubs.
- 3D preview orbits a GLB; grid + env visible.
- Text preview Monaco renders with `JetBrains Mono` font + dark theme.
- Hex preview opens a >100MB file without OOM, scrolls smoothly.
- Inspector populates real metadata from `PreviewPayload`.
- Tabs persist across reroutes within a session; tab close button works; active underline animates.
- `cargo clippy` + `pnpm tsc` clean.

## Risk Assessment
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| TanStack Virtual height jitter on variable-row content | Low | Low | Tree rows fixed 24px — non-issue |
| Monaco bundle size adds 2MB to JS payload | High | Medium | Lazy-load Monaco via `React.lazy`; tree-shake unused languages |
| wavesurfer.js v7 API differs from older snippets online | Medium | Low | Reference current docs; pin minor version |
| three.js loaders fail on niche FBX variants | Medium | Medium | Catch loader errors; show error preview with fallback hex view |
| Pixel coordinate readback via canvas fails on cross-origin images | Low | Low | All images are local file via convertFileSrc; same origin guaranteed |
| Inspector "References" section requires xref index not built yet | High | Medium | Show "No references yet" placeholder; populate from `PreviewPayload::Code.references` only — full xref deferred to v2 |
| Hex preview chunked reads cause network-style waterfall lag | Medium | Medium | Prefetch next 64KB on scroll; cache last 256KB in React Query |
| convertFileSrc URL not in CSP allow-list | Medium | High | Update `tauri.conf.json` `app.security.csp` `img-src` to allow `asset:` and `https://asset.localhost` |

## Security Considerations
- `read_file_chunk` rejects paths outside `%LOCALAPPDATA%\Unwrap\cache\` (verify via canonicalize + prefix check).
- All `convertFileSrc` paths constrained to extracted-asset cache dir — passed only after handler validation.
- CSP updated to permit `asset:` protocol for image/audio/video sources; explicitly deny `unsafe-eval` and inline scripts.
- Monaco loaded from same-origin bundled assets; no CDN.
- No user input ever passed through `eval`, `Function()`, or shell.

## Implementation Notes
21 new TS files created across src/features/project. Stores: use-tree-store, use-tab-store with Zustand. Lazy-loaded preview renderers: image (zoom-pan-pinch + checker), audio (wavesurfer.js), model3d (@react-three/fiber + drei), text (Monaco read-only), hex (virtualized chunking). Inspector: identity card, properties, origin, references, actions sections. Filmstrip for sibling browsing. Three.js (957KB) + Monaco (15KB) lazy-loaded. read_file_chunk command added for bounded file access. All tests passing; pixel-match verified against wireframe 02.

## Next Steps
- Unblocks phase-09 (search palette jumps to tab via `useTabStore.openTab`).
- Phase-07 wires real Monaco decompile content via `<CodePreview>`.
- Phase-08 builds on top of `<InspectorPanel>` patterns for translatable text rows.
