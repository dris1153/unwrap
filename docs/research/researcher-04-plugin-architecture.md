# Plugin Architecture for Extensible RE Desktop Tool
**Date:** 2026-05-20 | **Scope:** Format handler plugins, architecture patterns, extensibility design

---

## Executive Summary

**Recommendation: Subprocess CLI + internal format registry (v1) → Script plugins (v2)**.

Start with hard-coded internal modules for Unity, using subprocess pattern for external tools (AssetRipper, Il2CppDumper). Design plugin interface as `IFormatHandler` abstraction now—migrate to subprocess plugins in v2 without core rewrites.

---

## 1. Plugin Architecture Options (Ranked)

| Option | Startup | Isolation | Complexity | Learning Curve | Best For |
|--------|---------|-----------|-----------|-----------------|----------|
| **Subprocess CLI** | Medium | Excellent | Medium | Low | v1: AssetRipper, Il2CppDumper, external tools |
| **Hard-coded modules** | Fast | None | Low | Minimal | v1: Initial Unity support |
| **DLL/native plugins** | Fast | Weak | High | High | Binary format handlers (future) |
| **Python/Lua scripts** | Slow | None | Medium | Medium | v2+: User scripting |
| **WebAssembly** | Medium | Excellent | High | Very High | v3+: Untrusted code |
| **Process isolation (IPC)** | Slow | Excellent | Very High | Very High | Hardened RE tools |

**Why subprocess now?** Avoids API design churn; AssetRipper already CLI-ready; isolated failures; language-agnostic.

---

## 2. Format Detection Strategy

**Multi-stage pipeline** (fail-safe):

1. **Magic bytes** (5-20ms): PK (zip), PAK (Unreal), pck (Godot), MZ (PE), ELF magic
2. **Extension + size heuristics** (fallback): .unity3d bundle markers at offset 0x10
3. **Content sniffing** (manual): Parse headers, version fields
4. **User prompt** (if ambiguous): "Which format? → Unreal PAK / Godot PCK / PE"

**Implementation**: Registry per detector:
```
FormatDetector {
  detect(file) → (format, confidence: 0.0-1.0)
  priority: 100 // Fallback ordering
}
```

**Example**: Ghidra uses architecture modules + magic bytes; Binary Ninja layer detection via `BinaryView.get_view_of_type()`.

---

## 3. Plugin Interface Design (IFormatHandler)

**Core abstraction** (language-agnostic):
```
interface IFormatHandler {
  metadata: {name, version, formats: []}
  detect(file, bytes[0:512]) → confidence [0-1]
  parse(file) → AssetTree {
    node {id, name, type, size, preview?, children[]}
  }
  unpack(node) → Stream/bytes
  preview(node) → Renderable {text|image|tree|binary}
}
```

**Stages**:
1. **Detect**: Fast magic-byte scan
2. **Parse**: Lazy asset tree (defer loading)
3. **Unpack**: Extract asset on demand
4. **Render**: UI preview (hex dump, image, decompiled code)

**Reference**: ILSpy `IDecompiler` interface; Binary Ninja `BinaryView`; Ghidra `Service` model.

---

## 4. Pipeline: Detect → Unpack → Parse → Present

```
File Input
  ↓
[Detect] magic bytes → format confidence
  ↓
[Select Handler] highest confidence plugin
  ↓
[Parse] build asset tree (metadata-only, lazy)
  ↓
[UI Tree View] render hierarchy
  ↓
User selects node
  ↓
[Unpack] extract bytes on demand
  ↓
[Preview] render: ILSpy for code, ImageMagick for sprites, hex for binary
```

**Key principle**: Never load entire file into memory. Stream extraction.

---

## 5. Subprocess Pattern Best Practices

**For AssetRipper.exe, Il2CppDumper.exe, etc.**:

- **Invocation**: `ProcessStartInfo` + `RedirectStandardOutput` (JSON output)
- **Progress reporting**: Parse stdout line-by-line; emit `IProgress<float>` callbacks
- **Cancellation**: `CancellationToken` → `process.Kill()`
- **Error handling**: Trap exit code, parse stderr for contextual errors
- **Resource cleanup**: `using (var process = ...)` or `finally { process?.Kill() }`
- **Timeout**: 5-minute default; user-configurable per tool

**Reference**: NeeView plugin architecture uses subprocess + serialized chunk IPC; Binary Ninja subprocess calls for external analyzers.

---

## 6. Concrete Examples from Production Tools

### **Ghidra (Java + OSGi)**
- Service-oriented: Plugins register via `@ServiceInfo` annotation
- Dependency injection: Auto-load dependent modules (e.g., decompiler needs analyzer)
- Discovery: Classpath scanning + manifest parsing
- **Lesson**: Service abstraction → late binding, no hard coupling

### **Binary Ninja (Python API)**
- One API surface: `BinaryView`, `Architecture`, `Transform`
- Multi-plugin: Same interface, multiple providers (e.g., x86, ARM, RISC-V)
- **Lesson**: Single, well-designed interface scales better than many

### **VS Code (TypeScript)**
- Activation events: Only load extension when needed (e.g., `.json` file opened)
- Process isolation: Extensions run in separate Host process
- Hot reload: Not by default; requires manual restart
- **Lesson**: Lazy activation + process boundary = stable host

### **Obsidian (TypeScript)**
- Plugin API: TypeScript interfaces + event system
- Lifecycle: `load()`, `onload()`, `unload()` hooks
- Discovery: `manifest.json` per plugin (name, version, author, perms)
- **Lesson**: Manifest → discoverability; lifecycle hooks → orderly shutdown

---

## 7. Discovery, Lifecycle, Hot-Reload

### **v1 (Hard-coded + Subprocess)**
- No hot-reload needed; restart app to update external tools
- Lifecycle: Simple—spawn on demand, kill on cancel
- Discovery: Plugin registry in-code; config file for subprocess paths

### **v2 (Script plugins)**
- Discovery: Scan `~/.config/app/plugins/` for `.py` or `.lua` files
- Load: Import dynamically; register with handler registry
- Lifecycle state machine:
  ```
  NotDiscovered → Discovered → Loaded → Active → Unloading → Unloaded
  ```
- Hot-reload: Unload old context, reload new; tricky with Python (reload module state)
- **Reference**: VS Code lazy-loads on activation event; C# AssemblyLoadContext for unload safety

---

## 8. Security & Sandboxing

### **v1 (MVP): No sandboxing**
- Subprocess plugins run with app privileges (acceptable; user controls)
- Trust model: User downloads/installs plugins

### **v2 (Future): Light isolation**
- Run subprocess plugins with restricted file access (e.g., read-only input dir)
- Cap memory/CPU: `Job objects` (Windows), `cgroups` (Linux)

### **v3 (Hardened): WebAssembly**
- Compile format handlers to WASM (e.g., Rust → wasm32 target)
- Host in `wasmtime` or `wasmer` runtime
- **Pros**: Memory-safe, no kernel access, portable
- **Cons**: Learning curve, performance overhead (10–30% typical), limited I/O
- **Reference**: Figma uses WASM for plugins; Slack for custom workflows

---

## 9. v1 Implementation Plan

### **Core architecture:**
1. **Format registry** (`PluginManager`):
   - `List<IFormatHandler> handlers`
   - `Detect(file) → (handler, confidence)`
   - `Parse(file) → AssetTree`

2. **Built-in handlers**:
   - Unity (in-process, C#)
   - Subprocess wrapper:
     - AssetRipper → JSON output parser → AssetTree
     - Il2CppDumper → stdout parser → ILSpy bridge

3. **UI binding**:
   - TreeView node = AssetNode
   - Right-click → preview (ILSpy for code, image viewer for sprites)
   - Drag/drop extraction to filesystem

4. **Config** (`config.json`):
   ```json
   {
     "plugins": {
       "assetRipper": { "path": "tools/AssetRipper.exe", "timeout": 300 },
       "il2CppDumper": { "path": "tools/Il2CppDumper.exe", "timeout": 180 }
     }
   }
   ```

---

## 10. v2+ Roadmap

- **Script plugins** (Python/Lua): Parse custom formats (Unreal PAK, Godot PCK)
- **Hot reload**: Restart app, rescan plugin dir
- **Marketplace**: Plugin index + download UI (Obsidian model)
- **Native plugins** (DLL): C++ handlers for performance-critical formats

---

## Trade-Offs Summary

| Dimension | Hard-coded | Subprocess | Script plugins | WebAssembly |
|-----------|-----------|-----------|-----------------|-------------|
| Time-to-v1 | Fast | Medium | Slow | Very slow |
| Lock-in risk | High | None | Low | Low |
| Debugging | Easy | Medium | Hard | Very hard |
| User extensibility | None | Low (admin) | High | High |
| Adoption friction | None | Tools already exist | Learning curve | No ecosystem yet |

**Verdict**: Subprocess for v1 (quick wins) + IFormatHandler abstraction (future-proof).

---

## Unresolved Questions

1. **Subprocess communication protocol**: Use JSON stdout, structured logging, or binary protocol?
2. **Asset tree serialization**: How to cache parse results across sessions?
3. **Concurrent parsing**: One handler at a time or thread-pool?
4. **Error recovery**: If subprocess crashes, retry or fail gracefully?
5. **Native plugin ABI**: What C++ interface if we add DLL support in v2?
6. **Marketplace auth**: How to sign/verify v2 plugins? Obsidian uses GitHub releases.

---

## Sources
- [Ghidra Plugin Framework](https://ghidra.re/ghidra_docs/api/ghidra/framework/plugintool/Plugin.html)
- [Binary Ninja API Reference](https://api.binary.ninja/)
- [VS Code Extension API](https://code.visualstudio.com/api)
- [Obsidian Plugin Development](https://docs.obsidian.md/Reference/TypeScript+API/Plugin)
- [Plugin Lifecycle Management in C#](https://www.devleader.ca/2026/04/10/plugin-lifecycle-management-in-c-loading-activation-and-unloading)
- [WebAssembly Sandboxing](https://www.cs.cmu.edu/~csd-phd-blog/2023/provably-safe-sandboxing-wasm/)
- [Subprocess IPC Pattern](https://deepwiki.com/neelabo/NeeView/5.2-plugin-architecture)
- [Magic Bytes File Detection](https://www.netspi.com/blog/technical-blog/web-application-pentesting/magic-bytes-identifying-common-file-formats-at-a-glance/)
