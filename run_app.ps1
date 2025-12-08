# Quick run script for Repak Gui Revamped
# Just launches the built executable

$scriptRoot = Split-Path -Parent $PSCommandPath
$workspaceRoot = $scriptRoot

$relativePaths = @(
    "target\release\repak-gui.exe",
    "repak-gui\target\release\repak-gui.exe",
    "target\debug\repak-gui.exe"
)

$exePath = $null
foreach ($rel in $relativePaths) {
    $candidate = Join-Path -Path $workspaceRoot -ChildPath $rel
    if (Test-Path $candidate) {
        $exePath = $candidate
        break
    }
}

if ($exePath) {
    # Kill existing instances to ensure clean startup
    Stop-Process -Name "repak-gui" -Force -ErrorAction SilentlyContinue
    
    Write-Host "Launching Repak Gui Revamped..." -ForegroundColor Green
    Start-Process -FilePath $exePath
} else {
    Write-Host "Error: Application not built yet!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please run build_app.ps1 first:" -ForegroundColor Yellow
    Write-Host "  .\build_app.ps1" -ForegroundColor White
    Write-Host ""
    exit 1
}
