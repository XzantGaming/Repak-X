param (
    [string]$Message = "Update backend"
)

Write-Host "=== Backend Push Helper ===" -ForegroundColor Cyan

# Change to repo root (scripts are in scripts/Repak-X_scripts/, so go up 2 levels)
$scriptDir = $PSScriptRoot
$repoRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)
Set-Location $repoRoot

# Source the version bump utility
. "$scriptDir\version_bump.ps1"

# Check for [run-ci] release
$versionResult = Invoke-VersionBump -RepoRoot $repoRoot

# 1. Add Rust Source Files (Recursive)
Write-Host "Staging Rust files (*.rs, Cargo.toml, Cargo.lock)..."
git add "**/*.rs"
git add "**/Cargo.toml"

# 2. Add C# Source Files (UAssetAPI and UAssetTool)
Write-Host "Staging C# files (*.cs, *.csproj, *.sln)..."
git add "**/*.cs"
git add "**/*.csproj"
git add "**/*.sln"

# 3. Add Root Configuration and Scripts
Write-Host "Staging scripts and docs (*.bat, *.ps1, *.md)..."
git add "*.bat"
git add "*.ps1"
git add "*.md"
git add ".gitignore"

# 4. Add submodule pointer if it changed
Write-Host "Staging submodule reference (UassetToolRivals)..."
git add "UassetToolRivals"

# 5. Check if anything was staged
$status = git status --porcelain
if (-not $status) {
    Write-Host "No backend changes detected to commit." -ForegroundColor Yellow
    Write-Host "Current Git Status:" -ForegroundColor Gray
    git status
    exit
}

# 6. Commit (prepend [run-ci] if this is a release)
if ($versionResult.RunCI) {
    $Message = "[run-ci] $Message"
}
Write-Host "Committing: $Message" -ForegroundColor Green
git commit -m "$Message"

# 7. Push
Write-Host "Pushing to origin/main..." -ForegroundColor Cyan
git push

Write-Host "Backend push complete!" -ForegroundColor Green
