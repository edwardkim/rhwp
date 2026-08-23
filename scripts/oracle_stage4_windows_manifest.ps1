<#
.SYNOPSIS
  Issue #4963 Stage W5-4 disposable-guest font manifest probe.

.DESCRIPTION
  Produces a path-free digest summary for Windows-registered and Hancom-bundled
  fonts. The local-only font source root is accessed only through the four
  contract-declared files; the script never enumerates that root or any private
  corpus directory.
#>
[CmdletBinding()]
param(
  [string]$FontSourceRoot = 'D:\ttfs',
  [bool]$IncludeEntries = $false
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)
$ManagedSources = @(
  [ordered]@{
    label = 'rank1-exact'
    relativePath = 'hwp\MT.TTF'
    sha256 = 'd10509215d923fef07c1f2dffe8ebf55cbca706476559a861dff6f7cf969ff44'
  },
  [ordered]@{
    label = 'rank13-exact'
    relativePath = 'hwp\HMKMM.TTF'
    sha256 = 'd033df71fa41de48905407f7d39aca74ebd9ae87298c5420513acaa187fbefc6'
  },
  [ordered]@{
    label = 'rank7-exact'
    relativePath = 'kopubworld\KoPubWorld Dotum Light.ttf'
    sha256 = '069494cce21a4222c88e537f256b6f46fee209375aba769f82431b2d382bc84f'
  },
  [ordered]@{
    label = 'shared-subst'
    relativePath = 'kopubworld\KoPubWorld Batang Light.ttf'
    sha256 = 'e3ee21a86b6a6728c567a95aaebd8883480f27ce4f230207b0d7266b5cb3fb18'
  }
)
$FontExtensions = @('.hft', '.otf', '.ttc', '.ttf')

function Get-FileDigest([string]$Path) {
  return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-TextDigest([string]$Text) {
  $sha = [System.Security.Cryptography.SHA256]::Create()
  try {
    return ([BitConverter]::ToString(
        $sha.ComputeHash($Utf8NoBom.GetBytes($Text))
      )).Replace('-', '').ToLowerInvariant()
  } finally {
    $sha.Dispose()
  }
}

function Get-OrdinalSortKey([string]$Value) {
  return -join ([Text.Encoding]::UTF8.GetBytes($Value) | ForEach-Object {
      $_.ToString('x2')
    })
}

function Add-ManifestEntry(
  [System.Collections.Generic.List[object]]$Entries,
  [string]$SourceKind,
  [string]$Identity,
  [string]$FilePath
) {
  $exists = Test-Path -LiteralPath $FilePath -PathType Leaf
  $Entries.Add([ordered]@{
      sourceKind = $SourceKind
      identity = $Identity.Replace('\', '/')
      status = if ($exists) { 'observed' } else { 'unavailable' }
      bytes = if ($exists) { [int64](Get-Item -LiteralPath $FilePath).Length } else { $null }
      sha256 = if ($exists) { Get-FileDigest $FilePath } else { $null }
    })
}

function Find-HwpExecutable {
  $programFiles64 = [Environment]::GetEnvironmentVariable('ProgramFiles')
  $programFiles32 = [Environment]::GetEnvironmentVariable('ProgramFiles(x86)')
  $roots = @(
    (Join-Path $programFiles64 'Hnc'),
    (Join-Path $programFiles32 'Hnc')
  ) | Where-Object {
    -not [string]::IsNullOrWhiteSpace($_) -and
    (Test-Path -LiteralPath $_ -PathType Container)
  } | Select-Object -Unique
  $executables = @($roots | ForEach-Object {
      Get-ChildItem -LiteralPath $_ -Filter 'Hwp.exe' -File -Recurse -ErrorAction SilentlyContinue
    } | Sort-Object FullName -Unique)
  if ($executables.Count -ne 1) {
    throw "Hwp executable resolution is not unique: $($executables.Count)"
  }
  return $executables[0]
}

$sourceChecks = @($ManagedSources | ForEach-Object {
    $path = Join-Path $FontSourceRoot $_.relativePath
    $present = Test-Path -LiteralPath $path -PathType Leaf
    [ordered]@{
      label = $_.label
      present = $present
      hashMatch = ($present -and (Get-FileDigest $path) -eq $_.sha256)
    }
  })

$entries = New-Object 'System.Collections.Generic.List[object]'
$registryRoots = @(
  [ordered]@{
    sourceKind = 'windows-machine-registry'
    key = 'Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts'
    fontRoot = Join-Path $env:WINDIR 'Fonts'
  },
  [ordered]@{
    sourceKind = 'windows-user-registry'
    key = 'Registry::HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts'
    fontRoot = Join-Path $env:LOCALAPPDATA 'Microsoft\Windows\Fonts'
  }
)
foreach ($root in $registryRoots) {
  if (-not (Test-Path -LiteralPath $root.key)) { continue }
  $item = Get-Item -LiteralPath $root.key
  foreach ($name in ($item.GetValueNames() | Sort-Object)) {
    $data = [string]$item.GetValue($name)
    $path = if ([System.IO.Path]::IsPathRooted($data)) {
      $data
    } else {
      Join-Path $root.fontRoot $data
    }
    Add-ManifestEntry $entries $root.sourceKind "$name|$([IO.Path]::GetFileName($data))" $path
  }
}

$hwpExecutable = Find-HwpExecutable
$binRoot = Split-Path -Parent $hwpExecutable.FullName
$officeRoot = Split-Path -Parent $binRoot
$hwpRoots = @(
  [ordered]@{
    sourceKind = 'hancom-bin-shared-fonts'
    root = Join-Path $binRoot 'Shared\Fonts'
  },
  [ordered]@{
    sourceKind = 'hancom-office-shared-fonts'
    root = Join-Path $officeRoot 'Shared\Fonts'
  }
)
foreach ($root in $hwpRoots) {
  if (-not (Test-Path -LiteralPath $root.root -PathType Container)) { continue }
  $prefixLength = $root.root.TrimEnd('\').Length + 1
  Get-ChildItem -LiteralPath $root.root -File -Recurse |
    Where-Object { $FontExtensions -contains $_.Extension.ToLowerInvariant() } |
    Sort-Object FullName |
    ForEach-Object {
      $relative = $_.FullName.Substring($prefixLength)
      Add-ManifestEntry $entries $root.sourceKind $relative $_.FullName
    }
}

$orderedEntries = @($entries | Sort-Object @{
    Expression = {
      Get-OrdinalSortKey ($_.sourceKind + [char]0 + $_.identity)
    }
  })
$manifest = [ordered]@{
  schemaVersion = 1
  kind = 'font-oracle-ambient-font-manifest'
  issue = 4963
  entryCount = $orderedEntries.Count
  entries = $orderedEntries
}
$managedHashes = @($ManagedSources | ForEach-Object sha256)
$projectionEntries = @($orderedEntries | Where-Object {
    $null -eq $_.sha256 -or $managedHashes -notcontains $_.sha256
  })
$projection = [ordered]@{
  schemaVersion = 1
  kind = 'font-oracle-unrelated-font-projection'
  issue = 4963
  entryCount = $projectionEntries.Count
  entries = $projectionEntries
}

$summary = [ordered]@{
  schemaVersion = 1
  kind = 'font-oracle-stage4-guest-manifest-summary'
  issue = 4963
  manifestSha256 = Get-TextDigest ($manifest | ConvertTo-Json -Depth 8 -Compress)
  manifestEntryCount = $orderedEntries.Count
  unrelatedProjectionSha256 = Get-TextDigest (
    $projection | ConvertTo-Json -Depth 8 -Compress
  )
  unrelatedProjectionEntryCount = $projectionEntries.Count
  sourcesReady = @($sourceChecks | Where-Object {
      -not $_.present -or -not $_.hashMatch
    }).Count -eq 0
  sourceChecks = $sourceChecks
  managedInstalledByExactBytes = @($orderedEntries | Where-Object {
      $null -ne $_.sha256 -and $managedHashes -contains $_.sha256
    } | ForEach-Object sha256 | Select-Object -Unique)
  hwpExecutableSha256 = Get-FileDigest $hwpExecutable.FullName
  hwpProcessCount = @(Get-Process -Name Hwp -ErrorAction SilentlyContinue).Count
  privacy = [ordered]@{
    absolutePathIncluded = $false
    privateCorpusAccessed = $false
    fontBytesIncluded = $false
  }
}
if ($IncludeEntries) {
  $summary.manifestEntries = $orderedEntries
  $summary.unrelatedProjectionEntries = $projectionEntries
}
$summary | ConvertTo-Json -Depth 7 -Compress
