<#
.SYNOPSIS
  Issue #4963 Stage W5-3 read-only Hancom Oracle canary.

.DESCRIPTION
  Captures a path-free ambient font manifest, verifies an already-installed
  exact font, opens one deterministic HWPX fixture in a fresh HwpObject, and
  exports one PDF. The script never installs or removes fonts and never stops
  unrelated Hwp.exe processes.

  FontManifestOutput is local-only evidence. Keep it outside the repository
  with owner-only permissions; only its SHA-256 belongs in a public profile.
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)][string]$Source,
  [Parameter(Mandatory = $true)][string]$PdfOutput,
  [Parameter(Mandatory = $true)][string]$EvidenceOutput,
  [Parameter(Mandatory = $true)][string]$FontManifestOutput,
  [Parameter(Mandatory = $true)][string]$RequestedFace,
  [Parameter(Mandatory = $true)][ValidateRange(1, 17)][int]$QueueRank,
  [Parameter(Mandatory = $true)][string]$InstalledFontFile,
  [Parameter(Mandatory = $true)][ValidatePattern('^[0-9a-fA-F]{64}$')]
  [string]$ExpectedSourceSha256,
  [Parameter(Mandatory = $true)][ValidatePattern('^[0-9a-fA-F]{64}$')]
  [string]$ExpectedInstalledFontSha256,
  [string]$SecurityModuleName = 'FilePathCheckerModuleExample',
  [int]$MessageBoxMode = 0x00020000
)

$ErrorActionPreference = 'Stop'
$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)
$FontExtensions = @('.hft', '.otf', '.ttc', '.ttf')

function Resolve-OutputPath([string]$Value) {
  if ([System.IO.Path]::IsPathRooted($Value)) {
    return [System.IO.Path]::GetFullPath($Value)
  }
  return [System.IO.Path]::GetFullPath((Join-Path (Get-Location).Path $Value))
}

function Ensure-Parent([string]$Path) {
  $parent = Split-Path -Parent $Path
  if ($parent -and -not (Test-Path -LiteralPath $parent)) {
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
  }
}

function Get-Sha256([string]$Path) {
  return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Add-FontEntry(
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
      bytes = if ($exists) { (Get-Item -LiteralPath $FilePath).Length } else { $null }
      sha256 = if ($exists) { Get-Sha256 $FilePath } else { $null }
    })
}

function Get-HwpExecutablePath {
  $process = Get-CimInstance Win32_Process -Filter "Name = 'Hwp.exe'" |
    Where-Object { $_.CommandLine -match '-Automation' -and $_.ExecutablePath } |
    Sort-Object CreationDate -Descending |
    Select-Object -First 1
  if ($null -eq $process) {
    throw 'Fresh Hwp automation process could not be identified.'
  }
  return [string]$process.ExecutablePath
}

function Write-AmbientFontManifest(
  [string]$OutputPath,
  [string]$HwpExecutablePath
) {
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
      $resolved = if ([System.IO.Path]::IsPathRooted($data)) {
        $data
      } else {
        Join-Path $root.fontRoot $data
      }
      $leaf = [System.IO.Path]::GetFileName($data)
      Add-FontEntry $entries $root.sourceKind "$name|$leaf" $resolved
    }
  }

  $binRoot = Split-Path -Parent $HwpExecutablePath
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
    $prefixLength = $root.root.TrimEnd('\\').Length + 1
    Get-ChildItem -LiteralPath $root.root -File -Recurse |
      Where-Object { $FontExtensions -contains $_.Extension.ToLowerInvariant() } |
      Sort-Object FullName |
      ForEach-Object {
        $relative = $_.FullName.Substring($prefixLength).Replace('\', '/')
        Add-FontEntry $entries $root.sourceKind $relative $_.FullName
      }
  }

  $orderedEntries = @($entries | Sort-Object sourceKind, identity)
  $manifest = [ordered]@{
    schemaVersion = 1
    kind = 'font-oracle-ambient-font-manifest'
    issue = 4963
    entryCount = $orderedEntries.Count
    entries = $orderedEntries
  }
  [System.IO.File]::WriteAllText(
    $OutputPath,
    ($manifest | ConvertTo-Json -Depth 8 -Compress),
    $Utf8NoBom
  )
  return Get-Sha256 $OutputPath
}

function Set-AndReadbackFont($Hwp, [string]$Face) {
  $Hwp.HAction.Run('FileNew') | Out-Null
  $charShape = $Hwp.HParameterSet.HCharShape
  $Hwp.HAction.GetDefault('CharShape', $charShape.HSet) | Out-Null
  foreach ($language in @('Hangul', 'Latin', 'Hanja', 'Japanese', 'Other', 'Symbol', 'User')) {
    $charShape.('FaceName' + $language) = $Face
    $charShape.('FontType' + $language) = 1
    $charShape.('Ratio' + $language) = 100
    $charShape.('Spacing' + $language) = 0
  }
  $Hwp.HAction.Execute('CharShape', $charShape.HSet) | Out-Null
  $Hwp.HAction.GetDefault('CharShape', $charShape.HSet) | Out-Null
  return [ordered]@{
    requestedFace = $Face
    requestedFontType = 1
    readbackFace = [string]$charShape.FaceNameHangul
    readbackFontType = [int]$charShape.FontTypeHangul
    exact = (
      [string]$charShape.FaceNameHangul -eq $Face -and
      [int]$charShape.FontTypeHangul -eq 1
    )
  }
}

$sourcePath = (Resolve-Path -LiteralPath $Source).Path
$installedFontPath = (Resolve-Path -LiteralPath $InstalledFontFile).Path
$pdfPath = Resolve-OutputPath $PdfOutput
$evidencePath = Resolve-OutputPath $EvidenceOutput
$fontManifestPath = Resolve-OutputPath $FontManifestOutput
foreach ($path in @($pdfPath, $evidencePath, $fontManifestPath)) { Ensure-Parent $path }

$sourceSha256 = Get-Sha256 $sourcePath
if ($sourceSha256 -ne $ExpectedSourceSha256.ToLowerInvariant()) {
  throw "Source SHA-256 mismatch: $sourceSha256"
}
$installedFontSha256 = Get-Sha256 $installedFontPath
if ($installedFontSha256 -ne $ExpectedInstalledFontSha256.ToLowerInvariant()) {
  throw "Installed font SHA-256 mismatch: $installedFontSha256"
}
foreach ($path in @($pdfPath, $evidencePath, $fontManifestPath)) {
  if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force }
}

$startedAt = (Get-Date).ToUniversalTime().ToString('o')
$hwp = $null
try {
  $hwp = New-Object -ComObject HWPFrame.HwpObject
  $hwp.SetMessageBoxMode($MessageBoxMode) | Out-Null
  $registered = $hwp.RegisterModule('FilePathCheckDLL', $SecurityModuleName)
  if (-not $registered) { throw "Security module registration failed: $SecurityModuleName" }

  $hwpExecutablePath = Get-HwpExecutablePath
  $ambientManifestSha256 = Write-AmbientFontManifest $fontManifestPath $hwpExecutablePath
  $selection = Set-AndReadbackFont $hwp $RequestedFace
  if (-not $selection.exact) {
    throw "Requested exact font did not survive readback: $RequestedFace"
  }

  $hwp.Clear(1) | Out-Null
  $opened = $hwp.Open($sourcePath, '', '')
  if (-not $opened) { throw 'HWPX feature detection failed: Open returned false.' }
  $pageCount = [int]$hwp.PageCount
  $textLength = ([string]$hwp.GetTextFile('TEXT', '')).Length
  if ($pageCount -lt 1 -or $textLength -lt 1) {
    throw "Silent empty-open guard failed: PageCount=$pageCount TextLength=$textLength"
  }

  $action = $hwp.CreateAction('FileSaveAsPdf')
  $set = $action.CreateSet()
  $action.GetDefault($set) | Out-Null
  $set.SetItem('FileName', $pdfPath)
  $set.SetItem('Format', 'PDF')
  $set.SetItem('Attributes', 0)
  if (-not $action.Execute($set)) { throw 'FileSaveAsPdf returned false.' }

  $deadline = (Get-Date).AddSeconds(60)
  $lastSize = -1
  while ((Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 500
    if (-not (Test-Path -LiteralPath $pdfPath -PathType Leaf)) { continue }
    $size = (Get-Item -LiteralPath $pdfPath).Length
    if ($size -gt 0 -and $size -eq $lastSize) { break }
    $lastSize = $size
  }
  if (-not (Test-Path -LiteralPath $pdfPath -PathType Leaf)) {
    throw 'PDF output was not created.'
  }

  $finishedAt = (Get-Date).ToUniversalTime().ToString('o')
  $evidence = [ordered]@{
    schemaVersion = 1
    kind = 'font-oracle-stage3-windows-canary-evidence'
    issue = 4963
    candidate = [ordered]@{
      queueRank = $QueueRank
      documentFace = $RequestedFace
    }
    input = [ordered]@{
      sourceFormat = 'hwpx'
      sha256 = $sourceSha256
    }
    environment = [ordered]@{
      os = (Get-CimInstance Win32_OperatingSystem).Caption
      osVersion = [Environment]::OSVersion.VersionString
      locale = (Get-Culture).Name
      hancomVersion = [string]$hwp.Version
      hwpExecutableSha256 = Get-Sha256 $hwpExecutablePath
      ambientFontManifestSha256 = $ambientManifestSha256
      processReset = $true
      securityModuleRegistered = [bool]$registered
    }
    fontState = [ordered]@{
      installedFontSha256 = $installedFontSha256
      selection = $selection
    }
    featureDetection = [ordered]@{
      opened = [bool]$opened
      pageCount = $pageCount
      textLength = $textLength
    }
    export = [ordered]@{
      route = 'HwpObject FileSaveAsPdf action'
      pdfSha256 = Get-Sha256 $pdfPath
      pdfBytes = (Get-Item -LiteralPath $pdfPath).Length
    }
    execution = [ordered]@{
      startedAt = $startedAt
      finishedAt = $finishedAt
      repeatIndex = 1
    }
    privacy = [ordered]@{
      fontBytesEmbedded = $false
      privateDocumentIdentityIncluded = $false
      absoluteFontPathIncluded = $false
    }
  }
  [System.IO.File]::WriteAllText(
    $evidencePath,
    ($evidence | ConvertTo-Json -Depth 10 -Compress),
    $Utf8NoBom
  )
  Write-Output ($evidence | ConvertTo-Json -Depth 10 -Compress)
} finally {
  if ($null -ne $hwp) {
    try { $hwp.Clear(1) | Out-Null } catch { }
    try { $hwp.Quit() | Out-Null } catch { }
    try { [System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($hwp) | Out-Null } catch { }
  }
}
