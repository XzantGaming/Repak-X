# ============================================
# Repak GUI - Contributor Build Script
# ============================================
# This script builds the entire project from scratch:
# - C# projects (UAssetBridge, StaticMeshSerializeSizeFixer)
# - Rust workspace (frontend + backend)
# - All dependencies and tools
#
# Usage: .\build_contributor.ps1 [-Configuration <debug|release>]
# ============================================

param(
    [ValidateSet("debug", "release")]
    [string]$Configuration = "release"
)

$ErrorActionPreference = "Stop"

# Color output functions
function Write-Step {
    param([string]$Message)
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host $Message -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "[OK] $Message" -ForegroundColor Green
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Write-Info {
    param([string]$Message)
    Write-Host "-> $Message" -ForegroundColor Yellow
}

# Get workspace root
$workspaceRoot = Split-Path -Parent $PSCommandPath
Push-Location $workspaceRoot

try {
    Write-Step "Repak GUI - Full Contributor Build"
    Write-Info "Configuration: $Configuration"
    Write-Info "Workspace: $workspaceRoot"
    Write-Host ""

    # ============================================
    # Step 1: Check Prerequisites
    # ============================================
    Write-Step "[1/5] Checking Prerequisites"
    
    # Check Rust
    Write-Info "Checking Rust installation..."
    $rustVersion = & cargo --version 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error-Custom "Rust/Cargo not found! Install from https://rustup.rs/"
        exit 1
    }
    Write-Success "Rust: $rustVersion"

    # Check .NET SDK
    Write-Info "Checking .NET SDK installation..."
    $dotnetVersion = & dotnet --version 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error-Custom ".NET SDK not found! Install .NET 8.0 SDK from https://dotnet.microsoft.com/download"
        exit 1
    }
    Write-Success ".NET SDK: $dotnetVersion"

    # Check Node.js (for frontend)
    Write-Info "Checking Node.js installation..."
    $nodeVersion = & node --version 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error-Custom "Node.js not found! Install from https://nodejs.org/"
        exit 1
    }
    Write-Success "Node.js: $nodeVersion"

    # ============================================
    # Step 2: Build C# Projects
    # ============================================
    Write-Step "[2/5] Building C# Projects"

    # Build UAssetBridge
    Write-Info "Building UAssetBridge.exe..."
    $bridgeProject = Join-Path $workspaceRoot "uasset_toolkit\tools\UAssetBridge\UAssetBridge.csproj"
    if (Test-Path $bridgeProject) {
        $bridgeOutput = Join-Path $workspaceRoot "target\uassetbridge"
        New-Item -ItemType Directory -Force -Path $bridgeOutput | Out-Null
        
        & dotnet publish $bridgeProject `
            -c Release `
            -r win-x64 `
            --self-contained false `
            -o $bridgeOutput
        
        if ($LASTEXITCODE -ne 0) {
            Write-Error-Custom "UAssetBridge build failed!"
            exit 1
        }
        
        $bridgeExe = Join-Path $bridgeOutput "UAssetBridge.exe"
        if (Test-Path $bridgeExe) {
            Write-Success "UAssetBridge.exe built successfully"
            Write-Info "Location: $bridgeExe"
        } else {
            Write-Error-Custom "UAssetBridge.exe not found after build!"
            exit 1
        }
    } else {
        Write-Warning "UAssetBridge project not found at $bridgeProject"
    }

    # Build UAssetMeshFixer
    Write-Info "Building UAssetMeshFixer.exe..."
    $fixerProject = Join-Path $workspaceRoot "UAssetAPI\StaticMeshSerializeSizeFixer\UAssetMeshFixer.csproj"
    if (Test-Path $fixerProject) {
        $fixerOutput = Join-Path $workspaceRoot "target\serialsizefixer"
        New-Item -ItemType Directory -Force -Path $fixerOutput | Out-Null
        
        & dotnet publish $fixerProject `
            -c Release `
            -r win-x64 `
            --self-contained true `
            -p:PublishSingleFile=true `
            -o $fixerOutput
        
        if ($LASTEXITCODE -ne 0) {
            Write-Error-Custom "StaticMeshSerializeSizeFixer build failed!"
            exit 1
        }
        
        $fixerExe = Join-Path $fixerOutput "UAssetMeshFixer.exe"
        if (Test-Path $fixerExe) {
            Write-Success "UAssetMeshFixer.exe built successfully"
            Write-Info "Location: $fixerExe"
        } else {
            Write-Error-Custom "UAssetMeshFixer.exe not found after build!"
            exit 1
        }
    } else {
        Write-Warning "StaticMeshSerializeSizeFixer project not found at $fixerProject"
    }

    # ============================================
    # Step 3: Install Frontend Dependencies
    # ============================================
    Write-Step "[3/5] Installing Frontend Dependencies"
    
    $frontendDir = Join-Path $workspaceRoot "repak-gui"
    if (Test-Path $frontendDir) {
        Push-Location $frontendDir
        
        Write-Info "Running npm install..."
        & npm install
        if ($LASTEXITCODE -ne 0) {
            Pop-Location
            Write-Error-Custom "npm install failed!"
            exit 1
        }
        Write-Success "Frontend dependencies installed"
        
        Pop-Location
    } else {
        Write-Error-Custom "Frontend directory not found at $frontendDir"
        exit 1
    }

    # ============================================
    # Step 4: Build Frontend (React)
    # ============================================
    Write-Step "[4/5] Building Frontend (React)"
    
    Push-Location $frontendDir
    
    Write-Info "Building React frontend..."
    & npm run build
    if ($LASTEXITCODE -ne 0) {
        Pop-Location
        Write-Error-Custom "Frontend build failed!"
        exit 1
    }
    Write-Success "Frontend built successfully"
    
    Pop-Location

    # ============================================
    # Step 5: Build Rust Workspace (Backend + Tauri)
    # ============================================
    Write-Step "[5/5] Building Rust Workspace (Backend + Tauri)"

    Write-Info "Building Tauri application via CLI (ensures bundled frontend)..."
    Push-Location $frontendDir

    $tauriArgs = @("build")
    if ($Configuration -eq "debug") {
        $tauriArgs += "--debug"
    }

    & npx tauri $tauriArgs
    $tauriExitCode = $LASTEXITCODE

    Pop-Location

    if ($tauriExitCode -ne 0) {
        Write-Error-Custom "Tauri build failed!"
        exit 1
    }
    Write-Success "Tauri application built successfully"

    # ============================================
    # Build Complete - Summary
    # ============================================
    Write-Step "Build Complete!"
    
    $profileDir = if ($Configuration -eq "release") { "release" } else { "debug" }
    $exePath = Join-Path $workspaceRoot "target\$profileDir\repak-gui.exe"
    $bridgePath = Join-Path $workspaceRoot "target\$profileDir\uassetbridge\UAssetBridge.exe"
    $fixerPath = Join-Path $workspaceRoot "target\serialsizefixer\UAssetMeshFixer.exe"
    
    Write-Host ""
    Write-Host "Built Artifacts:" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    
    if (Test-Path $exePath) {
        Write-Success "Main Application: $exePath"
    } else {
        Write-Warning "Main Application not found at: $exePath"
    }
    
    if (Test-Path $bridgePath) {
        Write-Success "UAssetBridge: $bridgePath"
    } else {
        Write-Warning "UAssetBridge not found at: $bridgePath"
    }
    
    if (Test-Path $fixerPath) {
        Write-Success "SerializeSizeFixer: $fixerPath"
    } else {
        Write-Warning "SerializeSizeFixer not found at: $fixerPath"
    }
    
    Write-Host ""
    Write-Host "To run the application:" -ForegroundColor Yellow
    Write-Host "  .\target\$profileDir\repak-gui.exe" -ForegroundColor White
    Write-Host ""
    Write-Host "To create a distribution package:" -ForegroundColor Yellow
    Write-Host "  .\package_release.ps1 -Configuration $Configuration" -ForegroundColor White
    Write-Host ""

} catch {
    Write-Error-Custom "Build failed with error: $_"
    exit 1
} finally {
    Pop-Location
}
