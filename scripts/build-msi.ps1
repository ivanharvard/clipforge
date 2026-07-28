# Builds a release binary and packages it as an MSI using the WiX v4
# toolset and the definition in packaging/windows/clipforge.wxs.
$ErrorActionPreference = "Stop"

Set-Location "$PSScriptRoot\.."

$libMpvDir = "C:\libmpv"
$libMpvImportLibrary = Join-Path $libMpvDir "mpv.lib"
$libMpvDll = Join-Path $libMpvDir "libmpv-2.dll"

if (-not (Test-Path $libMpvImportLibrary -PathType Leaf)) {
    throw "libmpv import library not found at $libMpvImportLibrary"
}
if (-not (Test-Path $libMpvDll -PathType Leaf)) {
    throw "libmpv runtime DLL not found at $libMpvDll"
}

$env:LIB = if ($env:LIB) { "$libMpvDir;$env:LIB" } else { $libMpvDir }

$wixVersion = "4.0.6"
$wixDir = "target\wix\$wixVersion"
$wixExe = Join-Path $wixDir "wix.exe"
if (-not (Test-Path $wixExe -PathType Leaf)) {
    dotnet tool install wix --tool-path $wixDir --version $wixVersion
    if ($LASTEXITCODE -ne 0) {
        throw "WiX $wixVersion installation failed with exit code $LASTEXITCODE"
    }
}

$ffmpegDir = if ($env:FFMPEG_DIR) { $env:FFMPEG_DIR } else { "target\ffmpeg\bin" }
if (-not ((Test-Path "$ffmpegDir\ffmpeg.exe" -PathType Leaf) -and
          (Test-Path "$ffmpegDir\ffprobe.exe" -PathType Leaf) -and
          (Test-Path "$ffmpegDir\FFmpeg-LICENSE.txt" -PathType Leaf))) {
    & "$PSScriptRoot\fetch-ffmpeg.ps1"
    $ffmpegDir = "target\ffmpeg\bin"
}

cargo build --release --bin clipforge-app
if ($LASTEXITCODE -ne 0) {
    throw "Cargo build failed with exit code $LASTEXITCODE"
}

Copy-Item $libMpvDll target\release\libmpv-2.dll -Force
Copy-Item "$ffmpegDir\ffmpeg.exe" target\release\ffmpeg.exe -Force
Copy-Item "$ffmpegDir\ffprobe.exe" target\release\ffprobe.exe -Force
Copy-Item "$ffmpegDir\FFmpeg-LICENSE.txt" target\release\FFmpeg-LICENSE.txt -Force

New-Item -ItemType Directory -Force -Path dist | Out-Null
& $wixExe build -arch x64 packaging/windows/clipforge.wxs -out dist/ClipForge.msi
if ($LASTEXITCODE -ne 0) {
    throw "WiX build failed with exit code $LASTEXITCODE"
}
