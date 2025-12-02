@echo off
echo ========================================
echo Starting TWO instances for P2P testing
echo ========================================
echo.

REM Start first instance (SENDER)
echo Starting SENDER instance...
start "Repak GUI - SENDER" "%~dp0target\release\repak-gui.exe"

REM Wait 3 seconds for first instance to fully start
echo Waiting for first instance to start...
timeout /t 3 /nobreak > nul

REM Start second instance (RECEIVER)
echo Starting RECEIVER instance...
start "Repak GUI - RECEIVER" "%~dp0target\release\repak-gui.exe"

echo.
echo ========================================
echo Both instances are now running!
echo ========================================
echo.
echo INSTRUCTIONS:
echo.
echo 1. In SENDER window: Click "Share Mods"
echo 2. Copy the connection string (long base64 text)
echo 3. In RECEIVER window: Paste and click "Receive"
echo.
echo Logs are at: target\release\Logs\repak-gui.log
echo.
echo Press any key to close this window...
pause > nul
