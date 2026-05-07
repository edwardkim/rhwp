$ErrorActionPreference = 'Continue'
$root = 'C:\Users\planet\rhwp'
$srcRoot = Join-Path $root 'samples'
$outRoot = Join-Path $root 'pdf'
$log = Join-Path $outRoot '_convert.log'

if (Test-Path $log) { Remove-Item $log -Force }

function Log([string]$msg) {
  $line = "[{0}] {1}" -f (Get-Date -Format 'HH:mm:ss'), $msg
  Add-Content -LiteralPath $log -Value $line -Encoding UTF8
  Write-Output $line
}

function New-HwpInstance {
  $h = New-Object -ComObject 'HWPFrame.HwpObject'
  $reg = $h.RegisterModule('FilePathCheckDLL', 'FilePathCheckerModule')
  if (-not $reg) { Log "WARN: RegisterModule returned False (security dialogs may appear)" }
  return $h
}

function Close-HwpInstance($h) {
  if ($null -eq $h) { return }
  try { $h.Quit() } catch {}
  try { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($h) } catch {}
  # Kill any lingering Hwp processes from this COM instance
  Get-Process -Name 'Hwp' -ErrorAction SilentlyContinue | ForEach-Object {
    try { Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue } catch {}
  }
  Start-Sleep -Milliseconds 500
}

$files = Get-ChildItem -LiteralPath $srcRoot -Recurse -File |
  Where-Object { $_.Extension -ieq '.hwp' -or $_.Extension -ieq '.hwpx' } |
  Sort-Object FullName

Log ("TOTAL " + $files.Count + " files")

# Skip files that are already converted (resume support)
$alreadyOk = @{}
# (intentionally empty - this run forces full reconvert per user request)

$hwp = New-HwpInstance
$ok = 0; $fail = 0; $skip = 0; $i = 0; $crashes = 0
$failures = @()

foreach ($f in $files) {
  $i++
  $rel = $f.FullName.Substring($srcRoot.Length + 1)
  $relDir = Split-Path -Parent $rel
  $outDir = if ([string]::IsNullOrEmpty($relDir)) { $outRoot } else { Join-Path $outRoot $relDir }
  if (-not (Test-Path -LiteralPath $outDir)) { New-Item -ItemType Directory -Path $outDir -Force | Out-Null }
  $stem = [IO.Path]::GetFileNameWithoutExtension($f.Name)
  $outPath = Join-Path $outDir ($stem + '-2022.pdf')

  $start = Get-Date
  $opened = $false
  $errMsg = $null
  $crashed = $false
  try {
    $opened = $hwp.Open($f.FullName, '', '')
    if (-not $opened) { throw 'Open returned False' }
    # Use FileSaveAs_S action with Attributes=0 (pyhwpx pattern) for explicit 1-up PDF
    $pset = $hwp.HParameterSet.HFileOpenSave
    $null = $hwp.HAction.GetDefault('FileSaveAs_S', $pset.HSet)
    $pset.filename = $outPath
    $pset.Format = 'PDF'
    $pset.Attributes = 0
    $r = $hwp.HAction.Execute('FileSaveAs_S', $pset.HSet)
    if (-not $r) {
      # Fallback to direct SaveAs
      $r = $hwp.SaveAs($outPath, 'PDF', '')
    }
    if (-not $r) { throw 'SaveAs returned False' }
    $dur = ((Get-Date) - $start).TotalSeconds
    $sz = if (Test-Path -LiteralPath $outPath) { (Get-Item -LiteralPath $outPath).Length } else { 0 }
    Log ("[{0}/{1}] OK  {2}  ({3:N1}s, {4} bytes)" -f $i, $files.Count, $rel, $dur, $sz)
    $ok++
  } catch {
    $errMsg = $_.Exception.Message
    # Detect COM server crash (RPC failures)
    if ($errMsg -match '0x800706B[AE]|RPC server|remote procedure call') { $crashed = $true }
    Log ("[{0}/{1}] FAIL {2} : {3}" -f $i, $files.Count, $rel, $errMsg)
    $failures += $rel
    $fail++
  } finally {
    if ($opened -and -not $crashed) {
      try { $hwp.XHwpDocuments.Item(0).Close($false) } catch {}
    }
  }

  if ($crashed) {
    $crashes++
    Log ("CRASH detected -- restarting Hwp COM instance (crash #" + $crashes + ")")
    Close-HwpInstance $hwp
    $hwp = New-HwpInstance
  }
}

Log ("DONE: ok=" + $ok + " fail=" + $fail + " crashes=" + $crashes)
if ($failures.Count -gt 0) {
  Log "FAILED FILES:"
  foreach ($x in $failures) { Log ("  - " + $x) }
}

Close-HwpInstance $hwp
