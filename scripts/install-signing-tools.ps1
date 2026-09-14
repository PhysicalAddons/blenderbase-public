<#
Downloads the Azure Artifact Signing SignTool plugin (and, when no usable
SignTool is installed, the Windows SDK build tools) into .signing-tools/ at
the repository root. Nothing is installed system-wide.

  powershell -NoProfile -ExecutionPolicy Bypass -File scripts/install-signing-tools.ps1

Run once on a machine that will sign (CI does this on every release build).
The .NET 8 runtime must already be present; the plugin is a .NET 8 library.
#>
[CmdletBinding()]
param(
  # Download the Windows SDK build tools even if a Windows Kits SignTool exists.
  [switch]$IncludeSdk
)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$root = Split-Path $PSScriptRoot -Parent
$toolsDir = Join-Path $root '.signing-tools'
New-Item -ItemType Directory -Force -Path $toolsDir | Out-Null

$nuget = Join-Path $toolsDir 'nuget.exe'
if (-not (Test-Path $nuget)) {
  Write-Host "Downloading nuget.exe"
  Invoke-WebRequest -Uri 'https://dist.nuget.org/win-x86-commandline/latest/nuget.exe' -OutFile $nuget
}

function Install-Package([string]$id) {
  if (Get-ChildItem -Path $toolsDir -Directory -Filter "$id*" -ErrorAction SilentlyContinue) {
    Write-Host "$id already present"
    return
  }
  Write-Host "Installing $id"
  & $nuget install $id -ExcludeVersion -OutputDirectory $toolsDir -NonInteractive | Out-Null
  if ($LASTEXITCODE -ne 0) { throw "nuget install $id failed with exit code $LASTEXITCODE" }
}

Install-Package 'Microsoft.ArtifactSigning.Client'

$kitsSignTool = Get-ChildItem -Path "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.*\x64\signtool.exe" -ErrorAction SilentlyContinue |
  Where-Object { [version]$_.Directory.Parent.Name -ge [version]'10.0.22621.0' }
if ($IncludeSdk -or -not $kitsSignTool) {
  Install-Package 'Microsoft.Windows.SDK.BuildTools'
}

Write-Host "Signing tools ready in $toolsDir"
