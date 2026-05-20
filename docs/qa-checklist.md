# Manual QA Checklist — Unwrap v0.1

Run against each wireframe after `pnpm tauri dev` with a real Unity build dropped.
Check each item. Log failures as GitHub issues tagged `v0.1.1`.

---

## Wireframe 01 — Welcome screen

- [ ] Welcome screen renders on first launch (no recent projects)
- [ ] Drag-drop zone is visible and accepts a folder drop
- [ ] Dropping a non-Unity folder shows an error or chooser, not a crash
- [ ] Dropping a valid Unity folder triggers the detection + extraction flow
- [ ] Progress bar / spinner shows during extraction
- [ ] After extraction completes, app navigates to the asset preview view
- [ ] Recent projects grid populates on second launch
- [ ] Pinned projects appear at the top of recents grid
- [ ] Status bar shows correct text ("Ready" when idle; operation name when busy)
- [ ] Window controls (minimize, maximize, close) work via custom title bar

---

## Wireframe 02 — Asset browse & preview

- [ ] Asset tree renders for the opened project
- [ ] Tree virtualizes: scrolling 1000+ node list stays smooth (no jank)
- [ ] Folder rows show expand/collapse chevron; clicking toggles children
- [ ] Keyboard navigation: Up/Down moves selection; Right expands; Left collapses; Enter opens tab
- [ ] Tree filter input narrows visible rows in real time
- [ ] Clearing filter restores full tree
- [ ] Clicking a texture node opens image preview tab
- [ ] Image preview supports zoom and pan (pinch or scroll)
- [ ] Clicking an audio node opens audio preview tab
- [ ] Audio preview shows waveform; play/pause button works
- [ ] Clicking a mesh node opens 3D preview tab
- [ ] 3D preview allows orbit (drag), zoom (scroll), and reset
- [ ] Clicking a text/script node opens text preview tab
- [ ] Text preview shows syntax-highlighted content
- [ ] Clicking a binary node opens hex viewer tab
- [ ] Hex viewer paginates large files (no full-file load freeze)
- [ ] Inspector panel toggles via Cmd/Ctrl+\ or inspector button
- [ ] Tab bar: opening multiple nodes creates multiple tabs
- [ ] Tab bar: Cmd/Ctrl+W closes active tab; focus moves to neighbor

---

## Wireframe 03 — Code decompile

- [ ] Clicking a .dll node triggers decompile via ILSpyCmd or IL2CPP path
- [ ] Monaco editor renders decompiled C# with correct syntax colors
- [ ] Editor theme matches dark wireframe (dark background, green/blue/yellow tokens)
- [ ] Confidence badge shows backend (Mono / IL2CPP) and confidence percentage
- [ ] IL2CPP confidence is shown as approximate (~70%) with a warning badge
- [ ] Code outline panel lists classes and methods
- [ ] Clicking an outline entry scrolls Monaco to that symbol
- [ ] References panel lists known usages (may be empty for v0.1)
- [ ] Cmd/Ctrl+E exports decompiled source as a .cs file via save dialog
- [ ] Large DLLs (>500 KB decompiled output) do not freeze the UI during streaming

---

## Wireframe 04 — Translation editor

- [ ] Translate view opens for a project with detected TextAsset strings
- [ ] Source locale selector shows detected locales
- [ ] Target locale selector shows ISO-639 locale list
- [ ] String rows display source text on the left, translation input on the right
- [ ] Typing in translation input updates the row optimistically
- [ ] Status dot for each row reflects: pending (grey), translated (green), review (yellow)
- [ ] Filter dropdown (All / Translated / Pending / Review) narrows visible rows
- [ ] Autosave persists translations across app restart (reopen project, translations still visible)
- [ ] Progress bar at top updates as rows are translated
- [ ] Empty project (no TextAssets) shows a "No translatable strings detected" message

---

## Cross-cutting

- [ ] Cmd+K opens command palette from any screen
- [ ] Palette search filters results in real time
- [ ] Palette keyboard nav: Up/Down moves selection; Enter dispatches command
- [ ] "Toggle Inspector" command toggles panel
- [ ] "Focus Tree Filter" command focuses filter input
- [ ] "Close Project" command returns to welcome screen
- [ ] "About Unwrap" command shows version + license info dialog
- [ ] App handles rapid drag-drop of multiple folders without crash
- [ ] App handles cancellation mid-extraction without leaving corrupt state
- [ ] Status bar always reflects current operation or "Ready"
- [ ] No console errors in Tauri dev tools during normal use

---

## Smoke test (installed MSI only)

- [ ] MSI installs without requiring administrator elevation (or elevation is documented)
- [ ] Installed app launches from Start Menu shortcut
- [ ] NOTICE.txt present at `%PROGRAMFILES%\Unwrap\NOTICE.txt`
- [ ] Drop a Unity build — tree populates, decompile works, palette works
- [ ] Uninstaller removes all files under `%PROGRAMFILES%\Unwrap\`
- [ ] No leftover registry keys or files after uninstall (except user data in `%APPDATA%`)
