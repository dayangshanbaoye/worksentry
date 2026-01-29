@echo off
echo Starting WorkSentry...
cd /d C:\Users\zheng\worksentry\src-tauri\target\release
worksentry.exe
if errorlevel 1 (
    echo Error starting application
    pause
)
