# Write SHA256SUMS.txt for release assets.
param(
    [Parameter(Mandatory = $true)]
    [string]$Dir,
    [string]$OutFile = ""
)

$ErrorActionPreference = "Stop"
$Dir = [IO.Path]::GetFullPath($Dir)
if (-not (Test-Path $Dir)) {
    throw "Directory not found: $Dir"
}
if (-not $OutFile) {
    $OutFile = Join-Path $Dir "SHA256SUMS.txt"
}

$lines = @()
Get-ChildItem -Path $Dir -File |
    Where-Object { $_.Name -ne "SHA256SUMS.txt" } |
    Sort-Object Name |
    ForEach-Object {
        $hash = (Get-FileHash -Algorithm SHA256 -Path $_.FullName).Hash.ToLowerInvariant()
        $lines += "$hash  $($_.Name)"
    }

$lines -join "`n" | Set-Content -Path $OutFile -Encoding ascii -NoNewline
Add-Content -Path $OutFile -Value "`n" -Encoding ascii
Write-Host "Wrote $OutFile"
Get-Content $OutFile
