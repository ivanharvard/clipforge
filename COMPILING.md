# Building ClipForge from Source on Windows

This guide explains how to compile `clipforge-app` on 64-bit Windows and package it as an MSI installer.

> These instructions use the Rust **MSVC** toolchain. WSL builds produce Linux binaries and cannot be used directly in the Windows MSI.

## Requirements

Install the following before building:

- Git
- Rust with the `x86_64-pc-windows-msvc` toolchain
- Visual Studio 2022 Build Tools with the **Desktop development with C++** workload
- A Windows SDK
- .NET SDK 8 or newer
- WiX Toolset
- A 64-bit libmpv development build

## 1. Clone the repository

Open PowerShell:

```powershell
git clone https://github.com/ivanharvard/clipforge
cd clipforge
```

## 2. Install Rust

Install Rustup:

```powershell
winget install --exact --id Rustlang.Rustup
```

Close all PowerShell and Windows Terminal windows, then open a new PowerShell window.

Verify the installation:

```powershell
rustup --version; cargo --version; rustc --version
```

Use the stable MSVC toolchain:

```powershell
rustup default stable-msvc
```

If `cargo` exists in `%USERPROFILE%\.cargo\bin` but is not recognized, permanently add that directory to the user PATH:

```powershell
$c="$env:USERPROFILE\.cargo\bin";$p=[Environment]::GetEnvironmentVariable("Path","User");if(($p-split';')-notcontains$c){[Environment]::SetEnvironmentVariable("Path",($c+';'+$p),"User")}
```

Close every PowerShell and Windows Terminal process and reopen PowerShell. If necessary, sign out of Windows and sign back in.

Verify:

```powershell
where.exe cargo; cargo --version
```

## 3. Install the Microsoft C++ build tools

Open **Visual Studio Installer** and install or modify **Build Tools 2022**.

Select the workload:

- **Desktop development with C++**

Make sure it includes:

- MSVC x64/x86 build tools
- Windows 10 or Windows 11 SDK
- C++ CMake tools for Windows

After installation, open **Developer PowerShell for VS 2022** or **x64 Native Tools Command Prompt for VS 2022**.

Verify that the linker is available:

```powershell
where.exe link
```

If `link.exe` is not found, the shell has not loaded the Visual Studio build environment or the C++ workload is missing.

## 4. Install the .NET SDK and WiX

Install the .NET SDK:

```powershell
winget install --exact --id Microsoft.DotNet.SDK.8
```

Close and reopen PowerShell, then verify:

```powershell
dotnet --version
```

Install WiX globally:

```powershell
dotnet tool install --global wix
```

Permanently add the .NET global-tools directory to the user PATH:

```powershell
$d="$env:USERPROFILE\.dotnet\tools";$p=[Environment]::GetEnvironmentVariable("Path","User");if(($p-split';')-notcontains$d){[Environment]::SetEnvironmentVariable("Path",($d+';'+$p),"User")}
```

Close all PowerShell and Windows Terminal windows, reopen Developer PowerShell, and verify:

```powershell
wix --version
```

WiX v7 requires acceptance of its OSMF EULA before use:

```powershell
wix eula accept wix7
```

Review the current WiX licensing terms before accepting them.

## 5. Install libmpv

ClipForge links against libmpv. A normal mpv player installation is not sufficient; the build needs:

- `libmpv-2.dll`
- an MSVC-compatible import library named `mpv.lib`

Download a current 64-bit `mpv-dev-x86_64` archive from a trusted Windows mpv build provider and extract it to:

```text
C:\libmpv
```

The extracted directory may contain:

```text
C:\libmpv\libmpv-2.dll
C:\libmpv\libmpv.dll.a
```

`libmpv.dll.a` is a MinGW import library. The MSVC Rust toolchain requires `mpv.lib`.

### Generate `mpv.lib`

Run the following commands from **Developer PowerShell for VS 2022**.

Export the DLL symbols:

```powershell
dumpbin /exports C:\libmpv\libmpv-2.dll | Select-String '^\s+[0-9A-F]+\s+[0-9A-F]+\s+[0-9A-F]+\s+(\S+)$' | ForEach-Object { $_.Matches[0].Groups[1].Value } | Set-Content C:\libmpv\exports.txt
```

Create the module-definition file:

```powershell
"LIBRARY libmpv-2.dll`r`nEXPORTS" | Set-Content C:\libmpv\mpv.def; Get-Content C:\libmpv\exports.txt | Add-Content C:\libmpv\mpv.def
```

Generate the MSVC import library:

```powershell
lib.exe /def:C:\libmpv\mpv.def /machine:x64 /out:C:\libmpv\mpv.lib
```

Verify it:

```powershell
Test-Path C:\libmpv\mpv.lib
```

The command should print `True`.

## 6. Build ClipForge

From the repository root in Developer PowerShell, expose the libmpv import-library directory:

```powershell
$env:LIB="C:\libmpv;$env:LIB"
```

Build the release executable:

```powershell
cargo build --release --bin clipforge-app
```

The Windows executable should be created at:

```text
target\release\clipforge-app.exe
```

Verify it:

```powershell
Test-Path .\target\release\clipforge-app.exe
```

## 7. Copy the libmpv runtime DLL

The application requires `libmpv-2.dll` at runtime. Copy it beside the executable:

```powershell
Copy-Item C:\libmpv\libmpv-2.dll .\target\release\ -Force
```

Verify both files exist:

```powershell
Get-ChildItem .\target\release\clipforge-app.exe,.\target\release\libmpv-2.dll
```

Test the application before building the MSI:

```powershell
.\target\release\clipforge-app.exe
```

If the application opens and immediately closes, first confirm that `libmpv-2.dll` is next to the executable.

## 8. Configure the WiX source

The WiX definition must reference paths relative to the directory where `wix build` is executed.

Because `scripts\build-msi.ps1` changes to the repository root, the executable source should use:

```xml
<File Source="target\release\clipforge-app.exe" />
```

It should not use:

```xml
<File Source="..\..\target\release\clipforge-app.exe" />
```

The MSI must also include the libmpv runtime DLL beside the application:

```xml
<File Source="target\release\libmpv-2.dll" />
```

A simplified component may look like this:

```xml
<Component>
  <File Source="target\release\clipforge-app.exe" />
  <File Source="target\release\libmpv-2.dll" />
</Component>
```

Adapt the component IDs, directory structure, shortcuts, and feature references to the existing `packaging\windows\clipforge.wxs` file.

## 9. Improve the MSI build script

Use a build script similar to the following:

```powershell
# Builds a release binary and packages it as an MSI using WiX.
$ErrorActionPreference = "Stop"

Set-Location "$PSScriptRoot\.."

$env:LIB = "C:\libmpv;$env:LIB"

cargo build --release --bin clipforge-app
if ($LASTEXITCODE -ne 0) { throw "Cargo build failed with exit code $LASTEXITCODE" }

Copy-Item C:\libmpv\libmpv-2.dll .\target\release\ -Force

New-Item -ItemType Directory -Force -Path dist | Out-Null

wix build packaging/windows/clipforge.wxs -out dist/ClipForge.msi
if ($LASTEXITCODE -ne 0) { throw "WiX build failed with exit code $LASTEXITCODE" }
```

The exit-code checks prevent WiX from packaging a missing or stale executable after a failed Cargo build.

## 10. Build the MSI

From the repository root in Developer PowerShell:

```powershell
.\scripts\build-msi.ps1
```

The installer will be created at:

```text
dist\ClipForge.msi
```

Open the output folder:

```powershell
explorer .\dist
```

## 11. Test the installer

Before testing a rebuilt MSI, uninstall any earlier ClipForge installation or ensure the WiX package is configured for upgrades.

Double-click:

```text
dist\ClipForge.msi
```

After installation:

1. Launch ClipForge from its installed location or Start menu shortcut.
2. Confirm that `clipforge-app.exe` and `libmpv-2.dll` were installed in the same directory.
3. Test video and audio playback.
4. Test uninstalling ClipForge through Windows Settings.

## Troubleshooting

### `cargo`, `rustc`, or `rustup` is not recognized

Confirm the executable exists:

```powershell
Test-Path "$env:USERPROFILE\.cargo\bin\cargo.exe"
```

Check the persistent user PATH:

```powershell
[Environment]::GetEnvironmentVariable("Path","User")
```

Close all Windows Terminal processes after modifying PATH. New tabs opened by an existing Terminal process may inherit the old environment.

### `link.exe` not found

Install the **Desktop development with C++** workload through Visual Studio Installer and build from Developer PowerShell for VS 2022.

Verify:

```powershell
where.exe link
```

### `LNK1181: cannot open input file 'mpv.lib'`

Confirm that the import library exists:

```powershell
Test-Path C:\libmpv\mpv.lib
```

Expose its directory before building:

```powershell
$env:LIB="C:\libmpv;$env:LIB"
```

### `WIX0103: Cannot find the File file ...clipforge-app.exe`

Verify the executable exists:

```powershell
Test-Path .\target\release\clipforge-app.exe
```

Ensure the WiX source uses:

```xml
Source="target\release\clipforge-app.exe"
```

when WiX is launched from the repository root.

### The installed application closes immediately

Confirm that `libmpv-2.dll` is installed beside `clipforge-app.exe`.

The MSI must contain both files:

```text
clipforge-app.exe
libmpv-2.dll
```

### WSL notes

Do not package a WSL build into the Windows MSI. WSL produces a Linux executable such as:

```text
target/release/clipforge-app
```

The MSI requires the Windows executable:

```text
target\release\clipforge-app.exe
```

Build and package the Windows version from Windows Developer PowerShell using the MSVC toolchain.
