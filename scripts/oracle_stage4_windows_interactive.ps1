<#
.SYNOPSIS
  Run an Issue #4963 Stage W5-4 canary in an interactive Windows logon session.

.DESCRIPTION
  This wrapper is staged through Hyper-V PowerShell Direct and launched by a
  one-time Scheduled Task under the already logged-on user. It feature-detects
  every requested font name, then opens the byte-frozen HWPX and exports PDF
  even when the document face does not survive readback. A selection mismatch
  is evidence, not an automation failure.

  Font installation, removal, ambient-manifest validation, snapshot restore,
  and output retrieval remain responsibilities of the external control plane.
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)][string]$Source,
  [Parameter(Mandatory = $true)][string]$PdfOutput,
  [Parameter(Mandatory = $true)][string]$ResultOutput,
  [Parameter(Mandatory = $true)][string]$DocumentFace,
  [Parameter(Mandatory = $true)][ValidateRange(1, 17)][int]$QueueRank,
  [Parameter(Mandatory = $true)][ValidatePattern('^[0-9a-fA-F]{64}$')]
  [string]$ExpectedSourceSha256,
  [string[]]$ProbeFaces = @(),
  [string[]]$FontResourceFiles = @(),
  [string]$SecurityModuleName = 'FilePathCheckerModuleExample',
  [int]$MessageBoxMode = 0x00020000
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)

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

function Set-AndReadbackFont($Hwp, [string]$Face) {
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
$sourceSha256 = Get-Sha256 $sourcePath
if ($sourceSha256 -ne $ExpectedSourceSha256.ToLowerInvariant()) {
  throw "Source SHA-256 mismatch: $sourceSha256"
}
$pdfPath = Resolve-OutputPath $PdfOutput
$resultPath = Resolve-OutputPath $ResultOutput
Ensure-Parent $pdfPath
Ensure-Parent $resultPath
foreach ($path in @($pdfPath, $resultPath)) {
  if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force }
}

$startedAt = (Get-Date).ToUniversalTime().ToString('o')
$result = $null
$exitCode = 0
$hwp = $null
$fontSelections = @()
$fontResourceCounts = @()
$securityModuleRegistered = $false
try {
  if (-not ('RhwpStage4FontNative' -as [type])) {
    Add-Type @'
using System;
using System.Runtime.InteropServices;

public static class RhwpStage4FontNative {
    [DllImport("gdi32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern int AddFontResourceEx(
        string fileName, uint flags, IntPtr reserved);

    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr SendMessageTimeout(
        IntPtr window, uint message, UIntPtr wParam, IntPtr lParam,
        uint flags, uint timeout, out UIntPtr result);
}
'@
  }
  $expandedFontResources = @($FontResourceFiles | ForEach-Object { $_ -split '\|' })
  foreach ($fontResource in $expandedFontResources) {
    if ([string]::IsNullOrWhiteSpace($fontResource)) { continue }
    $fontPath = (Resolve-Path -LiteralPath $fontResource).Path
    $count = [RhwpStage4FontNative]::AddFontResourceEx(
      $fontPath,
      0,
      [IntPtr]::Zero
    )
    if ($count -lt 1) {
      throw 'Interactive AddFontResourceEx did not load a managed font.'
    }
    $fontResourceCounts += $count
  }
  if ($fontResourceCounts.Count -gt 0) {
    $broadcastResult = [UIntPtr]::Zero
    [void][RhwpStage4FontNative]::SendMessageTimeout(
      [IntPtr]0xffff,
      0x001d,
      [UIntPtr]::Zero,
      [IntPtr]::Zero,
      0x0002,
      5000,
      [ref]$broadcastResult
    )
    Start-Sleep -Milliseconds 500
  }

  $hwp = New-Object -ComObject HWPFrame.HwpObject
  $hwp.SetMessageBoxMode($MessageBoxMode) | Out-Null
  $securityModuleRegistered = [bool]$hwp.RegisterModule(
    'FilePathCheckDLL',
    $SecurityModuleName
  )
  if (-not $securityModuleRegistered) {
    throw "Security module registration failed: $SecurityModuleName"
  }

  # Windows PowerShell -File cannot bind a native command-line argument to a
  # string array reliably. Accept a pipe-delimited item as well as a real
  # array so the one-time Scheduled Task can preserve localized face names.
  $expandedProbeFaces = @($ProbeFaces | ForEach-Object { $_ -split '\|' })
  $faces = @($DocumentFace) + $expandedProbeFaces
  $faces = @($faces | Where-Object {
      -not [string]::IsNullOrWhiteSpace($_)
    } | Select-Object -Unique)
  $hwp.HAction.Run('FileNew') | Out-Null
  $fontSelections = @($faces | ForEach-Object {
      Set-AndReadbackFont $hwp $_
    })

  $hwp.Clear(1) | Out-Null
  $opened = [bool]$hwp.Open($sourcePath, '', '')
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

  $documentSelection = @($fontSelections | Where-Object {
      $_.requestedFace -eq $DocumentFace
    })[0]
  $result = [ordered]@{
    schemaVersion = 1
    kind = 'font-oracle-stage4-interactive-result'
    issue = 4963
    status = 'observed'
    queueRank = $QueueRank
    documentFace = $DocumentFace
    inputSha256 = $sourceSha256
    documentFaceSelectable = [bool]$documentSelection.exact
    fontSelections = $fontSelections
    featureDetection = [ordered]@{
      opened = $opened
      pageCount = $pageCount
      textLength = $textLength
    }
    export = [ordered]@{
      pdfSha256 = Get-Sha256 $pdfPath
      pdfBytes = [int64](Get-Item -LiteralPath $pdfPath).Length
    }
    environment = [ordered]@{
      currentCulture = (Get-Culture).Name
      currentUICulture = (Get-UICulture).Name
      systemLocale = (Get-WinSystemLocale).Name
      hancomVersion = [string]$hwp.Version
      securityModuleRegistered = $securityModuleRegistered
      processReset = $true
      fontCacheAction = if ($fontResourceCounts.Count -gt 0) {
        'font-cache-refresh-and-process-reset'
      } else {
        'process-reset'
      }
      fontResourceCounts = $fontResourceCounts
    }
    startedAt = $startedAt
    finishedAt = (Get-Date).ToUniversalTime().ToString('o')
    privacy = [ordered]@{
      absolutePathIncluded = $false
      privateCorpusAccessed = $false
      fontBytesIncluded = $false
    }
  }
} catch {
  $exitCode = 1
  $result = [ordered]@{
    schemaVersion = 1
    kind = 'font-oracle-stage4-interactive-result'
    issue = 4963
    status = 'failed'
    queueRank = $QueueRank
    documentFace = $DocumentFace
    inputSha256 = $sourceSha256
    errorType = $_.Exception.GetType().FullName
    hresult = $_.Exception.HResult
    fullyQualifiedErrorId = $_.FullyQualifiedErrorId
    fontSelections = $fontSelections
    securityModuleRegistered = $securityModuleRegistered
    fontResourceCounts = $fontResourceCounts
    startedAt = $startedAt
    finishedAt = (Get-Date).ToUniversalTime().ToString('o')
    privacy = [ordered]@{
      absolutePathIncluded = $false
      privateCorpusAccessed = $false
      fontBytesIncluded = $false
    }
  }
} finally {
  if ($null -ne $hwp) {
    try { $hwp.Clear(1) | Out-Null } catch { }
    try { $hwp.Quit() | Out-Null } catch { }
    try {
      [Runtime.InteropServices.Marshal]::FinalReleaseComObject($hwp) | Out-Null
    } catch { }
  }
}

[System.IO.File]::WriteAllText(
  $resultPath,
  ($result | ConvertTo-Json -Depth 8 -Compress),
  $Utf8NoBom
)
$result | ConvertTo-Json -Depth 8 -Compress
exit $exitCode
