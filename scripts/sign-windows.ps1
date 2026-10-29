# Optional Authenticode signing for Windows release binaries.
# No-ops when WINDOWS_CERT_PFX / WINDOWS_CERT_PASSWORD are unset (unsigned CI builds).
param(
    [Parameter(Mandatory = $true)]
    [string[]]$Path,
    [string]$Description = "Orpheon"
)

$ErrorActionPreference = "Stop"

$pfxB64 = $env:WINDOWS_CERT_PFX
$password = $env:WINDOWS_CERT_PASSWORD

if ([string]::IsNullOrWhiteSpace($pfxB64)) {
    Write-Host "WINDOWS_CERT_PFX not set — skipping code signing"
    exit 0
}

$signtool = @(
    "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe"
) | Get-Item -ErrorAction SilentlyContinue |
    Sort-Object FullName -Descending |
    Select-Object -First 1 -ExpandProperty FullName

if (-not $signtool) {
    throw "signtool.exe not found; install Windows SDK signing tools"
}

$tmp = Join-Path $env:TEMP ("orpheon-sign-" + [guid]::NewGuid().ToString("n"))
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
$pfxPath = Join-Path $tmp "cert.pfx"

try {
    [IO.File]::WriteAllBytes($pfxPath, [Convert]::FromBase64String($pfxB64))

    foreach ($file in $Path) {
        if (-not (Test-Path $file)) {
            throw "File to sign not found: $file"
        }
        Write-Host "Signing $file"
        & $signtool sign `
            /fd SHA256 `
            /td SHA256 `
            /tr http://timestamp.digicert.com `
            /f $pfxPath `
            /p $password `
            /d $Description `
            $file
        if ($LASTEXITCODE -ne 0) {
            throw "signtool failed for $file (exit $LASTEXITCODE)"
        }
    }
}
finally {
    Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
