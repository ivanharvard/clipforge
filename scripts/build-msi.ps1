# Builds a release binary and packages it as an MSI using the WiX v4
# toolset and the definition in packaging/windows/clipforge.wxs.
$ErrorActionPreference = "Stop"

Set-Location "$PSScriptRoot\.."

cargo build --release --bin clipforge-app
if ($LASTEXITCODE -ne 0) {
    throw "Cargo build failed with exit code $LASTEXITCODE"
}

$libMpvDll = "C:\libmpv\libmpv-2.dll"
if (-not (Test-Path $libMpvDll -PathType Leaf)) {
    throw "libmpv runtime DLL not found at $libMpvDll"
}

Copy-Item $libMpvDll target\release\libmpv-2.dll -Force

New-Item -ItemType Directory -Force -Path dist | Out-Null
wix build -arch x64 packaging/windows/clipforge.wxs -out dist/ClipForge.msi
if ($LASTEXITCODE -ne 0) {
    throw "WiX build failed with exit code $LASTEXITCODE"
}
