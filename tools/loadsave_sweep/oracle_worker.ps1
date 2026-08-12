# Phase B worker: open each task file (original or rhwp output) in Hangul via COM and
# record what Hangul sees: page count, body text (saved to TextDir, sha in the TSV), and
# a control census (CtrlID counts). The judge compares <docid>.orig against <docid>.<route>.
#
# Task list format: "key<TAB>abspath" per line, key = <docid>.<orig|h2h|h2x|x2h|x2x>.
# Output TSV columns: key, status, pages, textLen, textSha, ctrls, fileBytes, err
#
# COM discipline (heartbeat / stall recovery / warmup / two-attempt retry) is inherited from
# tools/hangul_version_oracle/page_oracle_worker.ps1 -- see that file and
# mydocs/manual/verification/hangul_version_oracle.md for the rationale of each rule.
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)][string]$TaskPath,
  [Parameter(Mandatory = $true)][string]$OutPath,
  [Parameter(Mandatory = $true)][string]$HeartbeatPath,
  [Parameter(Mandatory = $true)][int]$ExpectMajor,
  [Parameter(Mandatory = $true)][string]$TextDir,
  [int]$RecycleEvery = 200,
  [int]$WarmupDocs = 5,
  # Hangul 2018 (major 10) deadlocks in Open() when the window is hidden -- keep visible there.
  [switch]$HideWindow
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

if (-not (Test-Path -LiteralPath $TextDir)) { New-Item -ItemType Directory -Force -Path $TextDir | Out-Null }

$tasks = @()
foreach ($ln in (Get-Content -LiteralPath $TaskPath -Encoding UTF8 | Where-Object { $_.Trim().Length -gt 0 })) {
  $i = $ln.IndexOf("`t")
  if ($i -gt 0) { $tasks += , @($ln.Substring(0, $i), $ln.Substring($i + 1)) }
}

# Resume: skip keys already present in the output TSV.
$done = New-Object 'System.Collections.Generic.HashSet[string]'
if (Test-Path -LiteralPath $OutPath) {
  foreach ($ln in [System.IO.File]::ReadAllLines($OutPath, [System.Text.Encoding]::UTF8)) {
    $i = $ln.IndexOf("`t")
    if ($i -gt 0) { $null = $done.Add($ln.Substring(0, $i)) }
  }
}

$enc = New-Object System.Text.UTF8Encoding($false)
$writer = New-Object System.IO.StreamWriter($OutPath, $true, $enc)
$writer.AutoFlush = $true
$sha256 = [System.Security.Cryptography.SHA256]::Create()

$script:hwp = $null
$script:hwpPid = 0

function Get-HwpMajor([string]$version) {
  $match = [System.Text.RegularExpressions.Regex]::Match($version, '^\s*(\d+)')
  if (-not $match.Success) { throw "cannot parse Hangul version: $version" }
  return [int]$match.Groups[1].Value
}

function New-HwpInstance {
  $mutex = New-Object System.Threading.Mutex($false, 'Global\rhwp_hwp_spawn')
  $null = $mutex.WaitOne()
  try {
    $before = @(Get-Process Hwp -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
    $h = New-Object -ComObject HWPFrame.HwpObject
    # Auto-answer Hangul message boxes; the default (0) blocks forever waiting for a human.
    $null = $h.SetMessageBoxMode(0x00020000)
    try { $null = $h.RegisterModule("FilePathCheckDLL", "FilePathCheckerModule") } catch { }
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
    # Re-verify on every instance: a recycle must not silently pick up another Hangul version.
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
  $ms = [int64]([datetime]::UtcNow - [datetime]'1970-01-01').TotalMilliseconds
  try { [System.IO.File]::WriteAllText($HeartbeatPath, "$($script:hwpPid)|$ms|$current", $enc) } catch { }
}

Write-Heartbeat "startup"
try {
  New-HwpInstance
} catch {
  $writer.WriteLine("__VERSION_MISMATCH__`tERR`t0`t0`t`t`t0`t$($_.Exception.Message)")
  $writer.Dispose()
  Close-HwpInstance
  exit 3
}
Write-Heartbeat "ready ver=$($script:hwp.Version)"

function Start-Warmup {
  # Absorb the first-Open() block that follows a force-killed instance (see page_oracle_worker).
  if ($WarmupDocs -le 0 -or $tasks.Count -eq 0) { return }
  $warmFile = $tasks[0][1]
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
foreach ($t in $tasks) {
  $key = $t[0]; $path = $t[1]
  if ($done.Contains($key)) { continue }

  $fileBytes = 0
  try { $fileBytes = (Get-Item -LiteralPath $path).Length } catch { }

  # Two attempts: the first document after a stall-kill fails on the dead instance and must be
  # re-measured on its replacement, or resume treats the ERR row as done and loses it for good.
  $status = 'OK'; $pages = -1; $textLen = -1; $textSha = ''; $ctrls = ''; $err = ''
  for ($attempt = 1; $attempt -le 2; $attempt++) {
    Write-Heartbeat $key
    $status = 'OK'; $pages = -1; $textLen = -1; $textSha = ''; $ctrls = ''; $err = ''
    try {
      $null = $script:hwp.Open($path, "", "forceopen:true")
      $pages = [int]$script:hwp.PageCount

      # Body text as Hangul sees it. CP949-external chars arrive as &#N; escapes -- symmetric
      # between orig and variant under the same Hangul version, so comparisons stay fair.
      $text = [string]$script:hwp.GetTextFile("TEXT", "")
      if ($null -eq $text) { $text = '' }
      $textLen = $text.Length
      $bytes = [System.Text.Encoding]::UTF8.GetBytes($text)
      $textSha = ([System.BitConverter]::ToString($sha256.ComputeHash($bytes)) -replace '-', '').ToLowerInvariant()
      [System.IO.File]::WriteAllBytes((Join-Path $TextDir "$key.txt"), $bytes)

      # Control census: walk the HeadCtrl chain and count CtrlIDs (tbl, gso, ...). Whatever the
      # chain covers, it covers identically for orig and variant -- the comparison is fair.
      $counts = @{}
      try {
        $c = $script:hwp.HeadCtrl
        $guard = 0
        while ($null -ne $c -and $guard -lt 500000) {
          $id = [string]$c.CtrlID
          if ($counts.ContainsKey($id)) { $counts[$id]++ } else { $counts[$id] = 1 }
          $c = $c.Next
          $guard++
        }
        $ctrls = (($counts.GetEnumerator() | Sort-Object Name | ForEach-Object { "$($_.Name):$($_.Value)" }) -join ',')
      } catch {
        $ctrls = ''
        $msg = $_.Exception.Message -replace "[`t`r`n]", ' '
        $err = "ctrl-walk: $msg"
      }

      try { $null = $script:hwp.Clear(1) } catch { }
      break
    } catch {
      $status = 'ERR'
      $err = ($_.Exception.Message -replace "[`t`r`n]", ' ')
      $replaced = $false
      try { $null = $script:hwp.Clear(1) } catch {
        Close-HwpInstance
        try { New-HwpInstance; $replaced = $true } catch { }
      }
      # Only retry when the instance itself died and was replaced. A failure the live instance
      # shrugged off is document-specific and would just repeat.
      if (-not $replaced) { break }
    }
  }
  $writer.WriteLine("$key`t$status`t$pages`t$textLen`t$textSha`t$ctrls`t$fileBytes`t$err")

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
