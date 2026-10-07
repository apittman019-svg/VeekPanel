# VeekPanel - Windows Native Installer Build Script
# Run from PowerShell in the repository root or packaging/windows directory.
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = (Resolve-Path "$ScriptDir\..\..").Path

Write-Host "=== VeekPanel Windows Installer Build (Native Windows) ==="

# Check prerequisites
foreach ($cmd in @('pnpm', 'cargo')) {
    if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) {
        throw "Required tool '$cmd' is not installed or not in PATH."
    }
}

Set-Location $RepoRoot

# 1. Install frontend dependencies and build assets
Write-Host "--> Building UI frontend..."
pnpm --dir ui install --frozen-lockfile
pnpm --dir ui build

# 2. Build release application and package NSIS installer
Write-Host "--> Compiling Windows application and bundling NSIS installer..."
pnpm --dir ui exec tauri build

# 3. Verify output
$InstallerPath = Join-Path $RepoRoot "app\target\release\bundle\nsis\VeekPanel_0.1.0_x64-setup.exe"
if (Test-Path $InstallerPath) {
    $Item = Get-Item $InstallerPath
    $Hash = (Get-FileHash -Path $InstallerPath -Algorithm SHA256).Hash
    Write-Host ""
    Write-Host "=== Build Succeeded ==="
    Write-Host "Installer: $InstallerPath"
    Write-Host "Size: $([math]::Round($Item.Length / 1MB, 2)) MB ($($Item.Length) bytes)"
    Write-Host "SHA256: $Hash"
} else {
    throw "Expected installer was not found at $InstallerPath"
}
