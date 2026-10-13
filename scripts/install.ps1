<#
File: install.ps1
Author: Michal Švrček
LogLens Windows installer.
ver. 0.6.0
#>

param(
    [string]$BinaryPath
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

if (-not $BinaryPath) {
    $ReleaseBinary = Join-Path $ScriptDir "lens.exe"
    $BuildBinary = Join-Path $ProjectRoot "target\release\lens.exe"

    if (Test-Path $ReleaseBinary) {
        $BinaryPath = $ReleaseBinary
    }
    else {
        $BinaryPath = $BuildBinary
    }
}

if (-not (Test-Path -LiteralPath $BinaryPath -PathType Leaf)) {
    throw "lens.exe not found: $BinaryPath"
}

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\LogLens"
$Destination = Join-Path $InstallDir "lens.exe"

Write-Host "LogLens Installer" -ForegroundColor Cyan

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

$Source = (Resolve-Path -LiteralPath $BinaryPath).Path

if ($Source -ine $Destination) {
    Copy-Item -LiteralPath $Source -Destination $Destination -Force
}

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
$Entries = @($UserPath -split ';' | Where-Object { $_.Trim() -ne "" })

if (-not ($Entries | Where-Object {
    $_.TrimEnd('\') -ieq $InstallDir.TrimEnd('\')
})) {
    $Entries += $InstallDir

    [Environment]::SetEnvironmentVariable(
        "Path",
        ($Entries -join ';'),
        "User"
    )
}

Write-Host "[ OK ] Installed: $Destination" -ForegroundColor Green
& $Destination --version

Write-Host "Open a new terminal and run: lens --help"
