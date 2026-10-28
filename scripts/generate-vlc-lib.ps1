# Generate Windows import library (vlc.lib) from libvlc.dll.
# Prefer the in-repo vendor tree; used by vendor-libvlc.ps1 when vlc.lib is missing:
#   .\scripts\generate-vlc-lib.ps1
#   .\scripts\generate-vlc-lib.ps1 -VlcDir "C:\Program Files\VideoLAN\VLC"
param(
    [string]$VlcDir = "$PSScriptRoot\..\vendor\vlc",
    [string]$OutDir = "$PSScriptRoot\..\vendor\vlc"
)

$ErrorActionPreference = "Stop"
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$vs = & $vswhere -latest -property installationPath
if (-not $vs) { throw "Visual Studio not found (need dumpbin/lib)" }
$vcvars = Join-Path $vs "VC\Auxiliary\Build\vcvars64.bat"

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Copy-Item (Join-Path $VlcDir "libvlc.dll") $OutDir -Force
Copy-Item (Join-Path $VlcDir "libvlccore.dll") $OutDir -Force

$exports = Join-Path $OutDir "exports.txt"
$def = Join-Path $OutDir "vlc.def"

cmd /c "`"$vcvars`" && dumpbin /EXPORTS `"$(Join-Path $OutDir 'libvlc.dll')`" > `"$exports`""

$names = Get-Content $exports | ForEach-Object {
    if ($_ -match '^\s+\d+\s+[0-9A-F]+\s+[0-9A-F]+\s+(libvlc_\S+)') { $Matches[1] }
}
@("LIBRARY libvlc", "EXPORTS") + ($names | ForEach-Object { "    $_" }) |
    Set-Content -Encoding ASCII $def

cmd /c "`"$vcvars`" && cd /d `"$OutDir`" && lib /def:vlc.def /out:vlc.lib /machine:x64"
Write-Host "Created $OutDir\vlc.lib ($($names.Count) exports)"
Write-Host "Build with: `$env:VLC_LIB_DIR='$OutDir'; cargo build -p orpheon --features vlc"
