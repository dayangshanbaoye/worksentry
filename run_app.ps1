$proc = Start-Process -FilePath "C:\Users\zheng\worksentry\src-tauri\target\release\worksentry.exe" -PassThru -WindowStyle Normal
Start-Sleep -Seconds 3
Get-Process -Id $proc.Id | Select-Object Id, ProcessName, MainWindowHandle, MainWindowTitle, HasExited
