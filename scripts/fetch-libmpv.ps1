# Downloads a prebuilt libmpv-dev archive and converts its MinGW import
# library into an MSVC-compatible mpv.lib.
#
# libmpv is NOT a vcpkg port — despite an old open request (microsoft/vcpkg
# #9511), it was never added, and `vcpkg install mpv:x64-windows` simply
# fails with "port does not exist". This automates the same manual process
# documented in COMPILING.md (dumpbin -> .def -> lib.exe), which is the
# actual working path.
#
# Requires dumpbin.exe and lib.exe on PATH — run this from a Developer
# PowerShell / after `ilammy/msvc-dev-cmd` in CI.
$ErrorActionPreference = "Stop"

Set-Location "$PSScriptRoot\.."

$libMpvDir = "C:\libmpv"

if ((Test-Path "$libMpvDir\libmpv-2.dll" -PathType Leaf) -and
    (Test-Path "$libMpvDir\mpv.lib" -PathType Leaf)) {
    return
}

New-Item -ItemType Directory -Force -Path target, $libMpvDir | Out-Null

Write-Host "Looking up the latest libmpv-dev-x86_64 build..."
$feedUrl = "https://sourceforge.net/projects/mpv-player-windows/rss?path=/libmpv"
$userAgent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36"
try {
    # Invoke-RestMethod parses the response as XML itself based on the
    # returned content-type, instead of a manual [xml] cast on
    # Invoke-WebRequest's raw .Content — sidesteps encoding/decompression
    # edge cases that can otherwise leave the cast silently empty.
    $feed = Invoke-RestMethod -UseBasicParsing -UserAgent $userAgent -Uri $feedUrl
} catch {
    throw "Fetching the mpv-player-windows RSS feed failed: $_"
}

$allItems = @($feed.rss.channel.item)
Write-Host "Feed returned $($allItems.Count) item(s)."

# The feed also lists "-v3-" (requires AVX2-era CPUs) and "i686" (32-bit)
# builds newest-first; this pattern picks the plain x86_64 baseline build so
# the resulting MSI doesn't silently fail to launch on older CPUs.
$item = $allItems |
    Where-Object { $_.title -match '/libmpv/mpv-dev-x86_64-\d{8}-git-[0-9a-f]+\.7z$' } |
    Select-Object -First 1
if ($null -eq $item) {
    Write-Host "First 10 item titles seen:"
    $allItems | Select-Object -First 10 -ExpandProperty title | ForEach-Object { Write-Host "  $_" }
    throw "Could not find a libmpv-dev-x86_64 build in the mpv-player-windows RSS feed (see titles logged above)"
}

$fileName = Split-Path -Leaf $item.title
$archivePath = "target\$fileName"
if (-not (Test-Path $archivePath -PathType Leaf)) {
    Write-Host "Downloading $($item.link)"
    Invoke-WebRequest -UseBasicParsing -UserAgent $userAgent -Uri $item.link -OutFile $archivePath
}
if (-not (Test-Path $archivePath -PathType Leaf) -or (Get-Item $archivePath).Length -eq 0) {
    throw "Download of $fileName produced no file or an empty file"
}

$extractPath = "target\libmpv-package"
Remove-Item -Recurse -Force $extractPath -ErrorAction SilentlyContinue
$7z = "${env:ProgramFiles}\7-Zip\7z.exe"
if (-not (Test-Path $7z -PathType Leaf)) { $7z = "7z" } # fall back to PATH if not the hosted-runner install
& $7z x $archivePath "-o$extractPath" -y | Out-Null
if ($LASTEXITCODE -ne 0) { throw "Extracting the libmpv archive failed with exit code $LASTEXITCODE" }

$dll = Get-ChildItem -LiteralPath $extractPath -Filter "libmpv-2.dll" -Recurse | Select-Object -First 1
if ($null -eq $dll) { throw "Downloaded libmpv archive did not contain libmpv-2.dll" }
Copy-Item -LiteralPath $dll.FullName -Destination "$libMpvDir\libmpv-2.dll" -Force

# The archive ships libmpv.dll.a, a MinGW import library the MSVC linker
# can't use. Rebuild a real one from the DLL's own export table instead.
$exports = & dumpbin /exports "$libMpvDir\libmpv-2.dll" |
    Select-String '^\s+[0-9A-F]+\s+[0-9A-F]+\s+[0-9A-F]+\s+(\S+)$' |
    ForEach-Object { $_.Matches[0].Groups[1].Value }
if (-not $exports) { throw "dumpbin found no exports in libmpv-2.dll" }

"LIBRARY libmpv-2.dll`r`nEXPORTS" | Set-Content "$libMpvDir\mpv.def"
$exports | Add-Content "$libMpvDir\mpv.def"

& lib.exe "/def:$libMpvDir\mpv.def" /machine:x64 "/out:$libMpvDir\mpv.lib" | Out-Null
if ($LASTEXITCODE -ne 0) { throw "Generating mpv.lib failed with exit code $LASTEXITCODE" }
if (-not (Test-Path "$libMpvDir\mpv.lib" -PathType Leaf)) { throw "mpv.lib was not created" }

Write-Host "libmpv ready at $libMpvDir (from $fileName)"
