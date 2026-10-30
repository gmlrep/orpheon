# Populate vendor/vlc with a full libVLC 3.x redistributable (DLLs + plugins).
# Usage:
#   .\scripts\vendor-libvlc.ps1
#   .\scripts\vendor-libvlc.ps1 -VlcDir "C:\Program Files\VideoLAN\VLC"
param(
    [string]$VlcDir = "C:\Program Files\VideoLAN\VLC",
    [string]$OutDir = "$PSScriptRoot\..\vendor\vlc"
)

$ErrorActionPreference = "Stop"

$libvlc = Join-Path $VlcDir "libvlc.dll"
if (-not (Test-Path $libvlc)) {
    throw "libvlc.dll not found in $VlcDir. Install VLC 3.x x64 or pass -VlcDir."
}

$OutDir = [System.IO.Path]::GetFullPath($OutDir)
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

Write-Host "Vendoring libVLC from $VlcDir -> $OutDir"

Copy-Item $libvlc $OutDir -Force
Copy-Item (Join-Path $VlcDir "libvlccore.dll") $OutDir -Force

$pluginsSrc = Join-Path $VlcDir "plugins"
$pluginsDst = Join-Path $OutDir "plugins"
if (-not (Test-Path $pluginsSrc)) {
    throw "plugins/ not found in $VlcDir"
}
if (Test-Path $pluginsDst) {
    Remove-Item $pluginsDst -Recurse -Force
}
Copy-Item $pluginsSrc $pluginsDst -Recurse -Force

foreach ($name in @("COPYING.txt", "COPYING.LGPLv2.1")) {
    $src = Join-Path $VlcDir $name
    if (Test-Path $src) {
        Copy-Item $src $OutDir -Force
    }
}

# Best-effort version from vlc.exe FileVersion.
$version = "unknown"
$vlcExe = Join-Path $VlcDir "vlc.exe"
if (Test-Path $vlcExe) {
    $fi = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($vlcExe)
    if ($fi.FileVersion) { $version = $fi.FileVersion.Trim() }
}
Set-Content -Path (Join-Path $OutDir "VERSION.txt") -Value $version -Encoding ASCII

$vlcLib = Join-Path $OutDir "vlc.lib"
if (-not (Test-Path $vlcLib)) {
    Write-Host "vlc.lib missing - generating import library..."
    & "$PSScriptRoot\generate-vlc-lib.ps1" -VlcDir $VlcDir -OutDir $OutDir
} else {
    Write-Host "Keeping existing vlc.lib"
}

$pluginFiles = Get-ChildItem $pluginsDst -Recurse -File
$pluginCount = $pluginFiles.Count
$pluginMb = [math]::Round((($pluginFiles | Measure-Object Length -Sum).Sum) / 1MB, 1)
Write-Host "Done: VLC $version - $pluginCount plugins ($pluginMb MB)"
Write-Host "Build: cargo build -p orpheon"
