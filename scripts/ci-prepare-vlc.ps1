# Ensure vendor/vlc has libVLC + plugins for CI builds.
# Prefer an existing vendor tree, then a system VLC install, else download VideoLAN win64 zip.
param(
    [string]$OutDir = "$PSScriptRoot\..\vendor\vlc",
    [string]$VlcVersion = "3.0.21"
)

$ErrorActionPreference = "Stop"
$OutDir = [System.IO.Path]::GetFullPath($OutDir)
$dll = Join-Path $OutDir "libvlc.dll"
$plugins = Join-Path $OutDir "plugins"

if ((Test-Path $dll) -and (Test-Path $plugins)) {
    Write-Host "vendor/vlc already present"
    exit 0
}

$system = "C:\Program Files\VideoLAN\VLC"
if (Test-Path (Join-Path $system "libvlc.dll")) {
    Write-Host "Vendoring from system VLC: $system"
    & "$PSScriptRoot\vendor-libvlc.ps1" -VlcDir $system -OutDir $OutDir
    exit $LASTEXITCODE
}

$zipName = "vlc-$VlcVersion-win64.zip"
$url = "https://get.videolan.org/vlc/$VlcVersion/win64/$zipName"
$tmp = Join-Path $env:TEMP "orpheon-ci"
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
$zipPath = Join-Path $tmp $zipName
$extract = Join-Path $tmp "extract"

Write-Host "Downloading $url"
Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing

if (Test-Path $extract) {
    Remove-Item $extract -Recurse -Force
}
Expand-Archive -Path $zipPath -DestinationPath $extract -Force

$vlcRoot = Get-ChildItem $extract -Directory | Select-Object -First 1
if (-not $vlcRoot) {
    throw "VLC zip layout unexpected"
}

Write-Host "Vendoring from downloaded VLC: $($vlcRoot.FullName)"
& "$PSScriptRoot\vendor-libvlc.ps1" -VlcDir $vlcRoot.FullName -OutDir $OutDir
