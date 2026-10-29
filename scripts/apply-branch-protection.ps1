# Apply master branch protection (requires repo admin + GitHub Pro/Team for private repos).
# Usage: .\scripts\apply-branch-protection.ps1
$ErrorActionPreference = "Stop"
$repo = gh repo view --json nameWithOwner -q .nameWithOwner
$branch = gh repo view --json defaultBranchRef -q .defaultBranchRef.name
$config = Join-Path $PSScriptRoot "..\.github\branch-protection.json"
Write-Host "Applying protection to $repo branch $branch"
gh api "repos/$repo/branches/$branch/protection" -X PUT --input $config
Write-Host "Done."
