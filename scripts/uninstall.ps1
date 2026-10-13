<#
File: uninstall.ps1
Author: Michal Švrček
LogLens Windows uninstaller.
ver. 0.6.0
#>

$ErrorActionPreference = "Stop"

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\LogLens"
$Binary = Join-Path $InstallDir "lens.exe"

if (Test-Path $Binary) {
    Remove-Item -LiteralPath $Binary -Force
}

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")

$Entries = @(
    $UserPath -split ';' | Where-Object {
        $_.Trim() -ne "" -and
        $_.TrimEnd('\') -ine $InstallDir.TrimEnd('\')
    }
)

[Environment]::SetEnvironmentVariable(
    "Path",
    ($Entries -join ';'),
    "User"
)

Write-Host "[ OK ] LogLens uninstalled." -ForegroundColor Green
Write-Host "Device profiles were preserved."
