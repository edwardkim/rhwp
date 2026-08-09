# Compare two Hangul passes and report only the documents whose pagination differs.
#
# Classification per document:
#   PAGE_DELTA  - total page count differs
#   BREAK_DIFF  - same page count, but a page starts at a different paragraph
#   PARA_DIFF   - paragraph count differs (document read differently, not a layout diff)
#   ERR         - one or both sides failed to produce a fingerprint
#   MISSING     - present in only one pass
#   MATCH       - identical pagination (not written to the report)
#
# Guide: mydocs/manual/verification/hangul_version_oracle.md
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)][string]$DirA,
  [Parameter(Mandatory = $true)][string]$DirB,
  [Parameter(Mandatory = $true)][string]$OutPath,
  [string]$LabelA = 'A',
  [string]$LabelB = 'B'
)

$ErrorActionPreference = 'Stop'

function Read-Pass($dir) {
  $map = @{}
  foreach ($f in (Get-ChildItem (Join-Path $dir 'result_*.tsv') -ErrorAction SilentlyContinue)) {
    foreach ($ln in [System.IO.File]::ReadAllLines($f.FullName, [System.Text.Encoding]::UTF8)) {
      if (-not $ln) { continue }
      $c = $ln -split "`t", 6
      if ($c.Count -lt 6) { continue }
      # last write wins: a retried document supersedes its earlier error row
      $map[$c[0]] = [pscustomobject]@{
        Status = $c[1]; Pages = [int]$c[2]; Paras = [int]$c[3]; Breaks = [int]$c[4]; Fp = $c[5]
      }
    }
  }
  return $map
}

$a = Read-Pass $DirA
$b = Read-Pass $DirB
Write-Output "[cmp] $LabelA records: $($a.Count), $LabelB records: $($b.Count)"

$keys = New-Object 'System.Collections.Generic.HashSet[string]'
foreach ($k in $a.Keys) { $null = $keys.Add($k) }
foreach ($k in $b.Keys) { $null = $keys.Add($k) }

$rows = New-Object System.Collections.Generic.List[object]
$counts = @{}
foreach ($k in ($keys | Sort-Object)) {
  $ra = $a[$k]; $rb = $b[$k]
  $kind = $null; $detail = ''
  if ($null -eq $ra -or $null -eq $rb) {
    $kind = 'MISSING'
    $detail = "present in: " + $(if ($ra) { $LabelA } else { $LabelB })
  } elseif ($ra.Status -ne 'OK' -or $rb.Status -ne 'OK') {
    $kind = 'ERR'
    $detail = "$LabelA=$($ra.Status) $LabelB=$($rb.Status)"
  } elseif ($ra.Pages -ne $rb.Pages) {
    $kind = 'PAGE_DELTA'
    $detail = "$LabelA=$($ra.Pages)p $LabelB=$($rb.Pages)p delta=$($rb.Pages - $ra.Pages)"
  } elseif ($ra.Paras -ne $rb.Paras) {
    $kind = 'PARA_DIFF'
    $detail = "$LabelA=$($ra.Paras)para $LabelB=$($rb.Paras)para"
  } elseif ($ra.Fp -ne $rb.Fp) {
    $kind = 'BREAK_DIFF'
    $fa = $ra.Fp -split ','
    $fb = $rb.Fp -split ','
    $n = [Math]::Min($fa.Count, $fb.Count)
    $at = -1
    for ($i = 0; $i -lt $n; $i++) { if ($fa[$i] -ne $fb[$i]) { $at = $i; break } }
    if ($at -lt 0) { $at = $n }
    $va = if ($at -lt $fa.Count) { $fa[$at] } else { '(end)' }
    $vb = if ($at -lt $fb.Count) { $fb[$at] } else { '(end)' }
    $detail = "first divergence #${at}: $LabelA=$va $LabelB=$vb (pages=$($ra.Pages))"
  } else {
    $kind = 'MATCH'
  }
  if (-not $counts.ContainsKey($kind)) { $counts[$kind] = 0 }
  $counts[$kind]++
  if ($kind -ne 'MATCH') {
    $rows.Add([pscustomobject]@{ Kind = $kind; Path = $k; Detail = $detail })
  }
}

$order = @('PAGE_DELTA', 'BREAK_DIFF', 'PARA_DIFF', 'MISSING', 'ERR')
$sorted = $rows | Sort-Object @{ Expression = { $i = $order.IndexOf($_.Kind); if ($i -lt 0) { 99 } else { $i } } }, Path

$out = New-Object System.Collections.Generic.List[string]
$out.Add("kind`tpath`tdetail")
foreach ($r in $sorted) { $out.Add("$($r.Kind)`t$($r.Path)`t$($r.Detail)") }
[System.IO.File]::WriteAllLines($OutPath, $out, (New-Object System.Text.UTF8Encoding($false)))

Write-Output '=== SUMMARY ==='
foreach ($k in ($counts.Keys | Sort-Object)) { Write-Output ("{0,-12} {1}" -f $k, $counts[$k]) }
Write-Output "report: $OutPath"
