@echo off
echo ========================================
echo P2P Local Testing Setup
echo ========================================
echo.
echo This will open TWO instances of the app for testing P2P transfer
echo.
echo Instance 1 (SENDER): Share your mods
echo Instance 2 (RECEIVER): Enter the connection string
echo.
echo Press any key to start...
pause > nul

echo.
echo Starting Instance 1 (SENDER)...
start "Repak GUI - SENDER" cmd /k "cd /d %~dp0repak-gui && cargo tauri dev"

echo Waiting 5 seconds before starting second instance...
timeout /t 5 /nobreak > nul

echo.
echo Starting Instance 2 (RECEIVER)...
start "Repak GUI - RECEIVER" cmd /k "cd /d %~dp0repak-gui && cargo tauri dev"

echo.
echo ========================================
echo Both instances are starting!
echo ========================================
echo.
echo INSTRUCTIONS:
echo.
echo 1. Wait for both windows to open
echo 2. In SENDER window: Click "Share Mods"
echo 3. Copy the connection string (long base64 text)
echo 4. In RECEIVER window: Paste and click "Receive"
echo 5. Watch the terminal logs for transfer progress
echo.
echo Press any key to close this window...
pause > nul
