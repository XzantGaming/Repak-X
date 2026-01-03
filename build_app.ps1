# Build script for Repak Gui Revamped (Tauri + React)
# NOTE: Tauri's beforeBuildCommand in tauri.conf.json automatically
# runs the frontend build via npm-build.bat, so we don't need to build it separately.

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Building Repak Gui Revamped" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Build Tauri app (includes frontend build via beforeBuildCommand)
Write-Host "Building Tauri application (frontend + backend)..." -ForegroundColor Yellow
Set-Location repak-gui
npx tauri build --no-bundle
if ($LASTEXITCODE -ne 0) {
    Write-Host "Tauri build failed!" -ForegroundColor Red
    Set-Location ..
    exit 1
}
Write-Host "✓ Tauri app built successfully" -ForegroundColor Green
Write-Host ""

Set-Location ..

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Build Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Executable location:" -ForegroundColor Yellow
Write-Host "  target\release\repak-gui.exe" -ForegroundColor White
Write-Host ""
Write-Host "To run the app:" -ForegroundColor Yellow
Write-Host "  .\target\release\repak-gui.exe" -ForegroundColor White
Write-Host ""
