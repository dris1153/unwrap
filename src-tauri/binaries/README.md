# Sidecar Binaries

This directory contains external tool binaries bundled with Unwrap.
These files are tracked in `.gitignore` due to size — fresh checkouts must
re-acquire them before building.

## Contents

| File | Tool | Version | License | Bundled |
|------|------|---------|---------|---------|
| `AssetRipper-x86_64-pc-windows-msvc.exe` | AssetRipper | 1.3.14 | GPL-3.0 | Yes |
| `ilspycmd-x86_64-pc-windows-msvc.exe` | ILSpyCmd (custom wrapper) | 9.1.0.7988 | MIT | Yes |
| `sidecar-manifest.json` | Manifest | — | — | Yes |
| `Il2CppDumper.exe` | Il2CppDumper | 6.7.42 | MIT | No (lazy-download) |

## Sourcing Binaries After a Fresh Clone

### AssetRipper (v1.3.14)

```powershell
# Download and extract
Invoke-WebRequest -Uri "https://github.com/AssetRipper/AssetRipper/releases/download/1.3.14/AssetRipper_win_x64.zip" `
  -OutFile "$env:TEMP\AssetRipper.zip"
Expand-Archive "$env:TEMP\AssetRipper.zip" -DestinationPath "$env:TEMP\AssetRipper"
Copy-Item "$env:TEMP\AssetRipper\AssetRipper.GUI.Free.exe" `
  "src-tauri\binaries\AssetRipper-x86_64-pc-windows-msvc.exe"
```

Expected SHA-256: `11ec892dcd70b1b86f2e52db631e83e007f6103daa20a9616ecaa7a95baa9f21`

Verify:
```powershell
(Get-FileHash "src-tauri\binaries\AssetRipper-x86_64-pc-windows-msvc.exe" -Algorithm SHA256).Hash
```

### ILSpyCmd (v9.1.0.7988, self-contained wrapper)

The `ilspycmd-x86_64-pc-windows-msvc.exe` bundled here is a self-contained
.NET 8 / win-x64 single-file executable built from a thin wrapper project
referencing `ICSharpCode.Decompiler` and `ICSharpCode.ILSpyX` packages
(version 9.1.0.7988). It is NOT the official `ilspycmd` dotnet tool binary
(which requires a separate `dotnet` runtime install and has a circular
NuGet self-reference that prevents clean self-contained publish).

To rebuild the wrapper:

```powershell
# Requires .NET 8+ SDK
$BUILD = "$env:TEMP\ilspycmd_build"
New-Item -ItemType Directory -Force $BUILD | Out-Null

# Create project (copy from plans/20260520-1530-unwrap-mvp-bootstrap/ilspycmd-wrapper-src/)
dotnet publish "$BUILD\src\ilspywrap.csproj" `
  -r win-x64 --self-contained true `
  -p:PublishSingleFile=true `
  -p:AssemblyName=ilspycmd `
  -o "$BUILD\out"

Copy-Item "$BUILD\out\ilspycmd.exe" `
  "src-tauri\binaries\ilspycmd-x86_64-pc-windows-msvc.exe"
```

Expected SHA-256: `a7051adbef4494774fa6e982c0d016b2441dccf8fe5e824bafb8b8e58a5a90d5`

### Il2CppDumper (NOT bundled — lazy download)

Il2CppDumper is downloaded automatically on first IL2CPP game detection.
It is saved to `%LOCALAPPDATA%\Unwrap\bin\Il2CppDumper.exe`.

Manual download:
```
https://github.com/Perfare/Il2CppDumper/releases/download/v6.7.42/Il2CppDumper-net6.0-x64-v6.7.42.zip
```

Extract `Il2CppDumper.exe` from the zip and place at:
`%LOCALAPPDATA%\Unwrap\bin\Il2CppDumper.exe`

SHA-256 of the zip: update `sidecar-manifest.json` field `sha256` for `il2cpp-dumper`
after manual download — currently set to `"TBD"` (checksum verification skipped
for this entry until pinned).

## Tauri Target Triple Suffix

Tauri 2 requires bundled sidecar binaries to carry the target-triple suffix
on disk. The `tauri.conf.json` `bundle.externalBin` entries list the base
name WITHOUT the suffix — Tauri appends it automatically at build time:

- Config: `"binaries/AssetRipper"`
- Disk:   `binaries/AssetRipper-x86_64-pc-windows-msvc.exe`

## Size Budget

| Binary | Approximate Size |
|--------|-----------------|
| AssetRipper.GUI.Free.exe | ~119 MB |
| ilspycmd (self-contained) | ~69 MB |
| Total installer addition | ~188 MB |

This exceeds the spec's 80 MB estimate. AssetRipper 1.3.14 ships as a
single self-contained GUI exe with all assets embedded. A trim build
(CLI-only, no Avalonia UI) could reduce this significantly — deferred to v2.
