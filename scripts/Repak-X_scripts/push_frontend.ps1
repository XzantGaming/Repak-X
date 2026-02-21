# Frontend UI Push Script
# Easily commit and push frontend changes to the repository

param(
    [string]$Message = ""
)

$ErrorActionPreference = "Stop"

# Change to repo root (scripts are in scripts/Repak-X_scripts/, so go up 2 levels)
$scriptDir = $PSScriptRoot
$repoRoot = Split-Path -Parent (Split-Path -Parent $scriptDir)
Set-Location $repoRoot

Write-Host "`n=== Frontend UI Push Script ===" -ForegroundColor Cyan

# Source the version bump utility
. "$scriptDir\version_bump.ps1"

# Check for [run-ci] release
$versionResult = Invoke-VersionBump -RepoRoot $repoRoot

# Check if we need to pull first
Write-Host "`nChecking for remote changes..." -ForegroundColor Cyan
git fetch origin

$localCommit = git rev-parse HEAD
$remoteCommit = git rev-parse origin/main
$baseCommit = git merge-base HEAD origin/main

if ($localCommit -ne $remoteCommit) {
    if ($baseCommit -eq $localCommit) {
        # Local is behind remote
        Write-Host "`n[!] Your local branch is BEHIND remote." -ForegroundColor Red
        Write-Host "    You need to pull before pushing." -ForegroundColor Yellow
        $pullConfirm = Read-Host "Pull now? (Y/n)"
        if ($pullConfirm -eq "n" -or $pullConfirm -eq "N") {
            Write-Host "Aborted. Please pull manually before pushing." -ForegroundColor Red
            exit 1
        }
        Write-Host "`nPulling changes..." -ForegroundColor Cyan
        git pull --no-edit
        if ($LASTEXITCODE -ne 0) {
            Write-Host "`nPull failed! Resolve conflicts manually." -ForegroundColor Red
            exit 1
        }
        Write-Host "Pull successful!" -ForegroundColor Green
    }
    elseif ($baseCommit -eq $remoteCommit) {
        # Local is ahead of remote (normal case for pushing)
        Write-Host "Local is ahead of remote. Ready to push." -ForegroundColor Green
    }
    else {
        # Branches have diverged
        Write-Host "`n[!] Local and remote have DIVERGED." -ForegroundColor Red
        Write-Host "    You need to pull and merge before pushing." -ForegroundColor Yellow
        $pullConfirm = Read-Host "Pull and merge now? (Y/n)"
        if ($pullConfirm -eq "n" -or $pullConfirm -eq "N") {
            Write-Host "Aborted. Please resolve manually." -ForegroundColor Red
            exit 1
        }
        Write-Host "`nPulling and merging..." -ForegroundColor Cyan
        git pull --no-edit
        if ($LASTEXITCODE -ne 0) {
            Write-Host "`nPull/merge failed! Resolve conflicts manually." -ForegroundColor Red
            exit 1
        }
        Write-Host "Pull/merge successful!" -ForegroundColor Green
    }
}
else {
    Write-Host "Local is in sync with remote." -ForegroundColor Green
}

# Check for changes
$status = git status --porcelain
if (-not $status) {
    Write-Host "`nNo changes to commit." -ForegroundColor Yellow
    exit 0
}

# Show what's changed (excluding uasset_toolkit/)
Write-Host "`nChanged files:" -ForegroundColor Yellow
git status --short | Where-Object { $_ -notmatch 'uasset_toolkit/' }

# Get commit message
if (-not $Message) {
    Write-Host ""
    $Message = Read-Host "Enter commit message (or press Enter for default)"
    if (-not $Message) {
        $Message = "UI: Frontend updates"
    }
}

# Stage frontend files
Write-Host "`nStaging frontend changes..." -ForegroundColor Cyan
git add "repak-x/src/**"
git add "repak-x/package.json"
git add "repak-x/package-lock.json"
git add "repak-x/*.js"
git add "repak-x/*.json"
git add "repak-x/*.html"

# Check if there are staged changes
$staged = git diff --cached --name-status
if (-not $staged) {
    Write-Host "`nNo frontend files to commit." -ForegroundColor Yellow
    exit 0
}

Write-Host "`nFiles to commit:" -ForegroundColor Green
# Show with colored status indicators
git diff --cached --name-status | ForEach-Object {
    $parts = $_ -split "`t"
    $status = $parts[0]
    $file = $parts[1]
    
    switch ($status) {
        "A" { Write-Host "  [+] $file" -ForegroundColor Green }      # Added
        "M" { Write-Host "  [~] $file" -ForegroundColor Yellow }     # Modified
        "D" { Write-Host "  [-] $file" -ForegroundColor Red }        # Deleted
        "R" { Write-Host "  [>] $file -> $($parts[2])" -ForegroundColor Cyan }  # Renamed
        default { Write-Host "  [?] $file" -ForegroundColor Gray }   # Other
    }
}

# Confirm
Write-Host ""
$confirm = Read-Host "Commit and push these changes? (Y/n)"
if ($confirm -eq "n" -or $confirm -eq "N") {
    Write-Host "Aborted." -ForegroundColor Red
    git reset HEAD
    exit 1
}

# Commit (prepend [run-ci] if this is a release)
if ($versionResult.RunCI) {
    $Message = "[run-ci] $Message"
}
Write-Host "`nCommitting..." -ForegroundColor Cyan
git commit -m $Message

# Push
Write-Host "`nPushing to remote..." -ForegroundColor Cyan
git push

Write-Host "`n=== Done! ===" -ForegroundColor Green
