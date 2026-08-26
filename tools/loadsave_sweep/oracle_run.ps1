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
  # [#4899] 강제 종료 직후 첫 Open 은 수 분 블록된다(문서화된 현상). 그 회복 시간이 다음
  # 문서의 stall 카운트다운에 그대로 잡히면 kill 이 연쇄해, 6~14KB 짜리 짧은 서식까지
  # 줄줄이 죽는다(s13 실측: 00464·05148 은 orig/h2h/h2x 가 연달아 각 2회씩, 문서당 30분 손실).
  # kill 이후 이 시간 동안은 임계를 넉넉히 잡는다.
  [int]$RecoverySeconds = 600,
  # 회복 구간에서 더해 줄 여유(기본: 기준 임계만큼 = 두 배).
  [int]$RecoveryBonusSeconds = 300,
  [int]$RecycleEvery = 200,
  [int]$WarmupDocs = 5,
  # [#4899] 실패로 끝난 키는 패스 말미에 깨끗한 워커로 1회 다시 잰다. 두 번 다 실패해야
  # 결함으로 기록한다 — s13 에서는 실패 6건이 재측정에서 **6/6 정상**이었고, 그중 하나는
  # 최상위 판정인 OPEN_FAIL 로 남아 있었다.
  [switch]$NoRetryFailures,
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
  Tasks = $TaskPath
  Total = $all.Count
  Proc = $null
  Restarts = 0
}

# [#4899] kill 연쇄 방어용 상태.
$lastKillAt = $null
$killCount = @{}

function Start-Worker {
  $wargs = @(
    '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $here 'oracle_worker.ps1'),
    '-TaskPath', $state.Tasks, '-OutPath', $state.Out, '-HeartbeatPath', $state.HB,
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
        # [#4899] 직전 kill 의 회복 구간이면 임계를 늘린다 — 강제 종료 직후 첫 Open 이
        # 수 분 블록되는 것을 "이 문서가 느리다"로 오해하면 kill 이 연쇄한다.
        $sinceKill = if ($null -ne $lastKillAt) { ((Get-Date) - $lastKillAt).TotalSeconds } else { [double]::PositiveInfinity }
        if ($sinceKill -lt $RecoverySeconds) { $allowed += $RecoveryBonusSeconds }
        # 같은 키를 두 번 죽였으면 세 번째는 죽이지 않고 크게 기다린다(진짜 대형 문서 보호).
        $priorKills = if ($killCount.ContainsKey($curKey)) { [int]$killCount[$curKey] } else { 0 }
        if ($priorKills -ge 2) { $allowed = $allowed * 4 }
        if ($null -ne $age -and $age -gt $allowed) {
          $hp = [int]$parts[0]
          Write-Output ("[sup] worker stalled {0:N0}s (allowed {1}s) on '{2}' -> killing Hwp pid {3}" -f $age, $allowed, $curKey, $hp)
          # [#4751] Record the kill so judge.py grades the resulting ERR as
          # ORACLE_TIMEOUT (re-measure) instead of OPEN_FAIL (real defect).
          # [#4899] 직전 kill 이후 경과(초)를 함께 남긴다 — 사후에 kill 연쇄를 식별하려면
          # "언제 죽였나"만으로는 부족하고 "직전 kill 로부터 얼마 만인가"가 필요하다.
          $gap = if ([double]::IsPositiveInfinity($sinceKill)) { '-' } else { '{0:N0}' -f $sinceKill }
          try { Add-Content -LiteralPath $stallLog -Value ("{0}`t{1:N0}`t{2}`t{3}" -f $curKey, $age, (Get-Date -Format o), $gap) -Encoding UTF8 } catch { }
          $lastKillAt = Get-Date
          $killCount[$curKey] = $priorKills + 1
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

# ============================================================
# [#4899] 실패 키 자동 재측정 — 한 번의 실패로 결함을 단정하지 않는다.
#
# 한글 COM 은 강제 종료·자원 압박 뒤 한동안 불안정해서, 멀쩡한 문서도 `0x800706BE` 로
# 실패한다. s13 전수(29,896 opens)에서 실패로 남은 6건이 재측정에서 **6/6 정상**이었고,
# 그중 `03908.h2h` 는 최상위 판정인 OPEN_FAIL("저장 치명 결함")로 기록돼 있었다.
# 사람이 손으로 다시 재야만 걷히던 가짜 판정을, 패스가 스스로 걷어낸다.
# 재측정 대상은 보통 수 건이라 비용은 사실상 0이다(6/29,896).
# ============================================================
# [#4899] judge.py 는 result.tsv 를 `encoding="utf-8"`(BOM 미허용)로 읽는다. PowerShell 의
# `Set-Content -Encoding UTF8` 은 BOM 을 붙여, 병합본 첫 줄의 key 가 BOM 을 달고 나와
# 그 문서의 판정이 통째로 어긋난다. 워커가 쓰던 형식(BOM 없는 UTF-8)을 그대로 유지한다.
$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)
function Write-Utf8NoBom($path, $lines) {
  [System.IO.File]::WriteAllLines($path, [string[]]@($lines), $Utf8NoBom)
}
function Append-Utf8NoBom($path, $line) {
  [System.IO.File]::AppendAllText($path, ($line + "`r`n"), $Utf8NoBom)
}

function Get-FailedKeys($resultPath) {
  $failed = New-Object System.Collections.Generic.List[string]
  if (-not (Test-Path -LiteralPath $resultPath)) { return $failed }
  $text = Read-SharedText $resultPath
  if ($null -eq $text) { return $failed }
  foreach ($line in $text -split "`r?`n") {
    if ([string]::IsNullOrWhiteSpace($line)) { continue }
    $c = $line -split "`t"
    # result.tsv: key, status, pages, textLen, textSha, ctrls, fileBytes, err
    if ($c.Count -ge 2 -and $c[1] -ne 'OK') { $failed.Add($c[0]) }
  }
  return $failed
}

$retryLog = Join-Path $OutDir 'retried.tsv'
$failedKeys = if ($NoRetryFailures) { @() } else { Get-FailedKeys $state.Out }

if ($failedKeys.Count -gt 0) {
  Write-Output ("[sup] {0} failed key(s) -- re-measuring once with a clean worker (#4899)" -f $failedKeys.Count)

  # 재측정은 오염되지 않은 상태에서 시작한다: 남은 Hwp.exe 를 모두 내리고 잠시 쉰다.
  $stale = @(Get-Process -Name 'Hwp' -ErrorAction SilentlyContinue)
  if ($stale.Count -gt 0) {
    Write-Output "[sup] terminating $($stale.Count) leftover Hwp.exe process(es) before retry"
    foreach ($p in $stale) { try { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue } catch { } }
  }
  Start-Sleep -Seconds 20

  $failedSet = @{}
  foreach ($k in $failedKeys) { $failedSet[$k] = $true }
  $retryTasks = Join-Path $OutDir 'retry_tasks.tsv'
  $retryLines = foreach ($taskLine in $all) {
    $c = $taskLine -split "`t"
    if ($c.Count -ge 2 -and $failedSet.ContainsKey($c[0])) { $taskLine }
  }
  Write-Utf8NoBom $retryTasks $retryLines

  $retryOut = Join-Path $OutDir 'result_retry.tsv'
  if (Test-Path -LiteralPath $retryOut) { Remove-Item -LiteralPath $retryOut -Force }

  $state.Tasks = $retryTasks
  $state.Out = $retryOut
  $state.Total = @($retryLines).Count
  $state.Restarts = 0
  # 재측정에서는 임계를 넉넉히 준다 — 대형 문서가 기준 임계에 걸려 또 죽으면 의미가 없다.
  $StallSeconds = [Math]::Max($StallSeconds * 4, 1200)
  $t1 = Get-Date
  Start-Worker
  while ($true) {
    Start-Sleep -Seconds 10
    $rd = Get-DoneCount $state.Out
    $running = $false
    if ($null -ne $state.Proc) {
      try { $running = -not (Get-Process -Id $state.Proc.Id -ErrorAction Stop).HasExited } catch { $running = $false }
    }
    if (-not $running) {
      if ($rd -lt $state.Total -and $state.Restarts -lt 5) {
        $state.Restarts++
        Write-Output "[sup] retry worker exited early ($rd/$($state.Total)) -> restart #$($state.Restarts)"
        Start-Worker
        continue
      }
      break
    }
    if (((Get-Date) - $t1).TotalMinutes -gt 60) {
      Write-Output "[sup] retry pass over 60 min -- stopping"
      try { Stop-Process -Id $state.Proc.Id -Force -ErrorAction SilentlyContinue } catch { }
      break
    }
  }

  # 재측정 결과를 본 결과에 반영한다. 두 번 다 실패한 키는 그대로 둔다(진짜 결함).
  $retryRows = @{}
  $rtext = Read-SharedText $retryOut
  if ($null -ne $rtext) {
    foreach ($line in $rtext -split "`r?`n") {
      if ([string]::IsNullOrWhiteSpace($line)) { continue }
      $c = $line -split "`t"
      if ($c.Count -ge 2) { $retryRows[$c[0]] = $line }
    }
  }
  $mainPath = Join-Path $OutDir 'result.tsv'
  $mainText = Read-SharedText $mainPath
  $merged = New-Object System.Collections.Generic.List[string]
  $recovered = 0
  $stillFailed = 0
  foreach ($line in ($mainText -split "`r?`n")) {
    if ([string]::IsNullOrWhiteSpace($line)) { continue }
    $c = $line -split "`t"
    $key = $c[0]
    if ($retryRows.ContainsKey($key)) {
      $rc = $retryRows[$key] -split "`t"
      if ($rc.Count -ge 2 -and $rc[1] -eq 'OK') {
        $merged.Add($retryRows[$key])
        Append-Utf8NoBom $retryLog ("{0}`t{1}`t{2}" -f $key, $c[1], 'OK')
        $recovered++
        continue
      }
      Append-Utf8NoBom $retryLog ("{0}`t{1}`t{2}" -f $key, $c[1], $rc[1])
      $stillFailed++
    }
    $merged.Add($line)
  }
  Write-Utf8NoBom $mainPath $merged
  # 재측정 목록에 넣었는데 결과 행이 없는 키(작업 목록에 없거나 워커가 못 끝낸 경우)는
  # 회복도 잔존 실패도 아니다 — 따로 세어야 "왜 숫자가 안 맞나"를 나중에 되짚을 수 있다.
  $notMeasured = $failedKeys.Count - $recovered - $stillFailed
  Write-Output ("[sup] retry {0}: recovered {1} / still failed {2} / not re-measured {3} -> {4}" -f $failedKeys.Count, $recovered, $stillFailed, $notMeasured, $retryLog)
}

Write-Output "[sup] reminder: run tools/hangul_version_oracle/restore_com_default.ps1 when all passes are done."
