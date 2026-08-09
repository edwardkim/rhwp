# Supervisor: run one full pass of a document corpus under a chosen Hangul version.
#
#   - Selects the Hangul version by overriding the COM CLSID LocalServer32 in HKCU.
#     Every Hangul release shares the same CLSID, so the HKLM registration always points at
#     whichever version was installed last. HKCU wins over HKLM for COM activation, so this
#     picks a version per-user without admin rights and is fully reversible.
#   - The expected major version is read from the selected Hwp.exe itself, so any installed
#     release works (2018 = 10.x, 2020 = 11.x, 2022 = 12.x, 2024 = 13.x, ...) with no code change.
#   - Runs ONE worker by default. Running several Hangul instances at once corrupts the
#     measurement -- see the guide, section "why -Workers 1".
#   - Watches a heartbeat, kills a stalled Hwp.exe (the worker then recovers), and relaunches
#     a worker process that died.
#
# Guide: mydocs/manual/verification/hangul_version_oracle.md
[CmdletBinding()]
param(
  # Hangul release year as installed under Hnc\Office <year>, e.g. 2018 / 2020 / 2022 / 2024.
  [Parameter(Mandatory = $true, ParameterSetName = 'ByVersion')][string]$HwpVersion,
  # Explicit path to Hwp.exe, for non-standard install locations.
  [Parameter(Mandatory = $true, ParameterSetName = 'ByPath')][string]$HwpExe,
  [Parameter(Mandatory = $true)][string]$ListPath,
  [Parameter(Mandatory = $true)][string]$OutDir,
  [Parameter(Mandatory = $true)][string]$Root,
  [int]$Workers = 1,
  [int]$StallSeconds = 300,
  [int]$RecycleEvery = 0,
  # Throwaway opens each worker performs before the measured list. Absorbs the first-Open()
  # block that follows a force-killed instance; without it a pass loses its opening documents
  # to ERR and resume never retries them. 0 disables.
  [int]$WarmupDocs = 5,
  # Hide the Hangul document window through COM. Off by default -- Hangul 2018 deadlocks in
  # Open() when hidden. Only pass this on 2022+ and only if the visible window is in the way.
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
  $installed = @()
  foreach ($r in $roots) {
    $installed += (Get-ChildItem (Join-Path $r 'Hnc') -Directory -ErrorAction SilentlyContinue | ForEach-Object { $_.Name })
  }
  throw "Hangul $version not found. Installed: $($installed -join ', ')"
}

if ($PSCmdlet.ParameterSetName -eq 'ByVersion') {
  $exePath = Resolve-HwpExe $HwpVersion
  $label = $HwpVersion
} else {
  if (-not (Test-Path -LiteralPath $HwpExe)) { throw "not found: $HwpExe" }
  $exePath = (Resolve-Path -LiteralPath $HwpExe).Path
  $label = 'custom'
}

# The binary is the authority on which major version this pass must see.
$productVersion = (Get-Item -LiteralPath $exePath).VersionInfo.ProductVersion
$expectMajor = [int](($productVersion -split '[.,]')[0].Trim())
Write-Output "[sup] Hangul $label -> $exePath (ProductVersion $productVersion, major $expectMajor)"

foreach ($base in "HKCU:\Software\Classes\CLSID\$CLSID\LocalServer32", "HKCU:\Software\Classes\Wow6432Node\CLSID\$CLSID\LocalServer32") {
  New-Item -Path $base -Force | Out-Null
  Set-ItemProperty -Path $base -Name '(default)' -Value "$exePath -Automation"
}
Write-Output "[sup] COM CLSID override set (HKCU)"

# A leftover Hwp.exe makes the override useless: COM attaches to the ALREADY RUNNING instance,
# which may be the other version, and the whole pass is silently measured with the wrong binary.
# The worker's version assert catches it, but only after the pass has been wasted.
$stale = @(Get-Process Hwp -ErrorAction SilentlyContinue)
if ($stale.Count -gt 0) {
  Write-Output "[sup] terminating $($stale.Count) leftover Hwp.exe process(es) so the override takes effect"
  $stale | Stop-Process -Force -ErrorAction SilentlyContinue
  Start-Sleep -Seconds 2
}

# Prove the override actually took effect BEFORE spending a pass on it. Two things silently
# defeat it: a leftover Hwp.exe (COM attaches to the running instance), and a previously
# DELETED HKCU CLSID key (COM then ignores HKCU for the rest of the login session).
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

Check, in order:
  1. Every Hwp.exe is closed (this script kills them, but a UI instance may have respawned).
  2. The HKCU override key was never DELETED. Deleting
     HKCU\Software\Classes\[Wow6432Node\]CLSID\$CLSID
     makes COM ignore HKCU for the rest of the login session -- writing the value back does
     not help. Log off and back on, then rerun. Use restore_com_default.ps1 (which sets the
     value instead of deleting the key) to clean up after a run.
"@
}
Write-Output "[sup] verified: COM hands out major $probeMajor ($probeVersion)"

if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Force -Path $OutDir | Out-Null }
$all = Get-Content -LiteralPath $ListPath -Encoding UTF8 | Where-Object { $_.Trim().Length -gt 0 }
Write-Output "[sup] corpus: $($all.Count) files, workers: $Workers"
if ($Workers -gt 1) {
  Write-Warning "Workers > 1 runs several Hangul instances concurrently and CORRUPTS the measurement (18% of documents disagreed in the r1 A/B). Use it only for throughput experiments, never for results."
}

# Round-robin sharding keeps big and small documents spread evenly.
$shards = @{}
for ($i = 0; $i -lt $Workers; $i++) { $shards[$i] = New-Object System.Collections.Generic.List[string] }
for ($i = 0; $i -lt $all.Count; $i++) { $shards[$i % $Workers].Add($all[$i]) }

$enc = New-Object System.Text.UTF8Encoding($false)
$state = @{}
for ($i = 0; $i -lt $Workers; $i++) {
  $lp = Join-Path $OutDir "shard_$i.txt"
  [System.IO.File]::WriteAllLines($lp, $shards[$i], $enc)
  $state[$i] = @{
    List = $lp
    Out = Join-Path $OutDir "result_$i.tsv"
    HB = Join-Path $OutDir "hb_$i.txt"
    Total = $shards[$i].Count
    Proc = $null
    Restarts = 0
  }
}

function Start-Worker($i) {
  $s = $state[$i]
  $wargs = @(
    '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $here 'page_oracle_worker.ps1'),
    '-ListPath', $s.List, '-OutPath', $s.Out, '-HeartbeatPath', $s.HB,
    '-ExpectMajor', $expectMajor, '-Root', $Root, '-RecycleEvery', $RecycleEvery,
    '-WarmupDocs', $WarmupDocs
  )
  if ($HideWindow) { $wargs += '-HideWindow' }
  $s.Proc = Start-Process -FilePath 'powershell.exe' -ArgumentList $wargs -PassThru -WindowStyle Hidden
  Write-Output "[sup] worker $i started (pid $($s.Proc.Id))"
}

function Get-DoneCount($path) {
  if (-not (Test-Path -LiteralPath $path)) { return 0 }
  # The worker holds this file open for append. A plain read throws IOException, and the catch
  # below would then report 0 forever: the progress line stays at 0/N for the whole pass, and a
  # worker that finished normally looks unfinished and gets restarted until the restart cap.
  # Opening with FileShare.ReadWrite lets the supervisor read alongside the writer.
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

for ($i = 0; $i -lt $Workers; $i++) { Start-Worker $i }

$t0 = Get-Date
$lastReport = Get-Date
while ($true) {
  Start-Sleep -Seconds 10
  $nowMs = [int64]([datetime]::UtcNow - [datetime]'1970-01-01').TotalMilliseconds
  $alive = 0
  $doneTotal = 0
  for ($i = 0; $i -lt $Workers; $i++) {
    $s = $state[$i]
    $d = Get-DoneCount $s.Out
    $doneTotal += $d
    $running = $false
    if ($null -ne $s.Proc) {
      try { $running = -not (Get-Process -Id $s.Proc.Id -ErrorAction Stop).HasExited } catch { $running = $false }
    }
    if ($running) {
      $alive++
      if (Test-Path -LiteralPath $s.HB) {
        $parts = ([System.IO.File]::ReadAllText($s.HB, [System.Text.Encoding]::UTF8)) -split '\|'
        if ($parts.Count -ge 3) {
          $age = ($nowMs - [int64]$parts[1]) / 1000.0
          if ($age -gt $StallSeconds) {
            $hp = [int]$parts[0]
            Write-Output ("[sup] worker {0} stalled {1:N0}s on '{2}' -> killing Hwp pid {3}" -f $i, $age, $parts[2], $hp)
            if ($hp -gt 0) { try { Stop-Process -Id $hp -Force -ErrorAction SilentlyContinue } catch { } }
            # Give the worker one stall window to recover; if it does not, kill and relaunch it.
            if ($age -gt ($StallSeconds * 2)) {
              try { Stop-Process -Id $s.Proc.Id -Force -ErrorAction SilentlyContinue } catch { }
            }
          }
        }
      }
    } else {
      # A version assert failure is fatal for the whole pass -- restarting just repeats it.
      if ((Test-Path -LiteralPath $s.Out) -and (Select-String -LiteralPath $s.Out -SimpleMatch '__VERSION_MISMATCH__' -Quiet)) {
        Write-Error "[sup] worker $i could not obtain Hangul major $expectMajor. Close every Hwp.exe and rerun; COM attaches to an already running instance regardless of the override."
        for ($k = 0; $k -lt $Workers; $k++) {
          if ($null -ne $state[$k].Proc) { try { Stop-Process -Id $state[$k].Proc.Id -Force -ErrorAction SilentlyContinue } catch { } }
        }
        exit 3
      }
      if ($d -lt $s.Total -and $s.Restarts -lt 30) {
        $s.Restarts++
        Write-Output "[sup] worker $i exited early ($d/$($s.Total)) -> restart #$($s.Restarts)"
        Start-Worker $i
        $alive++
      }
    }
  }
  if (((Get-Date) - $lastReport).TotalSeconds -ge 60) {
    $el = ((Get-Date) - $t0).TotalMinutes
    $rate = if ($el -gt 0) { $doneTotal / $el } else { 0 }
    Write-Output ("[sup] {0}/{1} done, {2:N1} min elapsed, {3:N1} docs/min, alive={4}" -f $doneTotal, $all.Count, $el, $rate, $alive)
    $lastReport = Get-Date
  }
  if ($alive -eq 0) { break }
}

$doneTotal = 0
for ($i = 0; $i -lt $Workers; $i++) { $doneTotal += (Get-DoneCount $state[$i].Out) }
Write-Output ("[sup] PASS {0} COMPLETE: {1}/{2} records, {3:N1} min" -f $label, $doneTotal, $all.Count, ((Get-Date) - $t0).TotalMinutes)
Write-Output "[sup] reminder: run restore_com_default.ps1 when all passes are done."
