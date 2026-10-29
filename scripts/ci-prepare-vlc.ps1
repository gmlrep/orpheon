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

# Keep in sync with src-tauri/src/constants.rs VLC_SYSTEM_DIR
$system = "C:\Program Files\VideoLAN\VLC"
if (Test-Path (Join-Path $system "libvlc.dll")) {
    Write-Host "Vendoring from system VLC: $system"
    & "$PSScriptRoot\vendor-libvlc.ps1" -VlcDir $system -OutDir $OutDir
    exit $LASTEXITCODE
}

$zipName = "vlc-$VlcVersion-win64.zip"
# get.videolan.org often returns an HTML mirror picker that Invoke-WebRequest saves as a fake "zip".
# Prefer curl.exe (-L) and fall back across known CDN mirrors.
$urls = @(
    "https://get.videolan.org/vlc/$VlcVersion/win64/$zipName",
    "https://download.videolan.org/pub/videolan/vlc/$VlcVersion/win64/$zipName",
    "https://ftp.videolan.org/pub/videolan/vlc/$VlcVersion/win64/$zipName",
    "https://mirror.csclub.uwaterloo.ca/videolan/vlc/$VlcVersion/win64/$zipName",
    "https://videolan.mirror.wearetriple.com/vlc/$VlcVersion/win64/$zipName"
)

$tmp = Join-Path $env:TEMP "orpheon-ci"
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
$zipPath = Join-Path $tmp $zipName
$extract = Join-Path $tmp "extract"

function Test-ZipMagic([string]$Path) {
    if (-not (Test-Path $Path)) { return $false }
    $len = (Get-Item $Path).Length
    if ($len -lt 1MB) { return $false }
    $fs = [System.IO.File]::OpenRead($Path)
    try {
        $b0 = $fs.ReadByte(); $b1 = $fs.ReadByte()
        return ($b0 -eq 0x50 -and $b1 -eq 0x4B) # "PK"
    } finally {
        $fs.Dispose()
    }
}

function Get-VlcZip([string]$Url, [string]$OutFile) {
    if (Test-Path $OutFile) { Remove-Item $OutFile -Force }
    Write-Host "Downloading $Url"
    $curl = Get-Command curl.exe -ErrorAction SilentlyContinue
    if ($curl) {
        & curl.exe -L --fail --retry 4 --retry-delay 2 --retry-all-errors `
            --connect-timeout 30 --max-time 600 `
            -o $OutFile $Url
        if ($LASTEXITCODE -ne 0) {
            throw "curl failed (exit $LASTEXITCODE) for $Url"
        }
    } else {
        # Last resort: follow redirects manually; reject HTML bodies.
        $ProgressPreference = "SilentlyContinue"
        Invoke-WebRequest -Uri $Url -OutFile $OutFile -UseBasicParsing -MaximumRedirection 8
    }
    if (-not (Test-ZipMagic $OutFile)) {
        $size = if (Test-Path $OutFile) { (Get-Item $OutFile).Length } else { 0 }
        throw "Downloaded file is not a zip (size=$size). VideoLAN may have returned an HTML mirror page."
    }
}

$downloaded = $false
$lastError = $null
foreach ($url in $urls) {
    try {
        Get-VlcZip -Url $url -OutFile $zipPath
        $downloaded = $true
        break
    } catch {
        $lastError = $_
        Write-Warning $_.Exception.Message
        if (Test-Path $zipPath) { Remove-Item $zipPath -Force -ErrorAction SilentlyContinue }
    }
}

if (-not $downloaded) {
    throw "Failed to download VLC $VlcVersion win64 zip from all mirrors. Last error: $lastError"
}

if (Test-Path $extract) {
    Remove-Item $extract -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $extract | Out-Null

Write-Host "Extracting $zipPath"
Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::ExtractToDirectory($zipPath, $extract)

$vlcRoot = Get-ChildItem $extract -Directory | Select-Object -First 1
if (-not $vlcRoot) {
    throw "VLC zip layout unexpected (no top-level directory)"
}
if (-not (Test-Path (Join-Path $vlcRoot.FullName "libvlc.dll"))) {
    throw "libvlc.dll missing under $($vlcRoot.FullName)"
}

Write-Host "Vendoring from downloaded VLC: $($vlcRoot.FullName)"
& "$PSScriptRoot\vendor-libvlc.ps1" -VlcDir $vlcRoot.FullName -OutDir $OutDir
exit $LASTEXITCODE
