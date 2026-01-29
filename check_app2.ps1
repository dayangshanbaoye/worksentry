$processes = Get-Process worksentry -ErrorAction SilentlyContinue
if ($processes) {
    Write-Host "Found $($processes.Count) worksentry process(es)"
    $processes | Select-Object Id, ProcessName, MainWindowHandle, MainWindowTitle, StartTime | Format-List
} else {
    Write-Host "No worksentry processes found"
}
