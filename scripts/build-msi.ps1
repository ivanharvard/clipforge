# Builds a release binary and packages it as an MSI using the WiX v4
# toolset and the definition in packaging/windows/clipforge.wxs.
$ErrorActionPreference = "Stop"

Set-Location "$PSScriptRoot\.."

cargo build --release --bin clipforge-app

New-Item -ItemType Directory -Force -Path dist | Out-Null
wix build packaging/windows/clipforge.wxs -out dist/ClipForge.msi
