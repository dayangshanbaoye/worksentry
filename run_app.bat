@echo off
cd /d C:\Users\zheng\worksentry\src-tauri\target\release
worksentry.exe 2> error.log
echo Exit code: %ERRORLEVEL%
pause
