<#
Writes src-tauri/tauri.sign.conf.json, a Tauri config overlay that makes
`tauri build` sign the Windows exe and MSI through scripts/sign-windows.ps1,
and prints the `--config` argument to pass. The overlay is generated (and
git-ignored) because Tauri needs an absolute path to the script and the
working directory during bundling is not fixed.

  $cfg = powershell -NoProfile -ExecutionPolicy Bypass -File scripts/sign-config.ps1
  npm run tauri -- build --ci --config $cfg

Plain `tauri build` without the overlay stays unsigned, which is what local
test builds want.
#>
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$script = Join-Path $PSScriptRoot 'sign-windows.ps1'
$out = Join-Path $root 'src-tauri\tauri.sign.conf.json'

$config = @{
  bundle = @{
    windows = @{
      signCommand = @{
        cmd  = 'powershell'
        args = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', $script, '%1')
      }
    }
  }
}
# Tauri's JSON parser rejects a byte-order mark, which Set-Content -Encoding utf8 writes on PowerShell 5.1.
$json = $config | ConvertTo-Json -Depth 6
[System.IO.File]::WriteAllText($out, $json, (New-Object System.Text.UTF8Encoding $false))
Write-Output $out
