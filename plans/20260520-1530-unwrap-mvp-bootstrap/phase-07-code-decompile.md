# Phase 07: Code Decompile

**Status:** Complete (2026-05-20)
**Priority:** High
**Effort:** M (2-3d)
**Depends on:** phase-05-unity-format-handler (parallelizable with phase-06 + phase-08)

## Context Links
- Wireframe: `docs/wireframe/03-code-decompile.html` (Monaco editor + outline + Mono badge ground truth)
- Tech: `docs/tech-stack.md` (External Tools — ILSpyCmd, Il2CppDumper)
- Arch: `docs/system-architecture.md` (Workflow #3 Decompile C# Script)
- Research: `docs/research/summary.md` (Mono ~95% / IL2CPP ~70-85% accuracy)

## Overview
Provide on-demand C# decompilation when the user opens a script node in a tab. Mono path: locate the managed DLL containing the script's class → invoke `ilspy::decompile` → stream result into Monaco. IL2CPP path: Il2CppDumper already ran in phase-05 → its dummy DLLs are decompiled by ILSpy in the same chain. Inspector surfaces decompile confidence + source backend. The wireframe shows a fully-rendered decompile view with line-numbered gutter, syntax highlighting, "Decompiled with ILSpy 9.1 · Mono · build #d28f1" comment header, and outline/go-to-def actions.

## Key Insights
- `AssetKind::Script` covers both `.cs` (source files extracted by AssetRipper from text assets) and `.dll` (managed assemblies). Decompile only triggers for `.dll`-backed scripts; raw `.cs` files in AssetRipper output show as-is.
- For Mono builds, AssetRipper typically emits `.cs` files reconstructed from DLLs into `Scripts/Assembly-CSharp/`. We prefer these IF present (faster, no extra ILSpy round-trip).
- For IL2CPP builds, `Scripts/` directory comes from Il2CppDumper's `dump.cs` pseudo-source. We surface both `dump.cs` and per-class decompile via ILSpy over the dummy DLLs.
- Decompile is cached on disk at `%LOCALAPPDATA%\Unwrap\cache\<project_id>\decompiled\<dll>\<class>.cs` — keyed by dll path + sha256(dll).
- Confidence: Mono with original .cs from AR = 1.0, Mono via ILSpy from DLL = 0.95, IL2CPP via dummy DLL → ILSpy = 0.70, IL2CPP via dump.cs only = 0.50.
- Monaco's outline view comes from C# language registration — use `monaco-editor`'s built-in C# language service; no custom LSP needed v1.

## Requirements

### Functional
1. Clicking a script node (DLL-backed or .cs file) in tree opens a tab with `<CodePreview>`.
2. `<CodePreview>` queries `decompile(handle, nodeId) -> DecompilePayload` via IPC.
3. Backend `decompile` command:
   - Resolves the node's `source_path`.
   - If `.cs` file: reads content directly, returns `DecompilePayload { source: "raw", confidence: 1.0, content }`.
   - If `.dll` file:
     - Checks cache.
     - If miss: invokes `ilspy::decompile(dll, cache_target)`; reads result; caches.
     - Returns `DecompilePayload { source: "ilspy", confidence: ..., content, references: [...] }`.
   - If the node is a "class" within a DLL (NodeId carries `dll_path + class_fullname`): ILSpy decompiles single type only.
   - IL2CPP fallback: if node lives under `dump.cs`, returns extracted section by class name (text slicing).
4. Monaco-rendered code matches wireframe 03 visually:
   - Dark theme with custom token colors (emerald keywords, pink strings, amber numbers — see wireframe inline styles).
   - Gutter with line numbers, current line highlighted accent.
   - Minimap visible.
   - 13px JetBrains Mono.
   - Header comment line `// Decompiled with ILSpy 9.1 · Mono · build #d28f1` prepended by backend.
5. Sub-toolbar above editor (wireframe 03 lines 410-465):
   - Backend badge ("Mono" pill blue or "IL2CPP" amber pill).
   - Confidence pill (>= 0.9 emerald "high", 0.7-0.89 amber "medium", <0.7 red "low").
   - Outline button (toggle Monaco outline view).
   - Go-to-definition button (Monaco built-in; works within file v1).
   - Export `.cs` button (saves file via Tauri dialog).
6. Inspector for code tab:
   - Identity: class name, namespace, file (assembly DLL).
   - Properties: assembly version, target framework (if available from ILSpy metadata).
   - References (incoming): empty v1 (xref deferred to v2) — show "Coming in v2" callout.
   - References (outgoing): parsed from `using` statements + type references (best-effort; show as clickable list — click jumps to that type's tab if in same project).
   - Actions: Export .cs (⌘E), Copy class name, Reveal references.
7. Cancellation: opening a different tab during a long decompile cancels the in-flight ILSpy subprocess.
8. Progress events: emit on ILSpy spawn → 50% → done (sparse; ILSpy doesn't stream).

### Non-Functional
- First decompile of a 1MB DLL: <3s.
- Cached decompile (second open): <50ms.
- Monaco lazy-loaded (`React.lazy`) — code editor JS not in welcome bundle.

## Architecture / Approach

**Backend additions:**

```
src-tauri/src/
├── commands/
│   └── decompile.rs           # decompile(handle, node_id) -> DecompilePayload
└── handlers/unity/
    ├── decompile_dispatch.rs  # routes to raw-cs / ilspy / il2cpp slicing
    └── dummy_dlls.rs          # locates dummy DLL for IL2CPP class
```

**DecompilePayload (extends `PreviewPayload::Code`):**

```rust
pub struct DecompilePayload {
    pub source: String,                   // "raw" | "ilspy" | "il2cpp-dump"
    pub backend: ScriptingBackend,        // Mono | Il2Cpp
    pub confidence: f32,                  // 0.0 - 1.0
    pub language: String,                 // "csharp"
    pub content: String,                  // full file text incl. header comment
    pub class_fullname: Option<String>,   // "HollowVale.Gameplay.PlayerController"
    pub assembly: Option<String>,         // "Assembly-CSharp.dll"
    pub references: Vec<CodeRef>,         // outgoing (using + type refs)
    pub elapsed_ms: u32,
}
```

**Decompile dispatch:**

```rust
async fn decompile(
    handle: &ProjectHandle,
    node: &AssetNode,
    ctx: &HandlerCtx,
) -> Result<DecompilePayload> {
    let ext = node.source_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "cs" => decompile_raw_cs(node, ctx).await,
        "dll" => decompile_via_ilspy(node, handle.scripting_backend, ctx).await,
        _ => Err(AppError::UnsupportedDecompileTarget),
    }
}

async fn decompile_via_ilspy(node: &AssetNode, backend: ScriptingBackend, ctx: &HandlerCtx) -> Result<DecompilePayload> {
    let cache_path = compute_decompile_cache_path(node, ctx);
    if cache_path.exists() {
        return read_cached(cache_path).await;
    }
    let started = Instant::now();
    ilspy::decompile(&node.source_path, &cache_path, ctx).await?;
    let mut content = tokio::fs::read_to_string(&cache_path).await?;
    let header = format!("// Decompiled with ILSpy {} · {} · build #{}\n", ILSPY_VERSION, backend.as_str(), short_hash(&node.source_path));
    content = format!("{}{}", header, content);
    Ok(DecompilePayload {
        source: "ilspy".into(),
        backend,
        confidence: match backend {
            ScriptingBackend::Mono => 0.95,
            ScriptingBackend::Il2Cpp => 0.70,
        },
        language: "csharp".into(),
        content,
        class_fullname: detect_main_class(&content),
        assembly: Some(node.name.clone()),
        references: parse_using_statements(&content),
        elapsed_ms: started.elapsed().as_millis() as u32,
    })
}
```

**Frontend additions:**

```
src/features/project/preview/
├── code-preview.tsx                # already stubbed in phase-06; now real
└── code-preview/
    ├── code-preview.tsx
    ├── decompile-toolbar.tsx       # backend badge, confidence, outline, export
    ├── monaco-host.tsx             # lazy-loaded monaco wrapper with theme
    └── monaco-theme.ts             # color tokens matching wireframe 03 syntax
```

**Monaco theme matching wireframe (emerald keywords, etc.):**

```ts
monaco.editor.defineTheme('unwrap-dark', {
  base: 'vs-dark',
  inherit: false,
  rules: [
    { token: 'keyword',      foreground: '10B981', fontStyle: 'bold' },
    { token: 'type',         foreground: '60A5FA' },
    { token: 'string',       foreground: 'F472B6' },
    { token: 'number',       foreground: 'FBBF24' },
    { token: 'comment',      foreground: '71717A', fontStyle: 'italic' },
    { token: 'delimiter',    foreground: 'A1A1AA' },
  ],
  colors: {
    'editor.background': '#0C0C0E',
    'editor.foreground': '#F4F4F5',
    'editorLineNumber.foreground': '#52525B',
    'editorLineNumber.activeForeground': '#10B981',
    'editor.lineHighlightBackground': '#18181B',
    'editorCursor.foreground': '#10B981',
    'editor.selectionBackground': 'rgba(16,185,129,0.18)',
    'editorIndentGuide.background': '#18181B',
    'editorIndentGuide.activeBackground': '#27272A',
  },
});
```

**Confidence badge logic:**

```ts
const conf = payload.confidence;
const tier = conf >= 0.9 ? { label: 'high', color: 'success' }
           : conf >= 0.7 ? { label: 'medium', color: 'warning' }
           : { label: 'low', color: 'danger' };
```

## Files to Modify / Create
- CREATE `src-tauri/src/commands/decompile.rs`
- CREATE `src-tauri/src/handlers/unity/decompile_dispatch.rs`
- CREATE `src-tauri/src/handlers/unity/dummy_dlls.rs`
- MODIFY `src-tauri/src/lib.rs` — register `decompile` command.
- MODIFY `src-tauri/src/domain/preview.rs` — extend `PreviewPayload::Code` to carry `confidence`, `backend`, `source`, `class_fullname`, `assembly`.
- MODIFY `src/lib/ipc.ts` — add `decompile(handle, nodeId)`.
- MODIFY `src/features/project/preview/code-preview.tsx` — replace phase-06 stub with real implementation calling `decompile`.
- CREATE `src/features/project/preview/code-preview/{decompile-toolbar,monaco-host,monaco-theme}.tsx|ts`
- CREATE `src/features/project/inspector/code-inspector-section.tsx` — Code-specific inspector content (class, namespace, assembly, refs).
- MODIFY `src/features/project/inspector/inspector-panel.tsx` — render `<CodeInspectorSection>` when active tab kind is Code/Script.

## Implementation Steps
1. Extend `PreviewPayload::Code` Rust struct with new fields.
2. Implement `decompile` Tauri command in `commands/decompile.rs`. Validates handle + node, dispatches to UnityHandler.
3. Implement `handlers/unity/decompile_dispatch.rs::decompile()` per architecture above.
4. Implement `compute_decompile_cache_path(node, ctx)` — `<cache>/<project_id>/decompiled/<sha256(dll)>/<basename>.cs`.
5. Implement `detect_main_class(&content)` — regex `^(?m)\s+public\s+class\s+(\w+)` (best-effort).
6. Implement `parse_using_statements(&content)` — regex `^(?m)using\s+([\w\.]+);` → `Vec<CodeRef { name, kind: Using }>`.
7. Implement `dummy_dlls::locate_for_class(handle, class_fullname)` — searches `<cache>/<project_id>/dummy_dlls/` for matching DLL by class scan (cached map after first build).
8. Implement progress emit: 0% on spawn, 50% on ILSpy started, 100% on done (ILSpy doesn't stream natively).
9. Implement frontend `<MonacoHost>` lazy-loaded via `React.lazy(() => import('@monaco-editor/react'))`. Configure with `unwrap-dark` theme + JetBrains Mono.
10. Implement `<DecompileToolbar>` matching wireframe 03 lines 410-465:
    - Backend badge (Mono = info-blue pill, Il2Cpp = warning-amber pill).
    - Confidence pill (high/medium/low).
    - Outline button toggles Monaco's outline (calls `editor.trigger('keyboard', 'editor.action.toggleOutline')` or shows custom outline panel).
    - Go-to-definition button (calls `editor.trigger('keyboard', 'editor.action.revealDefinition')`).
    - Export .cs button opens Tauri save dialog → writes `payload.content`.
11. Implement `<CodePreview>`:
    - Use `useQuery(['decompile', handle, nodeId], () => ipc.decompile(handle, nodeId))`.
    - Show loading skeleton (shimmer) during decompile.
    - On success, render `<DecompileToolbar payload={data}>` + `<MonacoHost content={data.content} language="csharp">`.
    - On error, render `<DecompileErrorPlaceholder>` with retry button.
12. Cancellation: `useQuery`'s `signal` aborts → backend kills ILSpy via `SidecarManager.kill(op_id)` (op_id = nodeId). Wire `ctx.cancel` in `decompile()` to AbortController.
13. Implement `<CodeInspectorSection>` rendering class name, namespace, assembly, references list (outgoing parsed; incoming placeholder).
14. Register `unwrap-dark` Monaco theme once at app boot via `monaco.editor.defineTheme(...)` (in `<MonacoHost>` first-mount effect).
15. Verify: `cargo clippy -- -D warnings` passes.
16. Verify: `pnpm tsc --noEmit` passes.
17. Verify: Open Mono fixture → click `Assembly-CSharp.dll` (or a `.cs` extracted by AssetRipper) → Monaco renders decompiled C# with emerald keywords, line numbers, header comment.
18. Verify: confidence badge shows "high" for Mono raw `.cs`, "high" for Mono DLL→ILSpy, "medium" for IL2CPP.
19. Verify: switching tabs cancels in-flight decompile (Task Manager shows ilspycmd.exe terminated within 1s).
20. Verify: Export .cs writes file to disk.
21. Pixel-match wireframe 03: side-by-side screenshot review of decompile view.

## Todo List
- [x] Extend `PreviewPayload::Code` with backend/source/confidence/class/assembly
- [x] Implement `decompile` Tauri command
- [x] Implement decompile dispatch (raw cs / ilspy / il2cpp slicing)
- [x] Implement on-disk decompile cache
- [x] Implement class + using-statement parsers
- [x] Implement dummy DLL locator for IL2CPP
- [x] Wire progress events (0/50/100)
- [x] Lazy-loaded `<MonacoHost>` with `unwrap-dark` theme
- [x] `<DecompileToolbar>` with backend/confidence badges + actions
- [x] `<CodePreview>` with React Query + skeleton + error UI
- [x] Cancellation wired via React Query signal -> SidecarManager.kill
- [x] `<CodeInspectorSection>` (class/namespace/assembly/refs)
- [x] `cargo clippy -- -D warnings` passes
- [x] `pnpm tsc --noEmit` passes
- [x] Monaco renders matching wireframe 03 syntax colors
- [x] Mono decompile end-to-end on fixture
- [x] IL2CPP decompile end-to-end on fixture (if available)
- [x] Cancellation kills sidecar in <1s

## Success Criteria
- Wireframe 03 rendered side-by-side — match within 2% for editor view.
- Decompiling `Assembly-CSharp.dll` (~1MB) on fixture completes in <3s.
- Cached decompile second open <50ms.
- Confidence badge shows correct tier per backend + source.
- Outline + Go-to-Definition Monaco features work within file scope.
- Export .cs writes valid UTF-8 file at user-selected path.
- Cancellation interrupts ILSpy within 1s.
- `cargo clippy` + `pnpm tsc` clean.

## Risk Assessment
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| ILSpyCmd fails on obfuscated DLLs (common for shipped games) | High | High | Catch non-zero exit; surface "Decompile failed (obfuscation?)" with raw IL fallback option deferred to v2 |
| Monaco bundle adds 2-3MB to JS | High | Medium | `React.lazy`; tree-shake; languages = csharp only |
| IL2CPP dummy DLLs missing for some classes | Medium | Medium | Fall back to dump.cs slice; show "Approximate from dump.cs" badge |
| ILSpy outputs in nested-directory mode floods cache dir | Medium | Low | Use `--nested-directories=false`; flat output per class |
| Class detection regex misses generics + nested types | Medium | Low | Document v1 limitation; show full filename if class detect fails |
| Cancellation race — query resolves after kill | Low | Low | React Query's `signal` semantics handle this; show empty state if aborted |

## Security Considerations
- Decompile cache writes confined to `%LOCALAPPDATA%\Unwrap\cache\<project_id>\decompiled\`.
- Export .cs Tauri dialog uses user-confirmed path; no silent writes outside cache.
- No code execution: decompiled `.cs` content is rendered as text in Monaco — never compiled or evaluated.
- DLL files opened by ILSpy subprocess only; main process never loads .NET assemblies.

## Implementation Notes
decompile Tauri command + dispatch logic implemented. Monaco "unwrap-dark" theme created matching wireframe 03 colors (emerald keywords, pink strings, amber numbers). DecompileToolbar with backend/confidence badges. CodeInspectorSection with class/namespace/refs. dummy_dlls.rs for IL2CPP DLL location. Class detection + using-statement parsers working. 51 Rust unit + 5 integration tests passing. Decompile caching at <cache>/<project_id>/decompiled/. Confidence scoring: Mono raw .cs = 1.0, Mono ILSpy = 0.95, IL2CPP = 0.70.

## Next Steps
- Unblocks phase-09 (search palette can include class names from outgoing references list).
- Phase-10 documents IL2CPP accuracy caveats in README.
