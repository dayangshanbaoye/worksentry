$ErrorActionPreference = "Continue"
try {
    $webview = New-Object -ComObject WebView2.WebView2 -ErrorAction Stop
    Write-Host "WebView2 COM object created successfully"
    Write-Host "WebView2 is available"
} catch {
    Write-Host "WebView2 COM object failed: $_"
    Write-Host "Trying alternative check..."

    $path = "C:\Program Files (x86)\Microsoft\EdgeWebView\Application\144.0.3719.93\WebView2Loader.dll"
    if (Test-Path $path) {
        Write-Host "WebView2 Runtime DLL found at: $path"
    } else {
        Write-Host "WebView2 Runtime DLL NOT found"
    }
}

Get-Process worksentry -ErrorAction SilentlyContinue | Select-Object Id, ProcessName, HasExited, Responding
