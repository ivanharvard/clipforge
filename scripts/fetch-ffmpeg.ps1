# Downloads the static LGPL FFmpeg Windows tools used by the MSI build.
$ErrorActionPreference = "Stop"

Set-Location "$PSScriptRoot\.."

$downloadUrl = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-lgpl.zip"
$archivePath = "target\ffmpeg-master-latest-win64-lgpl.zip"
$extractPath = "target\ffmpeg-package"
$binPath = "target\ffmpeg\bin"

if ((Test-Path "$binPath\ffmpeg.exe" -PathType Leaf) -and
    (Test-Path "$binPath\ffprobe.exe" -PathType Leaf) -and
    (Test-Path "$binPath\FFmpeg-LICENSE.txt" -PathType Leaf)) {
    return
}

New-Item -ItemType Directory -Force -Path target,$extractPath,$binPath | Out-Null

if (-not (Test-Path $archivePath -PathType Leaf)) {
    Write-Host "Downloading FFmpeg LGPL tools..."
    Invoke-WebRequest -UseBasicParsing $downloadUrl -OutFile $archivePath
}
Expand-Archive -LiteralPath $archivePath -DestinationPath $extractPath -Force

$ffmpeg = Get-ChildItem -LiteralPath $extractPath -Filter ffmpeg.exe -Recurse | Select-Object -First 1
if ($null -eq $ffmpeg) {
    throw "Downloaded FFmpeg archive did not contain ffmpeg.exe"
}
$ffprobe = Join-Path $ffmpeg.DirectoryName "ffprobe.exe"
if (-not (Test-Path $ffprobe -PathType Leaf)) {
    throw "Downloaded FFmpeg archive did not contain ffprobe.exe"
}
$license = Get-ChildItem -LiteralPath $extractPath -Filter LICENSE.txt -Recurse | Select-Object -First 1
if ($null -eq $license) {
    throw "Downloaded FFmpeg archive did not contain LICENSE.txt"
}

Copy-Item -LiteralPath $ffmpeg.FullName -Destination "$binPath\ffmpeg.exe" -Force
Copy-Item -LiteralPath $ffprobe -Destination "$binPath\ffprobe.exe" -Force
Copy-Item -LiteralPath $license.FullName -Destination "$binPath\FFmpeg-LICENSE.txt" -Force
