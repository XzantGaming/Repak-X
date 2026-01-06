param (
    [string]$Message = "Update backend"
)

Write-Host "=== Backend Push Helper ===" -ForegroundColor Cyan

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

# 4. Check if anything was staged
$status = git status --porcelain
if (-not $status) {
    Write-Host "No backend changes detected to commit." -ForegroundColor Yellow
    Write-Host "Current Git Status:" -ForegroundColor Gray
    git status
    exit
}

# 5. Commit
Write-Host "Committing: $Message" -ForegroundColor Green
git commit -m "$Message"

# 6. Push
Write-Host "Pushing to origin/main..." -ForegroundColor Cyan
git push

Write-Host "Backend push complete!" -ForegroundColor Green
