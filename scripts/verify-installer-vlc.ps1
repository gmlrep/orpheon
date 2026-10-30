# Confirm MSI (and NSIS when possible) contain libVLC next to the app exe.
param(
    [string]$BundleRoot = "$PSScriptRoot\..\src-tauri\target\release\bundle"
)

$ErrorActionPreference = "Stop"
$BundleRoot = [System.IO.Path]::GetFullPath($BundleRoot)

if (-not (Test-Path $BundleRoot)) {
    throw "bundle output missing: $BundleRoot"
}

$msi = Get-ChildItem (Join-Path $BundleRoot "msi\*.msi") -ErrorAction SilentlyContinue |
    Select-Object -First 1
if (-not $msi) {
    throw "No MSI found under $BundleRoot\msi"
}

$extract = Join-Path $env:TEMP ("orpheon-msi-vlc-check-" + [guid]::NewGuid().ToString("n"))
New-Item -ItemType Directory -Force -Path $extract | Out-Null
try {
    Write-Host "Extracting MSI (administrative): $($msi.FullName)"
    $p = Start-Process -FilePath "msiexec.exe" -ArgumentList @(
        "/a", "`"$($msi.FullName)`"",
        "/qn",
        "TARGETDIR=`"$extract`""
    ) -Wait -PassThru
    if ($p.ExitCode -ne 0) {
        throw "msiexec /a failed with exit code $($p.ExitCode)"
    }

    $dll = Get-ChildItem -LiteralPath $extract -Recurse -Filter "libvlc.dll" -ErrorAction SilentlyContinue |
        Select-Object -First 1
    $core = Get-ChildItem -LiteralPath $extract -Recurse -Filter "libvlccore.dll" -ErrorAction SilentlyContinue |
        Select-Object -First 1
    $plugins = Get-ChildItem -LiteralPath $extract -Recurse -Directory -Filter "plugins" -ErrorAction SilentlyContinue |
        Select-Object -First 1

    if (-not $dll) { throw "MSI extract missing libvlc.dll" }
    if (-not $core) { throw "MSI extract missing libvlccore.dll" }
    if (-not $plugins) { throw "MSI extract missing plugins/" }

    $pluginDlls = @(Get-ChildItem -LiteralPath $plugins.FullName -Recurse -Filter "*.dll" -ErrorAction SilentlyContinue)
    if ($pluginDlls.Count -lt 10) {
        throw "MSI plugins/ looks empty ($($pluginDlls.Count) dlls)"
    }

    Write-Host "MSI OK: $($dll.FullName)"
    Write-Host "MSI OK: $($core.FullName)"
    Write-Host "MSI OK: plugins with $($pluginDlls.Count) dlls under $($plugins.FullName)"
}
finally {
    Remove-Item -LiteralPath $extract -Recurse -Force -ErrorAction SilentlyContinue
}

$nsis = Get-ChildItem (Join-Path $BundleRoot "nsis\*-setup.exe") -ErrorAction SilentlyContinue |
    Select-Object -First 1
if ($nsis) {
    $sevenCandidates = @(
        "${env:ProgramFiles}\7-Zip\7z.exe",
        "${env:ProgramFiles(x86)}\7-Zip\7z.exe"
    )
    $seven = $sevenCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
    if (-not $seven) {
        $cmd = Get-Command 7z -ErrorAction SilentlyContinue
        if ($cmd) { $seven = $cmd.Source }
    }

    if ($seven) {
        Write-Host "Listing NSIS with 7z: $($nsis.FullName)"
        $list = & $seven l -ba "$($nsis.FullName)" 2>&1 | Out-String
        if ($list -notmatch "libvlc\.dll") {
            throw "NSIS setup.exe listing has no libvlc.dll"
        }
        if ($list -notmatch "libvlccore\.dll") {
            throw "NSIS setup.exe listing has no libvlccore.dll"
        }
        Write-Host "NSIS OK: libvlc.dll / libvlccore.dll present in archive listing"
    } else {
        Write-Host "NSIS present but 7-Zip not available - skipped archive listing ($($nsis.Name))"
    }
}

Write-Host "Installer VLC verification passed."
