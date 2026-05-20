# Download Tauri sidecar binaries required for Unwrap development.
# Run after `pnpm install` on a fresh clone:
#     pnpm sidecars
# Or directly:
#     powershell -ExecutionPolicy Bypass -File scripts\download-sidecars.ps1
#
# Idempotent: skips binaries that already exist and pass sha256 verification.
# Pinned versions are documented in src-tauri\binaries\sidecar-manifest.json.

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$binDir = Join-Path $repoRoot "src-tauri\binaries"

if (-not (Test-Path $binDir)) {
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null
}

function Test-Sha256 {
    param([string]$Path, [string]$Expected)
    if (-not (Test-Path $Path)) { return $false }
    $actual = (Get-FileHash -Path $Path -Algorithm SHA256).Hash.ToLower()
    return $actual -eq $Expected.ToLower()
}

# ----------------------------------------------------------------------
# 1. AssetRipper 1.3.14 — auto-download from upstream GitHub release
# ----------------------------------------------------------------------
$arPath = Join-Path $binDir "AssetRipper-x86_64-pc-windows-msvc.exe"
$arSha  = "11ec892dcd70b1b86f2e52db631e83e007f6103daa20a9616ecaa7a95baa9f21"
$arUrl  = "https://github.com/AssetRipper/AssetRipper/releases/download/1.3.14/AssetRipper_win_x64.zip"

Write-Host ""
Write-Host "AssetRipper 1.3.14"
Write-Host "------------------"

if (Test-Sha256 $arPath $arSha) {
    Write-Host "  [ok] already installed (sha256 verified)"
} else {
    Write-Host "  [..] downloading (~120 MB from GitHub releases)..."
    $tmpZip     = Join-Path $env:TEMP "unwrap-ar-$(Get-Random).zip"
    $tmpExtract = Join-Path $env:TEMP "unwrap-ar-extract-$(Get-Random)"
    try {
        $ProgressPreference = "SilentlyContinue"   # speeds up Invoke-WebRequest 10x on Windows
        Invoke-WebRequest -Uri $arUrl -OutFile $tmpZip -UseBasicParsing
        $ProgressPreference = "Continue"

        Expand-Archive -Path $tmpZip -DestinationPath $tmpExtract -Force
        $sourceExe = Join-Path $tmpExtract "AssetRipper.GUI.Free.exe"
        if (-not (Test-Path $sourceExe)) {
            throw "AssetRipper.GUI.Free.exe not found inside archive. Upstream layout may have changed; update this script + sidecar-manifest.json."
        }
        Copy-Item -Path $sourceExe -Destination $arPath -Force

        if (Test-Sha256 $arPath $arSha) {
            Write-Host "  [ok] installed and verified"
        } else {
            $actual = (Get-FileHash -Path $arPath -Algorithm SHA256).Hash.ToLower()
            Write-Warning "  sha256 mismatch"
            Write-Warning "    expected: $arSha"
            Write-Warning "    actual:   $actual"
            Write-Warning "  Upstream may have re-released 1.3.14. Update sidecar-manifest.json + this script."
            exit 1
        }
    } finally {
        if (Test-Path $tmpZip)     { Remove-Item -Force $tmpZip }
        if (Test-Path $tmpExtract) { Remove-Item -Recurse -Force $tmpExtract }
    }
}

# ----------------------------------------------------------------------
# 2. ILSpyCmd 9.1.0.7988 — best-effort via `dotnet tool install`
#
# For DEV workflow we install the official NuGet tool to the user's global
# .dotnet\tools folder and copy from there. This requires .NET 8 SDK on PATH.
#
# For PRODUCTION MSI builds, see src-tauri\binaries\README.md to build the
# self-contained single-file wrapper (~69 MB, no .NET runtime dependency).
# The sha256 in sidecar-manifest.json refers to the self-contained wrapper,
# not the dotnet-tool exe, so sha verification is skipped on the dev path.
# ----------------------------------------------------------------------
$ilspyPath = Join-Path $binDir "ilspycmd-x86_64-pc-windows-msvc.exe"

Write-Host ""
Write-Host "ILSpyCmd 9.1.0.7988"
Write-Host "-------------------"

if (Test-Path $ilspyPath) {
    Write-Host "  [ok] already present at $ilspyPath"
} else {
    $dotnet = Get-Command dotnet -ErrorAction SilentlyContinue
    if (-not $dotnet) {
        Write-Warning "  .NET 8 SDK not found on PATH."
        Write-Warning "  Install:  https://dotnet.microsoft.com/en-us/download/dotnet/8.0"
        Write-Warning "  Then re-run: pnpm sidecars"
        Write-Warning ""
        Write-Warning "  Build (cargo + frontend) still works without ILSpyCmd, but C# decompile"
        Write-Warning "  preview will fail at runtime. Asset browse / image / audio / 3D preview / "
        Write-Warning "  translate / search are all unaffected."
    } else {
        Write-Host "  [..] running 'dotnet tool install --global ilspycmd --version 9.1.0.7988' (~30-60s)..."
        $installOk = $false
        try {
            $output = & dotnet tool install --global ilspycmd --version 9.1.0.7988 2>&1
            $installOk = $LASTEXITCODE -eq 0
        } catch {
            $installOk = $false
        }
        # Tool may report "already installed" — also treat that as success.
        if (-not $installOk) {
            $update = & dotnet tool update --global ilspycmd --version 9.1.0.7988 2>&1
            $installOk = $LASTEXITCODE -eq 0
        }

        $toolPath = Join-Path $env:USERPROFILE ".dotnet\tools\ilspycmd.exe"
        if (Test-Path $toolPath) {
            Copy-Item -Path $toolPath -Destination $ilspyPath -Force
            Write-Host "  [ok] copied from $toolPath"
            Write-Host "       Note: dev-mode binary differs from production wrapper (sha mismatch expected)."
            Write-Host "       For MSI release builds, see src-tauri\binaries\README.md."
        } else {
            Write-Warning "  ILSpyCmd dotnet-tool exe not found at $toolPath after install."
            Write-Warning "  Try manually:"
            Write-Warning "    dotnet tool install --global ilspycmd --version 9.1.0.7988"
            Write-Warning "  Then copy ~\.dotnet\tools\ilspycmd.exe to:"
            Write-Warning "    $ilspyPath"
        }
    }
}

# ----------------------------------------------------------------------
# 3. Il2CppDumper — NOT pre-downloaded.
# The app fetches it lazily into %LOCALAPPDATA%\Unwrap\bin\ on first
# IL2CPP project detection. Manifest pins the upstream zip + sha256.
# ----------------------------------------------------------------------
Write-Host ""
Write-Host "Il2CppDumper"
Write-Host "------------"
Write-Host "  [--] lazy-downloaded by the app on first IL2CPP project detection"
Write-Host "       target: %LOCALAPPDATA%\Unwrap\bin\Il2CppDumper.exe"

Write-Host ""
Write-Host "Sidecars setup complete. Next:"
Write-Host "    pnpm tauri dev"
Write-Host ""
