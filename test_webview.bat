@echo off
echo Starting test...
powershell.exe -ExecutionPolicy Bypass -File "C:\Users\zheng\worksentry\test_webview.ps1" > "C:\Users\zheng\worksentry\webview_test_output.txt" 2>&1
echo Done. Check output file.
pause
