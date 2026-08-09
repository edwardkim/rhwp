# List the Hangul releases installed on this machine and show which one COM currently resolves to.
# Run this first: it tells you which -HwpVersion values page_oracle_run.ps1 will accept.
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$CLSID = '{2291CF00-64A1-4877-A9B4-68CFE89612D6}'

Write-Output '=== installed Hangul releases ==='
$found = @()
foreach ($r in @(${env:ProgramFiles(x86)}, $env:ProgramFiles) | Where-Object { $_ }) {
  $hnc = Join-Path $r 'Hnc'
  if (-not (Test-Path -LiteralPath $hnc)) { continue }
  foreach ($office in (Get-ChildItem $hnc -Directory -ErrorAction SilentlyContinue)) {
    $exe = Get-Item -Path (Join-Path $office.FullName 'HOffice*\Bin\Hwp.exe') -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($exe) {
      $pv = $exe.VersionInfo.ProductVersion
      $major = ($pv -split '[.,]')[0].Trim()
      $year = ($office.Name -replace '^\s*Office\s*', '')
      $found += [pscustomobject]@{ Version = $year; Major = $major; ProductVersion = $pv; Path = $exe.FullName }
    }
  }
}
if ($found.Count -eq 0) { Write-Output '  (none found under Hnc)' }
$found | Format-Table -AutoSize

Write-Output '=== COM registration ==='
# Both registry views matter. A 32-bit caller reads Wow6432Node, a 64-bit caller reads the
# plain CLSID path, and the two can hold DIFFERENT values -- reading only one view reports
# "no override" while an override is in fact in effect and picking the version.
function Get-LocalServer32([string]$hive) {
  $r = [ordered]@{}
  foreach ($view in @('CLSID', 'Wow6432Node\CLSID')) {
    $r[$view] = (Get-ItemProperty "${hive}:\Software\Classes\$view\$CLSID\LocalServer32" -ErrorAction SilentlyContinue).'(default)'
  }
  return $r
}
$hklm = Get-LocalServer32 'HKLM'
$hkcu = Get-LocalServer32 'HKCU'
foreach ($view in $hklm.Keys) {
  $v = $hklm[$view]
  Write-Output ("  HKLM {0,-18} {1}" -f $view, $(if ($v) { $v } else { '(not set)' }))
}
foreach ($view in $hkcu.Keys) {
  $v = $hkcu[$view]
  Write-Output ("  HKCU {0,-18} {1}" -f $view, $(if ($v) { $v } else { '(not set)' }))
}
$overrides = @($hkcu.Values | Where-Object { $_ })
if ($overrides.Count -eq 0) {
  Write-Output '  -> no HKCU override; the machine default applies'
} else {
  Write-Output '  -> HKCU override is in effect and WINS over the machine default'
  if (@($overrides | Sort-Object -Unique).Count -gt 1) {
    Write-Warning 'The two HKCU views disagree. Run restore_com_default.ps1, or start a pass (it sets both).'
  }
}

Write-Output '=== what COM actually hands out right now ==='
# The probe below starts an instance and then kills every Hwp.exe. Doing that while a pass is
# running would both add a CONCURRENT instance (the one thing that corrupts the measurement)
# and terminate the worker's instance. If Hangul is already up, report and skip.
$live = @(Get-Process Hwp -ErrorAction SilentlyContinue)
if ($live.Count -gt 0) {
  Write-Output "  skipped: $($live.Count) Hwp.exe already running (a pass may be in progress)."
  Write-Output '  probing would run a second instance and then kill them all. Close Hangul and rerun.'
  foreach ($p in $live) { Write-Output ("  running exe = " + $p.Path) }
  return
}
try {
  $h = New-Object -ComObject HWPFrame.HwpObject
  Write-Output ("  hwp.Version = " + $h.Version)
  $proc = Get-Process Hwp -ErrorAction SilentlyContinue | Select-Object -First 1
  if ($proc) { Write-Output ("  running exe = " + $proc.Path) }
  try { $h.Quit() } catch { }
  [System.Runtime.InteropServices.Marshal]::ReleaseComObject($h) | Out-Null
  Get-Process Hwp -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
} catch {
  Write-Output ("  COM activation failed: " + $_.Exception.Message)
}
