@echo off
echo ========================================
echo Building Repak GUI for P2P Testing
echo ========================================
echo.
echo This will:
echo 1. Build the release version
echo 2. Open TWO instances for testing
echo.
echo Building... (this may take a few minutes)
echo.

cd repak-gui
cargo build --release

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo ========================================
    echo BUILD FAILED!
    echo ========================================
    pause
    exit /b 1
)

echo.
echo ========================================
echo BUILD SUCCESS!
echo ========================================
echo.
echo Starting two instances for testing...
echo.

REM Start first instance (SENDER)
start "Repak GUI - SENDER" "%~dp0repak-gui\target\release\repak-gui.exe"

REM Wait a moment
timeout /t 2 /nobreak > nul

REM Start second instance (RECEIVER)
start "Repak GUI - RECEIVER" "%~dp0repak-gui\target\release\repak-gui.exe"

echo.
echo ========================================
echo Both instances are running!
echo ========================================
echo.
echo INSTRUCTIONS:
echo.
echo 1. In SENDER window: Click "Share Mods"
echo 2. Copy the connection string
echo 3. In RECEIVER window: Paste and click "Receive"
echo.
echo Check the log files for transfer progress:
echo %~dp0repak-gui\target\release\Logs\repak-gui.log
echo.
pause
