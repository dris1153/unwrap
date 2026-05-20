# UI/UX Research: RE Tools Design Patterns

**Scope:** 9 tools analyzed (Ghidra, IDA Pro, Binary Ninja, dnSpy, JADX-GUI, Cutter, AssetStudio, AssetRipperGUI, UABE). Modern design ref: VS Code, Rider, Linear, Warp.

## 1. Dominant Layout Patterns

**Multi-pane DDD** (Data-Driven Design):
- **Left:** File tree + navigation (narrow, collapsible)
- **Center:** Primary editor (code/hex/decompile) — full width focus
- **Right:** Properties/inspector (collapsible, context-dependent)
- **Bottom:** Search results, output, console (tab-based, resizable)

**IDA/Binary Ninja standard:** Tabs within panes. Customizable drag-to-reorganize.
**Ghidra/Cutter:** Pinned xref panels (tabs = modal-lite UX, avoids dialog fatigue).
**dnSpy/JADX:** Simplified 2-pane (file tree left, editor right) — lower cognitive load.

## 2. Essential Panel Types

| Panel | Purpose | Interaction |
|-------|---------|-------------|
| File Tree | Navigation | Expand/collapse, search, drag-drop |
| Hex Viewer | Binary inspection | Addr + byte cells (ImHex: custom cell viz) |
| Decompile | Code view | Syntax highlight, jump-to-def, xref tabs |
| Asset Preview | 3D/image/audio | Zoom/pan, orbit controls, play buttons |
| Inspector | Props/metadata | Read-only or inline edit, copy buttons |
| Search | Global find | Fuzzy matching, filter by type |
| Xref | Cross-references | Pinned tabs (Ghidra style), not modal dialogs |

## 3. Modern Design Language

**VS Code/Rider principles:**
- High contrast accent colors (not neon, not gray)
- Monospace: JetBrains Mono, Fira Code, IBM Plex (ligatures optional)
- Inter/System UI fonts for UI text (not monospace for labels)
- Dark-first (true dark, not near-black), optional light mode
- 16px base, 12px small, 14px standard code
- Aggressive whitespace; icons > text labels
- Shadows for depth (not borders)
- Focus rings (2px, accent color) for accessibility

**Rider's "New UI" wins:** Large tabs (better hit target), reduced icon clutter, progressive disclosure (menus collapse to icons).

## 4. Traditional RE Tool Failures → Fixes

| Problem | Root Cause | Modern Fix |
|---------|-----------|-----------|
| Cramped toolbar | 90s density | Icon+label collapse; groups hide in menu |
| Gray palette | Default Windows | Curated dark theme (Dracula, Nord, One Dark) |
| Modal dialogs | Pre-tabbed era | Pinned side panels (xref, search, inspector) |
| Inconsistent spacing | No design system | 8px grid, 4px increments |
| Unreadable tabs | Small font | Larger tabs, scroll/wrap overflow |
| Drag-drop broken | No affordance | Visual feedback (drop zone highlight) |
| Poor empty state | Overlooked | Hint text ("Drop APK here") + onboarding |

## 5. Asset Preview Patterns

**Image:** ImHex-style (zoom/pan with checker bg, histogram side-by-side).
**3D Models:** Orbit controls (drag to rotate), zoom scroll, grid floor, lighting toggle. Format support: GLB, FBX, OBJ (use Babylon.js or Three.js library).
**Audio:** Waveform display (interactive scrub), play/pause, speed control (0.5x–2x).
**Text:** Translate-column layout (original | translation, diff highlight).
**Hex:** Byte-addressable, offset column, ASCII preview, search highlight overlay.

## 6. Navigation Patterns (Recommended)

**Primary:** Tabbed editing (familiar, VS Code style). Tabs show file path breadcrumb or icon + name.
**Secondary:** Split panes (vertical/horizontal). Avoids modal dialogs entirely.
**Jump-to-symbol:** Cmd/Ctrl+K search (fuzzy, cross-file, type-aware).
**Back/forward:** History stack (not browser-style buttons), Cmd/Ctrl+[ / ]).
**Breadcrumb:** Optional; show context path (file > function > block) in subtitle, not header.

## 7. Onboarding & Empty States

**Entry point:** Drag-drop zone (center, prominent).
```
┌─────────────────────────────┐
│  Drop .exe, .apk, or .dll   │
│       to start               │
└─────────────────────────────┘
[or click to browse]
```

**Recent files:** Grid below (thumbnail + name, delete button on hover).
**Project creation:** Wizard skips steps (direct drop → auto-detect type → done).
**First-time help:** Inline tooltips (?) + collapsible "Getting Started" panel (right side, closable).

## 8. Color & Typography (Dark-First)

**Dark palette example:**
- Background: #0D1117 (GitHub dark)
- Surface: #161B22
- Text: #E6EDF3
- Accent: #58A6FF (blue) or #79C0FF (lighter context)
- Warning: #F0883E (orange)
- Error: #F85149 (red)

**Monospace fonts ranked:**
1. JetBrains Mono (default weight 400, ligatures on)
2. Fira Code (11–13px, perfect for code)
3. IBM Plex Mono (clean italics for comments)

**Avoid:** Consolas (Windows), Courier (90s), Comic Sans (😱).

**Typography scale:** 12px (smallest), 13px (UI), 14px (body code), 16px (headers).

## 9. Project/Workspace Model

**Multi-project support:**
- Sidebar: Pinned projects (icon + name, context-menu to unpin)
- Workspace file: `.re-workspace.json` (format list, metadata, layout state)
- Per-project state: `.re/project.json` in folder (open files, breakpoints, notes)
- Recents tab: Global recent list, searchable

**Organization:** Folder hierarchy (game mods grouped by game; each APK/EXE = project).

## Top 5 Design Patterns to Steal

1. **Pinned panels > modal dialogs** (Ghidra's xref tabs) — 10x better UX for cross-ref workflows.
2. **Drag-drop as primary entry** — 80% of users expect it; offloads file browser.
3. **Dark theme by default + high contrast accents** — Reduces eye strain, increases perceived quality.
4. **Icon + label collapsible toolbar** (Rider New UI) — Saves 20% vertical space without sacrificing discoverability.
5. **Monospace + sans-serif split** (Rider: code = JetBrains Mono, UI = Inter) — Improves scannability.

---

## Key Sources

- [Binary Ninja 5.3 Migration Guide](https://docs.binary.ninja/guide/migration/migrationguideida.html) — Cross-tool UI compatibility insight
- [JetBrains Rider New UI](https://www.jetbrains.com/help/rider/New_UI.html) — Progressive disclosure, tab legibility
- [ImHex Hex Editor](https://github.com/werwolv/imhex) — Modern RE-tool reference (custom visualizers)
- [Dracula Theme Fonts Guide](https://draculatheme.com/blog/best-free-fonts-for-programming) — Monospace font recommendations
- [NN/G Tabs Best Practices](https://www.nngroup.com/articles/tabs-used-right/) — Tab UX patterns
- [Userlane Legacy UI Modernization](https://www.userlane.com/blog/legacy-issues-with-ui-and-ux/) — Why legacy tools feel bad
- [VS Code User Interface](https://code.visualstudio.com/docs/getstarted/userinterface) — Split pane, breadcrumb patterns

---

## Unresolved Questions

1. **Collaborative RE workflows:** Multi-cursor editing in decompile view? (High scope, defer Phase 2)
2. **Custom cell visualizers in hex:** Endianness, struct overlay UI? (AssetStudio model; research Phase 2)
3. **Asset format support priority:** Which 3D formats to support in MVP? (Stakeholder decision)
4. **Search indexing perf:** Fuzzy symbol search on 100MB+ binaries? (PoC needed)
5. **Version control integration:** Git diff for decompiled code? (Out of scope MVP)
