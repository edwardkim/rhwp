# Phase B supervisor: run one full load/save oracle pass under a chosen Hangul version.
#
# Direct adaptation of tools/hangul_version_oracle/page_oracle_run.ps1 (same CLSID override,
# override probe, heartbeat watch, stall-kill, worker restart). Differences: the worker is
# oracle_worker.ps1, the list is a key<TAB>path task file from rhwp_phase.py, and extracted
# body texts land in <OutDir>\texts for the judge.
#
# Run restore_com_default.ps1 (tools/hangul_version_oracle/) when all passes are done.
# Guide: mydocs/manual/verification/hangul_version_oracle.md
[CmdletBinding()]
param(
  # Hangul release year as installed under Hnc\Office <year>, e.g. 2018 / 2022 / 2024.
  [Parameter(Mandatory = $true)][string]$HwpVersion,
  [Parameter(Mandatory = $true)][string]$TaskPath,
  [Parameter(Mandatory = $true)][string]$OutDir,
  [int]$StallSeconds = 300,
  # [#4751] Large documents legitimately take minutes to open. The stall allowance grows
  # with file size: allowed = StallSeconds + StallSecondsPerMB * MB. A 11MB doc gets ~960s
  # (same order as the 1200s recheck that passed 18/18), a 4KB doc keeps the base 300s.
  [int]$StallSecondsPerMB = 60,
  [int]$RecycleEvery = 200,
  [int]$WarmupDocs = 5,
  # Hangul 2018 deadlocks in Open() when hidden -- only pass this on 2022+.
  [switch]$HideWindow
)

$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$CLSID = '{2291CF00-64A1-4877-A9B4-68CFE89612D6}'

function Resolve-HwpExe([string]$version) {
  $roots = @(${env:ProgramFiles(x86)}, $env:ProgramFiles) | Where-Object { $_ }
  foreach ($r in $roots) {
    $glob = Join-Path $r "Hnc\Office $version\HOffice*\Bin\Hwp.exe"
    $hit = Get-Item -Path $glob -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($hit) { return $hit.FullName }
  }
  throw "Hangul $version not found under Program Files\Hnc"
}

$exePath = Resolve-HwpExe $HwpVersion
$productVersion = (Get-Item -LiteralPath $exePath).VersionInfo.ProductVersion
$expectMajor = [int](($productVersion -split '[.,]')[0].Trim())
Write-Output "[sup] Hangul $HwpVersion -> $exePath (ProductVersion $productVersion, major $expectMajor)"

foreach ($base in "HKCU:\Software\Classes\CLSID\$CLSID\LocalServer32", "HKCU:\Software\Classes\Wow6432Node\CLSID\$CLSID\LocalServer32") {
  New-Item -Path $base -Force | Out-Null
  Set-ItemProperty -Path $base -Name '(default)' -Value "$exePath -Automation"
}
Write-Output "[sup] COM CLSID override set (HKCU)"

# A leftover Hwp.exe defeats the override: COM attaches to the already-running instance.
$stale = @(Get-Process Hwp -ErrorAction SilentlyContinue)
if ($stale.Count -gt 0) {
  Write-Output "[sup] terminating $($stale.Count) leftover Hwp.exe process(es)"
  $stale | Stop-Process -Force -ErrorAction SilentlyContinue
  Start-Sleep -Seconds 2
}

# Prove the override took effect BEFORE spending a pass on it.
Write-Output "[sup] verifying COM hands out major $expectMajor ..."
$probeMajor = 0
try {
  $probe = New-Object -ComObject HWPFrame.HwpObject
  $probeVersion = $probe.Version
  $probeMajor = [int](($probeVersion -split '[.,]')[0].Trim())
  try { $probe.Quit() } catch { }
  [System.Runtime.InteropServices.Marshal]::ReleaseComObject($probe) | Out-Null
} catch {
  throw "COM activation failed: $($_.Exception.Message)"
}
Get-Process Hwp -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
if ($probeMajor -ne $expectMajor) {
  Write-Output "[sup] got major $probeMajor ($probeVersion), expected $expectMajor"
  throw @"
COM did not honour the HKCU override.
  1. Close every Hwp.exe (a UI instance may have respawned).
  2. Never DELETE the HKCU CLSID key -- COM then ignores HKCU until logoff.
     Use tools/hangul_version_oracle/restore_com_default.ps1 to clean up after a run.
"@
}
Write-Output "[sup] verified: COM hands out major $probeMajor ($probeVersion)"

if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Force -Path $OutDir | Out-Null }
$all = Get-Content -LiteralPath $TaskPath -Encoding UTF8 | Where-Object { $_.Trim().Length -gt 0 }
Write-Output "[sup] tasks: $($all.Count) opens (single worker -- concurrent Hangul instances corrupt measurements)"

# [#4751] key -> file MB, for the size-scaled stall allowance. Missing files map to 0
# (base allowance). Built once up front; the loop below only does hashtable lookups.
$sizeMB = @{}
foreach ($taskLine in $all) {
  $c = $taskLine -split "`t"
  if ($c.Count -ge 2) {
    $mb = 0.0
    try { $mb = (Get-Item -LiteralPath $c[1] -ErrorAction Stop).Length / 1MB } catch { }
    $sizeMB[$c[0]] = $mb
  }
}
# [#4751] Every stall-kill is recorded here so judge.py can distinguish supervisor-made
# failures (ORACLE_TIMEOUT: re-measure) from genuine Hangul crashes (OPEN_FAIL).
$stallLog = Join-Path $OutDir 'stall_kills.tsv'

$state = @{
  Out = Join-Path $OutDir 'result.tsv'
  HB = Join-Path $OutDir 'hb.txt'
  Texts = Join-Path $OutDir 'texts'
  Total = $all.Count
  Proc = $null
  Restarts = 0
}

function Start-Worker {
  $wargs = @(
    '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $here 'oracle_worker.ps1'),
    '-TaskPath', $TaskPath, '-OutPath', $state.Out, '-HeartbeatPath', $state.HB,
    '-ExpectMajor', $expectMajor, '-TextDir', $state.Texts,
    '-RecycleEvery', $RecycleEvery, '-WarmupDocs', $WarmupDocs
  )
  if ($HideWindow) { $wargs += '-HideWindow' }
  $state.Proc = Start-Process -FilePath 'powershell.exe' -ArgumentList $wargs -PassThru -WindowStyle Hidden
  Write-Output "[sup] worker started (pid $($state.Proc.Id))"
}

# [#4749] The worker writes the heartbeat with WriteAllText while we read it; ReadAllText
# opens with FileShare.Read, which conflicts with the writer's live handle and throws --
# and $ErrorActionPreference='Stop' escalates that into supervisor death (measured: died
# at minute 88 of a 217-minute pass). Open with FileShare.ReadWrite like Get-DoneCount,
# and return $null on any failure so the caller just skips this 10s tick.
function Read-SharedText($path) {
  try {
    $fs = New-Object System.IO.FileStream($path, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
    try {
      $sr = New-Object System.IO.StreamReader($fs, (New-Object System.Text.UTF8Encoding($false)))
      try { return $sr.ReadToEnd() } finally { $sr.Dispose() }
    } finally { $fs.Dispose() }
  } catch { return $null }
}

function Get-DoneCount($path) {
  if (-not (Test-Path -LiteralPath $path)) { return 0 }
  # FileShare.ReadWrite: the worker holds the TSV open for append; a plain read throws and
  # progress would report 0 forever (see page_oracle_run.ps1).
  try {
    $fs = New-Object System.IO.FileStream($path, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
    try {
      $sr = New-Object System.IO.StreamReader($fs, (New-Object System.Text.UTF8Encoding($false)))
      try {
        $n = 0
        while ($null -ne $sr.ReadLine()) { $n++ }
        return $n
      } finally { $sr.Dispose() }
    } finally { $fs.Dispose() }
  } catch { return 0 }
}

Start-Worker

$t0 = Get-Date
$lastReport = Get-Date
while ($true) {
  Start-Sleep -Seconds 10
  $nowMs = [int64]([datetime]::UtcNow - [datetime]'1970-01-01').TotalMilliseconds
  $alive = $false
  $d = Get-DoneCount $state.Out
  $running = $false
  if ($null -ne $state.Proc) {
    try { $running = -not (Get-Process -Id $state.Proc.Id -ErrorAction Stop).HasExited } catch { $running = $false }
  }
  if ($running) {
    $alive = $true
    if (Test-Path -LiteralPath $state.HB) {
      $hbText = Read-SharedText $state.HB
      $parts = if ($null -ne $hbText) { $hbText -split '\|' } else { @() }
      if ($parts.Count -ge 3) {
        # A torn read (writer mid-flight) can leave a non-numeric timestamp; skip the tick.
        $age = $null
        try { $age = ($nowMs - [int64]$parts[1]) / 1000.0 } catch { }
        # [#4751] Size-scaled allowance; heartbeat states without a task key (startup /
        # warmup / ready) fall back to the base allowance.
        $curKey = $parts[2]
        $allowed = $StallSeconds
        if ($sizeMB.ContainsKey($curKey)) { $allowed = $StallSeconds + [int]($StallSecondsPerMB * $sizeMB[$curKey]) }
        if ($null -ne $age -and $age -gt $allowed) {
          $hp = [int]$parts[0]
          Write-Output ("[sup] worker stalled {0:N0}s (allowed {1}s) on '{2}' -> killing Hwp pid {3}" -f $age, $allowed, $curKey, $hp)
          # [#4751] Record the kill so judge.py grades the resulting ERR as
          # ORACLE_TIMEOUT (re-measure) instead of OPEN_FAIL (real defect).
          try { Add-Content -LiteralPath $stallLog -Value ("{0}`t{1:N0}`t{2}" -f $curKey, $age, (Get-Date -Format o)) -Encoding UTF8 } catch { }
          if ($hp -gt 0) { try { Stop-Process -Id $hp -Force -ErrorAction SilentlyContinue } catch { } }
          if ($age -gt ($allowed * 2)) {
            try { Stop-Process -Id $state.Proc.Id -Force -ErrorAction SilentlyContinue } catch { }
          }
        }
      }
    }
  } else {
    # A version assert failure is fatal for the whole pass -- restarting just repeats it.
    if ((Test-Path -LiteralPath $state.Out) -and (Select-String -LiteralPath $state.Out -SimpleMatch '__VERSION_MISMATCH__' -Quiet)) {
      Write-Error "[sup] worker could not obtain Hangul major $expectMajor. Close every Hwp.exe and rerun."
      exit 3
    }
    if ($d -lt $state.Total -and $state.Restarts -lt 30) {
      $state.Restarts++
      Write-Output "[sup] worker exited early ($d/$($state.Total)) -> restart #$($state.Restarts)"
      Start-Worker
      $alive = $true
    }
  }
  if (((Get-Date) - $lastReport).TotalSeconds -ge 60) {
    $el = ((Get-Date) - $t0).TotalMinutes
    $rate = if ($el -gt 0) { $d / $el } else { 0 }
    Write-Output ("[sup] {0}/{1} done, {2:N1} min elapsed, {3:N1} opens/min" -f $d, $state.Total, $el, $rate)
    $lastReport = Get-Date
  }
  if (-not $alive) { break }
}

$d = Get-DoneCount $state.Out
Write-Output ("[sup] PASS {0} COMPLETE: {1}/{2} records, {3:N1} min" -f $HwpVersion, $d, $state.Total, ((Get-Date) - $t0).TotalMinutes)
Write-Output "[sup] reminder: run tools/hangul_version_oracle/restore_com_default.ps1 when all passes are done."
