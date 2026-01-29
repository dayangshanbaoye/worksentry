Add-Type -AssemblyName System.Drawing
$b = New-Object System.Drawing.Bitmap(256, 256)
$g = [System.Drawing.Graphics]::FromImage($b)
$g.Clear([System.Drawing.Color]::FromArgb(26, 26, 46))
$brush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(233, 69, 96))
$g.FillEllipse($brush, 10, 10, 236, 236)
$pen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(233, 69, 96), 8)
$g.DrawEllipse($pen, 10, 10, 236, 236)
$g.DrawLine($pen, 180, 180, 76, 76)
$b.Save("C:/Users/zheng/worksentry/src-tauri/icons/icon.ico", [System.Drawing.Imaging.ImageFormat]::Icon)
$b.Dispose()
Write-Host "Icon created successfully"
