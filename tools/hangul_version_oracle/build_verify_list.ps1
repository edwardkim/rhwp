# Build the reproduction-check list for a diff report.
#
# The guide requires that a diff list is never trusted as measured: rerun the documents that
# came out DIFFERENT together with a control group that came out IDENTICAL, in a different
# order, and keep only the classifications that reproduce. This builds that mixed list.
#
# Controls matter as much as the diffs. Without them a rerun can only lose differences, never
# reveal ones the first run missed, so a silent false-negative rate stays invisible.
#
# Output is an absolute-path list in UTF-8 without BOM, ready for page_oracle_run.ps1 -ListPath.
#
# Guide: mydocs/manual/verification/hangul_version_oracle.md
[CmdletBinding()]
param(
  # diff.tsv produced by compare_passes.ps1 (header + kind/path/detail rows).
  [Parameter(Mandatory = $true)][string]$DiffPath,
  # Any pass directory covering the same corpus; supplies the pool of identical documents.
  [Parameter(Mandatory = $true)][string]$PassDir,
  # Corpus root, prepended to the relative paths recorded in the passes.
  [Parameter(Mandatory = $true)][string]$Root,
  [Parameter(Mandatory = $true)][string]$OutPath,
  [int]$Controls = 100,
  # Fixed so the same inputs rebuild the same list.
  [int]$Seed = 20260807
)

$ErrorActionPreference = 'Stop'

$diffRel = New-Object 'System.Collections.Generic.List[string]'
$seen = New-Object 'System.Collections.Generic.HashSet[string]'
$first = $true
foreach ($ln in [System.IO.File]::ReadAllLines($DiffPath, [System.Text.Encoding]::UTF8)) {
  if ($first) { $first = $false; if ($ln -like "kind`t*") { continue } }
  if (-not $ln) { continue }
  $c = $ln -split "`t", 3
  if ($c.Count -lt 2) { continue }
  if ($seen.Add($c[1])) { $diffRel.Add($c[1]) }
}

$allRel = New-Object 'System.Collections.Generic.List[string]'
foreach ($f in (Get-ChildItem (Join-Path $PassDir 'result_*.tsv') -ErrorAction SilentlyContinue)) {
  foreach ($ln in [System.IO.File]::ReadAllLines($f.FullName, [System.Text.Encoding]::UTF8)) {
    if (-not $ln) { continue }
    $i = $ln.IndexOf("`t")
    if ($i -gt 0) { $allRel.Add($ln.Substring(0, $i)) }
  }
}
$pool = @($allRel | Sort-Object -Unique | Where-Object { -not $seen.Contains($_) })
if ($pool.Count -eq 0) { throw "no identical documents left for controls -- is $PassDir the right pass?" }
if ($Controls -gt $pool.Count) {
  Write-Warning "requested $Controls controls but only $($pool.Count) identical documents exist; using all of them"
  $Controls = $pool.Count
}

$ctrl = @($pool | Get-Random -Count $Controls -SetSeed $Seed)
# Shuffle diffs and controls together: the rerun must not process them in the original order,
# because Hangul's pagination state carries across documents within one instance.
$mixed = @(@($diffRel) + $ctrl | Get-Random -Count ($diffRel.Count + $ctrl.Count) -SetSeed ($Seed + 1))

$abs = foreach ($r in $mixed) { Join-Path $Root $r }
[System.IO.File]::WriteAllLines($OutPath, $abs, (New-Object System.Text.UTF8Encoding($false)))

$missing = @($abs | Where-Object { -not (Test-Path -LiteralPath $_) }).Count
Write-Output "diffs: $($diffRel.Count), controls: $Controls, total: $($abs.Count)"
if ($missing -gt 0) { Write-Warning "$missing listed paths do not exist under $Root -- check -Root" }
Write-Output "list: $OutPath"
