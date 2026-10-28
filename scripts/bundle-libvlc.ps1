# Copy libVLC 3.x (+ plugins) next to the release binary for Windows packaging.
# Defaults to the in-repo vendor tree (no system VLC install required).
# Usage:
#   .\scripts\bundle-libvlc.ps1 -OutDir .\dist\win
#   .\scripts\bundle-libvlc.ps1 -VlcDir "C:\Program Files\VideoLAN\VLC" -OutDir .\dist\win
param(
    [string]$VlcDir = "$PSScriptRoot\..\vendor\vlc",
    [string]$OutDir = ".\dist\win",
    [string]$ExeSource = ""
)

$ErrorActionPreference = "Stop"

$VlcDir = [System.IO.Path]::GetFullPath($VlcDir)
if (-not (Test-Path (Join-Path $VlcDir "libvlc.dll"))) {
    Write-Error "libvlc.dll not found in $VlcDir. Run .\scripts\vendor-libvlc.ps1 or pass -VlcDir."
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Copy-Item (Join-Path $VlcDir "libvlc.dll") $OutDir -Force
Copy-Item (Join-Path $VlcDir "libvlccore.dll") $OutDir -Force
$plugins = Join-Path $VlcDir "plugins"
if (Test-Path $plugins) {
    $dstPlugins = Join-Path $OutDir "plugins"
    if (Test-Path $dstPlugins) {
        Remove-Item $dstPlugins -Recurse -Force
    }
    Copy-Item $plugins $dstPlugins -Recurse -Force
}

if ($ExeSource -and (Test-Path $ExeSource)) {
    Copy-Item $ExeSource (Join-Path $OutDir (Split-Path $ExeSource -Leaf)) -Force
}

Write-Host "Bundled libVLC from $VlcDir -> $OutDir"
Write-Host "Runtime: plugins next to the exe (or set VLC_PLUGIN_PATH=$OutDir\plugins)."
