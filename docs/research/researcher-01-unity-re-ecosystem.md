# Unity Reverse Engineering Ecosystem Research
**Date:** 2026-05-20 | **Scope:** Asset extraction, IL2CPP/Mono, decompilation, text translation

---

## 1. State-of-the-Art Tools (2024–2026)

| Tool | Role | Status | Community |
|------|------|--------|-----------|
| **AssetRipper** | Asset → Unity format | Active (GUI) | 1.5k+ ⭐ |
| **UnityPy** | Asset extraction (Python) | Latest: Feb 2026 | PyPI maintained |
| **AssetStudio** | Asset exploration/export | Archived; forks active | Razviar fork 2025+ |
| **Il2CppDumper** | IL2CPP → dummy DLLs | Active | 8.9k ⭐, 1.8k forks |
| **Il2CppInspector** | IL2CPP metadata extraction | Maintained | djkaty/Il2CppInspector |
| **Cpp2IL** | IL2CPP → reconstructed IL | Active research | SamboyCoding |
| **ILSpy** | .NET decompiler (Mono) | **MIT licensed** | icsharpcode, 100%+ adoption |
| **dnSpy/dnSpyEx** | .NET debugger + decompiler | GPL-3.0 (toxic) | dnSpyEx maintained |

---

## 2. Capabilities & Limits Matrix

### Asset Extraction
- **AssetRipper**: All formats (sprites, models, audio, TextAssets), Unity 3.5–6000.5.X ✓
- **UnityPy**: Same scope, Python-native, edit-capable, platform-agnostic ✓
- **AssetStudio**: Original frozen at 2022.1; forks extend to 2024+ ✓

### IL2CPP Decompilation
- **Il2CppDumper**: Generates dummy DLLs (classes, methods, strings). **Cannot reconstruct IL** → must pipe to ILSpy/dnSpy.
- **Il2CppInspector**: Metadata + IDA/Ghidra annotations. Better for static analysis.
- **Cpp2IL**: Reconstructs IL from C++ binary → **direct ILSpy compatibility** (gold standard but unstable).

### Mono Decompilation
- **ILSpy/dnSpy**: Direct from Assembly-CSharp.dll → near-perfect 1:1 C# recovery. **Mono = trivial, IL2CPP = hard.**

### String/Text Extraction
- **TextAsset parsing**: AssetRipper + UnityPy extract raw strings.
- **TMP fonts**: Font metadata available but glyph extraction varies by tool.
- **Localization packages**: Found in ScriptableObject asset blobs; requires semantic understanding.

---

## 3. Licensing Analysis (CRITICAL FOR EMBEDDING)

| Tool | License | Embed Risk | Notes |
|------|---------|------------|-------|
| **AssetRipper** | GPL-3.0 | 🔴 HIGH | Viral—any derivative must be GPL-3.0 |
| **UnityPy** | ? (check) | 🟡 CHECK | Python library, CI suggests permissive |
| **ILSpy** | **MIT** | 🟢 SAFE | Can embed, modify, license proprietary wrapper |
| **dnSpy** | GPL-3.0 | 🔴 HIGH | Unsustainable for commercial product |
| **Il2CppDumper** | MIT | 🟢 SAFE | Generate, distribute freely |
| **Il2CppInspector** | ? (check) | 🟡 CHECK | Verify before embedding |
| **Cpp2IL** | MIT | 🟢 SAFE | Reconsts IL from binary |

**Recommendation**: GPL tools are exclusions. **Avoid AssetRipper embedding; use as CLI subprocess only.**

---

## 4. Integration Approaches (Ranked for MVP)

### **Tier 1: Subprocess CLI** (RECOMMENDED)
- **Best for**: AssetRipper (GPL issue), Il2CppDumper, Cpp2IL
- **Method**: Spawn process, capture stdout, parse JSON/text output
- **Pros**: Isolation, no licensing entanglement, updatable separately
- **Cons**: IPC overhead, error handling complexity
- **Verdict**: Use for GPL + large tools

### **Tier 2: Library Embedding** (Python/C#)
- **Best for**: UnityPy (Python library), ILSpy.Abstraction (C# NuGet)
- **Method**: Direct API calls, in-process
- **Pros**: Low latency, tight integration, direct error handling
- **Cons**: Licensing constraints, version lock-in
- **Verdict**: Use UnityPy for fast asset reading; ILSpy for IL decompilation

### **Tier 3: HTTP API** (Not viable MVP)
- Unnecessary complexity for 2-3 tools; subprocess simpler for now.

---

## 5. IL2CPP vs Mono Coverage

| Scenario | Tool Chain | Difficulty | Output |
|----------|-----------|-----------|--------|
| **Mono game** | Extract .dll → ILSpy | Easy (2 steps) | ~95% accurate C# |
| **IL2CPP game** | Il2CppDumper + ILSpy | Hard (3+ steps) | ~70% accurate pseudo-C# |
| **IL2CPP game (ideal)** | Cpp2IL + ILSpy | Hard but better | ~85% accurate pseudo-C# |
| **IL2CPP + string obfuscation** | Il2CppInspector + IDA/Ghidra | Expert-only | 50% recovery |

**Key insight**: Mono is 1–2 clicks. IL2CPP is entire workflow + manual analysis.

---

## 6. Translatable Text Extraction

### Storage locations in Unity:
1. **TextAsset** (.txt, .json, .yaml): Plaintext in serialized asset → easy extraction
2. **ScriptableObject**: Structured data; requires semantic parsing
3. **TMP fonts**: Font asset metadata, glyphs embedded (extraction tool-dependent)
4. **Localization package**: Official `com.unity.localization` stores strings in language-specific tables (complex)

### Extraction approach:
- AssetRipper/UnityPy: Dump all TextAssets + ScriptableObject YAML → grep/parse for strings
- **No tool extracts UI labels directly**; must parse scene hierarchies + code references

---

## 7. Decompiled Code Display Best Practices

- **Syntax highlighting**: ILSpy native; build atop VS Code Monaco editor for desktop app
- **Cross-references**: ILSpy supports click-to-definition; implement breadcrumb navigation
- **Search**: Index dumped IL/C# locally; avoid full-text parsing per query
- **Scope limitation**: Show decompiled method bodies; don't require full AST navigation

---

## 8. MVP Tool Foundation (RANKED RECOMMENDATION)

### **Primary Stack** (Win32-first desktop app)
1. **AssetRipper (subprocess)** → Extract all assets (no embedding; GPL-safe)
   - Latest: Supports up to Unity 6000.5.X
   - Output: Native Unity objects, textures, models, TextAssets
   
2. **UnityPy (embedded Python library)** → Fallback asset reading + string indexing
   - Advantage: Pure Python, no GPL, scriptable
   - Latest release: Feb 2026
   - Integration: Invoke as library from C#/.NET via subprocess or ctypes wrapper
   
3. **ILSpy (embedded C# decompiler via NuGet)** → Display decompiled Mono code
   - License: MIT → safe to distribute
   - Integration: Include `ILSpy.Abstraction` NuGet package, hook into syntax highlighter

### **Conditional (IL2CPP games)**
- **Il2CppDumper (subprocess)** → Generate dummy DLLs
  - Pipe output through ILSpy
  - Trade-off: Accuracy loss (~70%) vs speed
  
- **Cpp2IL (subprocess, experimental)** → Better IL reconstruction
  - Emerging tool; less stable but better output
  - Use if project can tolerate alpha code

### **Exclusion List**
- ❌ **AssetStudio embedding**: Archived; use Razviar fork via subprocess if needed
- ❌ **dnSpy embedding**: GPL-3.0 viral; use ILSpy instead
- ❌ **Il2CppInspector embedding**: Specialized; only via subprocess for expert mode

---

## 9. Summary Recommendation

**MVP architecture for Windows desktop app:**
1. **Asset layer**: AssetRipper (subprocess) + UnityPy (fallback, embedded)
2. **IL decompilation layer**: Il2CppDumper (subprocess) → ILSpy (embedded, Mono decompiler)
3. **UI layer**: C#/.NET WPF/WinUI3 with Monaco editor + navigation breadcrumbs
4. **String extraction**: TextAsset dump + simple regex on YAML/JSON outputs

**Phase 2** (post-MVP): Cpp2IL integration for better IL2CPP accuracy; TMP font character-level extraction.

---

## Unresolved Questions
- UnityPy license: Verify before committing to embedding; check if MIT/Apache or if GPL-adjacent
- Il2CppInspector license: Same check needed before embedding
- ILSpy NuGet versioning: Confirm stable API surface for cross-version compatibility
- Asset bundle nested extraction: Do tools recursively unpack bundles within bundles?
- TMP glyph extraction: No tool found; may require custom bitmap processing

---

## Sources
- [How to Reverse Engineer a Unity Game | Kodeco](https://www.kodeco.com/36285673-how-to-reverse-engineer-a-unity-game)
- [UnityPy on PyPI](https://pypi.org/project/UnityPy/)
- [AssetRipper GitHub](https://github.com/AssetRipper/AssetRipper)
- [Il2CppDumper GitHub](https://github.com/perfare/il2cppdumper)
- [Il2CppInspector GitHub](https://github.com/djkaty/il2cppinspector)
- [Cpp2IL GitHub](https://github.com/SamboyCoding/Cpp2IL)
- [ILSpy GitHub](https://github.com/icsharpcode/ILSpy)
- [dnSpy GitHub](https://github.com/dnSpy/dnSpy)
- [Unity Localization Package Docs](https://docs.unity3d.com/Packages/com.unity.localization@1.4/)
- [AssetStudio forks (2025 maintained)](https://github.com/Razviar/assetstudio)
- [Licensing Guide](https://www.opensourcealternatives.to/blog/open-source-license-guide)
