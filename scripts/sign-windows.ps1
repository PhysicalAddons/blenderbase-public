<#
Signs Windows binaries with Azure Artifact Signing (SignTool + the Microsoft
plugin). Tauri calls this for the exe and the MSI through
bundle.windows.signCommand; the overlay config that wires it up is written by
scripts/sign-config.ps1. It can also be run by hand:

  powershell -NoProfile -ExecutionPolicy Bypass -File scripts/sign-windows.ps1 path\to\file.exe

Credentials are resolved by DefaultAzureCredential, so `az login` (locally) or
azure/login with OIDC (GitHub Actions) is enough; no secret is read here. The
signing identity needs the "Artifact Signing Certificate Profile Signer" role
on the account named in scripts/artifact-signing.json.

Overrides: SIGNTOOL_PATH, ARTIFACT_SIGNING_DLIB, ARTIFACT_SIGNING_METADATA.
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory, ValueFromRemainingArguments)]
  [string[]]$Files
)
$ErrorActionPreference = 'Stop'

$root = Split-Path $PSScriptRoot -Parent
$toolsDir = Join-Path $root '.signing-tools'
$minSignTool = [version]'10.0.22621.0'

function Find-SignTool {
  if ($env:SIGNTOOL_PATH) { return $env:SIGNTOOL_PATH }
  $candidates = @(
    Get-ChildItem -Path (Join-Path $toolsDir 'Microsoft.Windows.SDK.BuildTools\bin\10.*\x64\signtool.exe') -ErrorAction SilentlyContinue
    Get-ChildItem -Path "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.*\x64\signtool.exe" -ErrorAction SilentlyContinue
  ) | Where-Object { $_ -and [version]$_.Directory.Parent.Name -ge $minSignTool }
  $best = $candidates | Sort-Object { [version]$_.Directory.Parent.Name } -Descending | Select-Object -First 1
  if (-not $best) {
    throw "No SignTool >= $minSignTool found. Run scripts/install-signing-tools.ps1 -IncludeSdk or set SIGNTOOL_PATH."
  }
  return $best.FullName
}

function Find-Dlib {
  if ($env:ARTIFACT_SIGNING_DLIB) { return $env:ARTIFACT_SIGNING_DLIB }
  $dlib = Get-ChildItem -Path (Join-Path $toolsDir 'Microsoft.ArtifactSigning.Client\bin\x64\*.Dlib.dll') -ErrorAction SilentlyContinue |
    Select-Object -First 1
  if (-not $dlib) {
    throw "Artifact Signing plugin not found under $toolsDir. Run scripts/install-signing-tools.ps1 or set ARTIFACT_SIGNING_DLIB."
  }
  return $dlib.FullName
}

$signTool = Find-SignTool
$dlib = Find-Dlib
$metadata = if ($env:ARTIFACT_SIGNING_METADATA) { $env:ARTIFACT_SIGNING_METADATA } else { Join-Path $PSScriptRoot 'artifact-signing.json' }
if (-not (Test-Path $metadata)) { throw "Metadata file not found: $metadata" }

Write-Host "SignTool: $signTool"
Write-Host "Plugin:   $dlib"
Write-Host "Metadata: $metadata"

foreach ($file in $Files) {
  if (-not (Test-Path $file)) { throw "File to sign not found: $file" }
  Write-Host "Signing $file"
  # signtool writes progress to stderr. When the caller pipes stderr (Tauri does),
  # Windows PowerShell 5.1 would turn those lines into terminating errors under
  # 'Stop', so the native calls run under 'Continue' and are judged by exit code.
  $ErrorActionPreference = 'Continue'
  # Certificates from the service live three days; the timestamp is what keeps
  # the signature valid after that.
  & $signTool sign /v /fd SHA256 /tr 'http://timestamp.acs.microsoft.com' /td SHA256 /dlib $dlib /dmdf $metadata $file 2>&1 | ForEach-Object { "$_" }
  if ($LASTEXITCODE -ne 0) { $ErrorActionPreference = 'Stop'; throw "signtool sign failed for $file with exit code $LASTEXITCODE" }
  & $signTool verify /pa /v $file 2>&1 | ForEach-Object { "$_" }
  if ($LASTEXITCODE -ne 0) { $ErrorActionPreference = 'Stop'; throw "signtool verify failed for $file with exit code $LASTEXITCODE" }
  $ErrorActionPreference = 'Stop'
}
