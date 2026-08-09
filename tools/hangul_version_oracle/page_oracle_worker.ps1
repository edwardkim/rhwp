# Worker: open each HWP/HWPX in Hangul via COM and emit a pagination fingerprint.
#
# Output TSV columns: relpath, status, pages, paras, breakCount, fingerprint
#   fingerprint = comma-joined "page@firstParaIndex" run-length list of page starts,
#   so two versions can be compared on WHERE pages break, not only on how many there are.
#
# The page-position recipe follows tools/verify_pi_page_vs_hangul.py
# (SetPos over every paragraph + XHwpDocumentInfo.CurrentPage).
#
# Launched by page_oracle_run.ps1; can be run directly for a single shard.
# Guide: mydocs/manual/verification/hangul_version_oracle.md
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)][string]$ListPath,
  [Parameter(Mandatory = $true)][string]$OutPath,
  [Parameter(Mandatory = $true)][string]$HeartbeatPath,
  [Parameter(Mandatory = $true)][int]$ExpectMajor,
  [Parameter(Mandatory = $true)][string]$Root,
  [int]$RecycleEvery = 0,
  # Throwaway opens before the measured list, to absorb the blocking first Open() described
  # in Start-Warmup below. 0 disables.
  [int]$WarmupDocs = 5,
  # Hide the Hangul document window through COM. Off by default: Hangul 2018 (major 10)
  # DEADLOCKS in Open() when the window is hidden, even through the COM route. Hiding is
  # cosmetic and does not affect the fingerprint (verified 2022 hidden vs visible, see guide).
  [switch]$HideWindow
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$files = Get-Content -LiteralPath $ListPath -Encoding UTF8 | Where-Object { $_.Trim().Length -gt 0 }

# Resume: skip relpaths already present in the output TSV.
$done = New-Object 'System.Collections.Generic.HashSet[string]'
if (Test-Path -LiteralPath $OutPath) {
  foreach ($ln in [System.IO.File]::ReadAllLines($OutPath, [System.Text.Encoding]::UTF8)) {
    $i = $ln.IndexOf("`t")
    if ($i -gt 0) { $null = $done.Add($ln.Substring(0, $i)) }
  }
}

$writer = New-Object System.IO.StreamWriter($OutPath, $true, (New-Object System.Text.UTF8Encoding($false)))
$writer.AutoFlush = $true

$script:hwp = $null
$script:hwpPid = 0

function Get-HwpMajor([string]$version) {
  $match = [System.Text.RegularExpressions.Regex]::Match($version, '^\s*(\d+)')
  if (-not $match.Success) { throw "cannot parse Hangul version: $version" }
  return [int]$match.Groups[1].Value
}

function New-HwpInstance {
  # Serialize COM creation so the freshly spawned Hwp.exe PID is unambiguous even if a
  # second worker is running (the supervisor needs the PID to kill a stalled instance).
  $mutex = New-Object System.Threading.Mutex($false, 'Global\rhwp_hwp_spawn')
  $null = $mutex.WaitOne()
  try {
    $before = @(Get-Process Hwp -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
    $h = New-Object -ComObject HWPFrame.HwpObject
    # Auto-answer Hangul message boxes; the default (0) blocks forever waiting for a human.
    $null = $h.SetMessageBoxMode(0x00020000)
    # Only effective when Hancom's FilePathCheckerModule is registered; harmless otherwise.
    try { $null = $h.RegisterModule("FilePathCheckDLL", "FilePathCheckerModule") } catch { }
    # Window hiding is OPT-IN. Win32 ShowWindow(SW_HIDE) deadlocks automation on every version,
    # and on Hangul 2018 even the COM route below deadlocks the first Open() -- the call never
    # returns and the pass stalls on document 1. Leaving the window visible costs nothing.
    if ($HideWindow) {
      try { $h.XHwpWindows.Item(0).Visible = $false } catch { }
    }
    $newPid = 0
    for ($i = 0; $i -lt 100; $i++) {
      $after = @(Get-Process Hwp -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
      $diff = @($after | Where-Object { $before -notcontains $_ })
      if ($diff.Count -ge 1) { $newPid = $diff[0]; break }
      Start-Sleep -Milliseconds 100
    }
    # Re-verify on every instance: a recycle or crash-recovery must not silently pick up
    # another Hangul version if the CLSID override changed mid-run.
    $v = $h.Version
    if ((Get-HwpMajor $v) -ne $ExpectMajor) {
      throw "version mismatch: expected major $ExpectMajor, got $v"
    }
    $script:hwp = $h
    $script:hwpPid = $newPid
  } finally {
    $mutex.ReleaseMutex()
    $mutex.Dispose()
  }
}

function Close-HwpInstance {
  if ($null -ne $script:hwp) {
    try { $script:hwp.Quit() } catch { }
    try { [System.Runtime.InteropServices.Marshal]::ReleaseComObject($script:hwp) | Out-Null } catch { }
    $script:hwp = $null
  }
  if ($script:hwpPid -gt 0) {
    try { Stop-Process -Id $script:hwpPid -Force -ErrorAction SilentlyContinue } catch { }
    $script:hwpPid = 0
  }
}

function Write-Heartbeat([string]$current) {
  # supervisor reads: hwpPid | unixMs | current file
  $ms = [int64]([datetime]::UtcNow - [datetime]'1970-01-01').TotalMilliseconds
  try { [System.IO.File]::WriteAllText($HeartbeatPath, "$($script:hwpPid)|$ms|$current", (New-Object System.Text.UTF8Encoding($false))) } catch { }
}

Write-Heartbeat "startup"
try {
  New-HwpInstance
} catch {
  $writer.WriteLine("__VERSION_MISMATCH__`tERR`t0`t0`t0`t$($_.Exception.Message)")
  $writer.Dispose()
  Close-HwpInstance
  exit 3
}
Write-Heartbeat "ready ver=$($script:hwp.Version)"

function Start-Warmup {
  # The first Open() on a Hangul instance that follows a force-killed one can block for minutes
  # (reproduced on 2018 and 2020, section 8 of the guide). The supervisor stall-kills it, the
  # replacement instance blocks the same way, and the pass burns its opening documents on ERR --
  # which resume then treats as done, losing them for good. So spend the block on a throwaway
  # open instead, retrying on a fresh instance until one succeeds.
  #
  # The warm-up target is the list's own first document; opening it twice costs nothing. Extra
  # documents processed before the measured ones do not change fingerprints -- hermetic
  # (one instance per document) and single-instance measurement agreed 200/200, report section 3 --
  # and every version's pass warms up identically anyway.
  if ($WarmupDocs -le 0 -or $files.Count -eq 0) { return }
  $warmFile = $files[0]
  for ($w = 1; $w -le $WarmupDocs; $w++) {
    Write-Heartbeat "warmup $w/$WarmupDocs"
    try {
      $null = $script:hwp.Open($warmFile, "", "forceopen:true")
      try { $null = $script:hwp.Clear(1) } catch { }
      Write-Heartbeat "warmup ok after $w"
      return
    } catch {
      try { $null = $script:hwp.Clear(1) } catch {
        Close-HwpInstance
        try { New-HwpInstance } catch { }
      }
    }
  }
  Write-Heartbeat "warmup exhausted after $WarmupDocs"
}
Start-Warmup

$n = 0
foreach ($f in $files) {
  $rel = $f
  if ($f.StartsWith($Root, [System.StringComparison]::OrdinalIgnoreCase)) {
    $rel = $f.Substring($Root.Length).TrimStart('\')
  }
  if ($done.Contains($rel)) { continue }

  # Two attempts. When the supervisor stall-kills the instance the document that was open at
  # the time fails, and without a retry it is recorded ERR -- which resume then treats as done,
  # so the document is lost for good. That matters because the very first Open() on an instance
  # that follows a force-killed one blocks long enough to be stall-killed itself (reproduced on
  # Hangul 2018 and 2020, section 8 of the guide): every pass would silently drop its first
  # document. Re-measuring on the fresh instance is equivalent -- hermetic and single-instance
  # measurement agreed 200/200 (report section 3).
  $status = 'OK'; $pages = -1; $paras = -1; $fp = ''; $bc = 0
  for ($attempt = 1; $attempt -le 2; $attempt++) {
    # Inside the loop: a retry runs against a new PID and must reset the supervisor's stall timer.
    Write-Heartbeat $rel
    $status = 'OK'; $pages = -1; $paras = -1; $fp = ''; $bc = 0
    try {
      $null = $script:hwp.Open($f, "", "forceopen:true")
      $pages = [int]$script:hwp.PageCount
      $null = $script:hwp.HAction.Run("MoveDocEnd")
      # GetPos() uses [out] params and is not callable from PowerShell; GetPosBySet() is.
      $endSet = $script:hwp.GetPosBySet()
      $maxPara = [int]$endSet.Item("Para")
      $info = $script:hwp.XHwpDocuments.Item(0).XHwpDocumentInfo
      $sb = New-Object System.Text.StringBuilder
      $prev = -1
      for ($p = 0; $p -le $maxPara; $p++) {
        $null = $script:hwp.SetPos(0, $p, 0)
        $pg = [int]$info.CurrentPage
        if ($pg -ne $prev) {
          if ($bc -gt 0) { $null = $sb.Append(',') }
          $null = $sb.Append($pg).Append('@').Append($p)
          $prev = $pg
          $bc++
        }
      }
      $paras = $maxPara + 1
      $fp = $sb.ToString()
      try { $null = $script:hwp.Clear(1) } catch { }
      break
    } catch {
      $status = 'ERR'
      $fp = ($_.Exception.Message -replace "[`t`r`n]", ' ')
      # A dead or hung instance (supervisor kill, COM fault) must be replaced before continuing.
      $replaced = $false
      try { $null = $script:hwp.Clear(1) } catch {
        Close-HwpInstance
        try { New-HwpInstance; $replaced = $true } catch { }
      }
      # Only retry when the instance itself died and was replaced. A failure the live instance
      # shrugged off is document-specific (broken file, unsupported feature) and would repeat.
      if (-not $replaced) { break }
    }
  }
  $writer.WriteLine("$rel`t$status`t$pages`t$paras`t$bc`t$fp")

  $n++
  if ($RecycleEvery -gt 0 -and ($n % $RecycleEvery) -eq 0) {
    Close-HwpInstance
    Start-Sleep -Milliseconds 300
    New-HwpInstance
    Write-Heartbeat "recycled after $n"
  }
}

$writer.Dispose()
Close-HwpInstance
Write-Heartbeat "finished"
exit 0
