@echo off
title Repak GUI - RECEIVER
echo ========================================
echo Starting RECEIVER Instance
echo ========================================
echo.
echo Waiting for sender to start first...
timeout /t 3 /nobreak
echo.
cd repak-gui
set VITE_PORT=5174
cargo tauri dev -- --port 5174
