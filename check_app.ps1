Get-Process worksentry -ErrorAction SilentlyContinue | Select-Object Id, ProcessName, MainWindowHandle, MainWindowTitle, StartTime
