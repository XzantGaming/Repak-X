@echo off
setlocal enabledelayedexpansion

REM ============================================================
REM  Repak GUI Revamped - Package Release Script
REM  Creates a distributable ZIP with all required components
REM ============================================================

set VERSION=2.6.4
set DIST_NAME=Repak-Gui-Revamped-v%VERSION%
set DIST_DIR=%~dp0dist\%DIST_NAME%
set ZIP_FILE=%~dp0dist\%DIST_NAME%.zip

echo.
echo ============================================================
echo  Repak GUI Revamped - Package Release Script v%VERSION%
echo ============================================================
echo.

REM Check for required tools
where dotnet >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: .NET SDK not found! Please install .NET 8.0 SDK.
    echo Download: https://dotnet.microsoft.com/download/dotnet/8.0
    pause
    exit /b 1
)

where cargo >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Rust/Cargo not found! Please install Rust.
    echo Download: https://rustup.rs/
    pause
    exit /b 1
)

where npm >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Node.js/npm not found! Please install Node.js.
    echo Download: https://nodejs.org/
    pause
    exit /b 1
)

REM Clean previous dist
echo [1/8] Cleaning previous build artifacts...
if exist "%~dp0dist" rmdir /s /q "%~dp0dist"
mkdir "%DIST_DIR%"
mkdir "%DIST_DIR%\uassetbridge"

REM ============================================================
REM  Step 2: Build Modified UAssetAPI Library First
REM ============================================================
echo.
echo [2/8] Building Modified UAssetAPI library (.NET 8.0)...
pushd "%~dp0UAssetAPI\UAssetAPI"
dotnet build -c Release
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Failed to build UAssetAPI library!
    popd
    pause
    exit /b 1
)
popd

REM ============================================================
REM  Step 3: Build .NET Tools (depends on UAssetAPI)
REM ============================================================
echo.
echo [3/8] Building StaticMeshSerializeSizeFixer (.NET 8.0)...
pushd "%~dp0UAssetAPI\StaticMeshSerializeSizeFixer"
dotnet publish -c Release -r win-x64 --self-contained true -p:PublishSingleFile=true
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Failed to build StaticMeshSerializeSizeFixer!
    popd
    pause
    exit /b 1
)
popd

echo.
echo [4/8] Building UAssetBridge (.NET 8.0)...
pushd "%~dp0uasset_toolkit\tools\UAssetBridge"
dotnet publish -c Release -r win-x64 --self-contained true -p:PublishSingleFile=true
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Failed to build UAssetBridge!
    popd
    pause
    exit /b 1
)
popd

REM ============================================================
REM  Step 5: Build Frontend
REM ============================================================
echo.
echo [5/8] Building Frontend (React + Vite)...
pushd "%~dp0repak-gui"
call npm install
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: npm install failed!
    popd
    pause
    exit /b 1
)
call npm run build
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: npm build failed!
    popd
    pause
    exit /b 1
)
popd

REM ============================================================
REM  Step 6: Build Tauri App
REM ============================================================
echo.
echo [6/8] Building Tauri Application (Release)...
pushd "%~dp0repak-gui"
cargo tauri build --no-bundle
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Tauri build failed!
    popd
    pause
    exit /b 1
)
popd

REM ============================================================
REM  Step 7: Assemble Distribution
REM ============================================================
echo.
echo [7/8] Assembling distribution package...

REM Copy main executable
echo   - Copying repak-gui.exe...
copy "%~dp0target\release\repak-gui.exe" "%DIST_DIR%\" >nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: repak-gui.exe not found! Build may have failed.
    pause
    exit /b 1
)

REM Copy Oodle DLL
echo   - Copying oo2core_9_win64.dll...
copy "%~dp0oo2core_9_win64.dll" "%DIST_DIR%\" >nul
if %ERRORLEVEL% NEQ 0 (
    echo WARNING: oo2core_9_win64.dll not found in workspace root!
    echo          Oodle compression will not work without this file.
)

REM Copy StaticMeshSerializeSizeFixer
echo   - Copying StaticMeshSerializeSizeFixer.exe...
copy "%~dp0UAssetAPI\StaticMeshSerializeSizeFixer\bin\Release\net8.0\win-x64\publish\StaticMeshSerializeSizeFixer.exe" "%DIST_DIR%\" >nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: StaticMeshSerializeSizeFixer.exe not found!
    pause
    exit /b 1
)

REM Copy UAssetBridge to uassetbridge subfolder
echo   - Copying UAssetBridge.exe...
copy "%~dp0uasset_toolkit\tools\UAssetBridge\bin\Release\net8.0\win-x64\publish\UAssetBridge.exe" "%DIST_DIR%\uassetbridge\" >nul
if %ERRORLEVEL% NEQ 0 (
    REM Try the target/release path as fallback (copied by build.rs)
    copy "%~dp0target\release\uassetbridge\UAssetBridge.exe" "%DIST_DIR%\uassetbridge\" >nul
    if %ERRORLEVEL% NEQ 0 (
        echo WARNING: UAssetBridge.exe not found!
        echo          Texture pipeline will not work without this file.
    )
)

REM Copy icon if exists
if exist "%~dp0repak-gui\icons\RepakIcon.ico" (
    echo   - Copying application icon...
    copy "%~dp0repak-gui\icons\RepakIcon.ico" "%DIST_DIR%\" >nul
)

REM Create README for distribution
echo   - Creating README.txt...
(
echo ============================================================
echo  Repak GUI Revamped v%VERSION%
echo  Marvel Rivals Mod Installer
echo ============================================================
echo.
echo CONTENTS:
echo   repak-gui.exe                    - Main application
echo   oo2core_9_win64.dll              - Oodle compression library
echo   StaticMeshSerializeSizeFixer.exe - Static mesh fixer ^(uses modified UAssetAPI^)
echo   uassetbridge\UAssetBridge.exe    - Texture pipeline ^(uses modified UAssetAPI^)
echo.
echo TOOLS INCLUDED:
echo   Both .NET tools include the modified UAssetAPI library for:
echo   - Asset type detection ^(static mesh, skeletal mesh, texture^)
echo   - SerializeSize header fixing for static meshes
echo   - Mipmap/texture processing
echo.
echo INSTALLATION:
echo   1. Extract this folder anywhere ^(avoid Program Files^)
echo   2. Run repak-gui.exe
echo   3. Drag and drop .pak mods to install
echo.
echo REQUIREMENTS:
echo   - Windows x64
echo   - Marvel Rivals installed
echo.
echo For more info: https://github.com/natimerry/repak-rivals
echo ============================================================
) > "%DIST_DIR%\README.txt"

REM Copy licenses
echo   - Copying license files...
if exist "%~dp0LICENSE-MIT" copy "%~dp0LICENSE-MIT" "%DIST_DIR%\" >nul
if exist "%~dp0LICENSE-APACHE" copy "%~dp0LICENSE-APACHE" "%DIST_DIR%\" >nul

REM ============================================================
REM  Step 8: Create ZIP Archive
REM ============================================================
echo.
echo [8/8] Creating ZIP archive...

REM Try PowerShell Compress-Archive
powershell -NoProfile -Command "Compress-Archive -Path '%DIST_DIR%\*' -DestinationPath '%ZIP_FILE%' -Force" 2>nul
if %ERRORLEVEL% EQU 0 (
    echo   - Created: %ZIP_FILE%
) else (
    echo   - PowerShell compression failed, trying tar...
    pushd "%~dp0dist"
    tar -a -cf "%DIST_NAME%.zip" "%DIST_NAME%"
    popd
    if %ERRORLEVEL% EQU 0 (
        echo   - Created: %ZIP_FILE%
    ) else (
        echo   WARNING: Could not create ZIP. Please manually zip the folder:
        echo            %DIST_DIR%
    )
)

REM No temp cleanup needed - using default publish paths

REM ============================================================
REM  Summary
REM ============================================================
echo.
echo ============================================================
echo  BUILD COMPLETE!
echo ============================================================
echo.
echo Distribution folder: %DIST_DIR%
echo.
echo Contents:
dir /b "%DIST_DIR%"
echo.

if exist "%ZIP_FILE%" (
    echo ZIP archive: %ZIP_FILE%
    for %%A in ("%ZIP_FILE%") do echo Size: %%~zA bytes
)

echo.
echo Done! Press any key to exit.
pause >nul
