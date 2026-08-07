# Point the per-user COM CLSID override back at the machine-default Hangul, so ordinary
# Hangul automation on this machine behaves as it did before a measurement run.
#
# IMPORTANT -- this script sets the override VALUE. It deliberately does NOT delete the key.
# Deleting HKCU\...\CLSID\{2291CF00-...} makes COM ignore the HKCU override for the rest of
# the login session: afterwards every activation resolves to the HKLM (machine default)
# binary no matter what value you write back, so version switching silently stops working
# until you log off and back on. Measured on Windows 11 with Hangul 2022 + 2024 installed.
#
# Guide: mydocs/manual/verification/hangul_version_oracle.md
[CmdletBinding()]
param(
  # Remove the key entirely instead of neutralising it. Only do this right before a logoff.
  [switch]$Purge
)

$ErrorActionPreference = 'Stop'
$CLSID = '{2291CF00-64A1-4877-A9B4-68CFE89612D6}'

$machine = (Get-ItemProperty "HKLM:\SOFTWARE\Classes\WOW6432Node\CLSID\$CLSID\LocalServer32" -ErrorAction SilentlyContinue).'(default)'
if (-not $machine) {
  $machine = (Get-ItemProperty "HKLM:\SOFTWARE\Classes\CLSID\$CLSID\LocalServer32" -ErrorAction SilentlyContinue).'(default)'
}
if (-not $machine) { throw "machine default registration not found -- is Hangul installed?" }
Write-Output "machine default: $machine"

foreach ($base in "HKCU:\Software\Classes\CLSID\$CLSID", "HKCU:\Software\Classes\Wow6432Node\CLSID\$CLSID") {
  if ($Purge) {
    if (Test-Path $base) { Remove-Item -Path $base -Recurse -Force; Write-Output "purged: $base" }
    continue
  }
  $ls = Join-Path $base 'LocalServer32'
  New-Item -Path $ls -Force | Out-Null
  Set-ItemProperty -Path $ls -Name '(default)' -Value $machine
  Write-Output "set to machine default: $ls"
}
if ($Purge) {
  Write-Warning "Purged the override key. Version switching will not work again until you log off and back on."
}

Get-Process Hwp -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

try {
  $h = New-Object -ComObject HWPFrame.HwpObject
  Write-Output ("COM now resolves to: " + $h.Version)
  try { $h.Quit() } catch { }
  [System.Runtime.InteropServices.Marshal]::ReleaseComObject($h) | Out-Null
  Get-Process Hwp -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
} catch {
  Write-Output ("COM activation failed: " + $_.Exception.Message)
}
