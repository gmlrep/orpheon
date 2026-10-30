# Stage libVLC into src-tauri/vlc-runtime for Tauri bundle.resources.
# Paths with ../vendor are unreliable in installers; keep a copy under src-tauri.
param(
    [string]$VendorDir = "$PSScriptRoot\..\vendor\vlc",
    [string]$OutDir = "$PSScriptRoot\..\src-tauri\vlc-runtime"
)

$ErrorActionPreference = "Stop"

if ($env:OS -and $env:OS -ne "Windows_NT") {
    Write-Host "stage-vlc-for-bundle: skip (non-Windows)"
    exit 0
}

$VendorDir = [System.IO.Path]::GetFullPath($VendorDir)
$OutDir = [System.IO.Path]::GetFullPath($OutDir)

$libvlc = Join-Path $VendorDir "libvlc.dll"
$libvlccore = Join-Path $VendorDir "libvlccore.dll"
$plugins = Join-Path $VendorDir "plugins"

if (-not (Test-Path $libvlc)) {
    Write-Error "Missing $libvlc - run scripts/ci-prepare-vlc.ps1 or scripts/vendor-libvlc.ps1"
}
if (-not (Test-Path $libvlccore)) {
    Write-Error "Missing $libvlccore"
}
if (-not (Test-Path $plugins)) {
    Write-Error "Missing $plugins"
}

if (Test-Path $OutDir) {
    Remove-Item $OutDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

Copy-Item $libvlc (Join-Path $OutDir "libvlc.dll") -Force
Copy-Item $libvlccore (Join-Path $OutDir "libvlccore.dll") -Force
Copy-Item $plugins (Join-Path $OutDir "plugins") -Recurse -Force

foreach ($name in @("COPYING.txt", "COPYING.LGPLv2.1")) {
    $src = Join-Path $VendorDir $name
    $dst = Join-Path $OutDir $name
    if (Test-Path $src) {
        Copy-Item $src $dst -Force
    } else {
        Set-Content -Path $dst -Value "See https://www.videolan.org/legal.html and vendor/vlc licenses."
    }
}

Write-Host "Staged libVLC for installer bundle: $OutDir"
Get-ChildItem $OutDir | Format-Table Name, Length
