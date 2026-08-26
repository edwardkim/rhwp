<#
.SYNOPSIS
  Configure one disposable Windows guest font state for Issue #4963.

.DESCRIPTION
  Installs only the font files declared by a local-only JSON state spec. The
  script is deliberately one-way: removal is performed only by restoring the
  externally controlled Hyper-V checkpoint. It refuses physical hosts, a
  running Hwp.exe, hash drift, pre-existing destination files, and undeclared
  physical states.

  The state spec is local-only because it contains guest absolute paths. The
  JSON result is path-free and can be retained as execution evidence.
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)][string]$StateSpec,
  [Parameter(Mandatory = $true)][string]$ResultOutput,
  [Parameter(Mandatory = $true)][switch]$CheckpointRestoreAttested
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)

function Get-Sha256([string]$Path) {
  return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-OutputPath([string]$Value) {
  if ([System.IO.Path]::IsPathRooted($Value)) {
    return [System.IO.Path]::GetFullPath($Value)
  }
  return [System.IO.Path]::GetFullPath((Join-Path (Get-Location).Path $Value))
}

if (-not $CheckpointRestoreAttested) {
  throw 'External checkpoint restore attestation is required.'
}

$computer = Get-CimInstance Win32_ComputerSystem
if (
  [string]$computer.Manufacturer -notmatch '^Microsoft Corporation$' -or
  [string]$computer.Model -notmatch '^Virtual Machine$'
) {
  throw 'Font mutation is allowed only inside an attested Hyper-V guest.'
}
if (@(Get-Process -Name Hwp -ErrorAction SilentlyContinue).Count -ne 0) {
  throw 'Hwp.exe must be stopped before configuring a font state.'
}

$specPath = (Resolve-Path -LiteralPath $StateSpec).Path
$spec = Get-Content -LiteralPath $specPath -Raw -Encoding UTF8 | ConvertFrom-Json
if (
  $spec.schemaVersion -ne 1 -or
  $spec.kind -ne 'font-oracle-hyperv-state-spec' -or
  $spec.issue -ne 4963
) {
  throw 'State spec identity is invalid.'
}
$allowedStates = @('exact-only', 'subst-only', 'none-related')
if ($allowedStates -notcontains [string]$spec.physicalState) {
  throw 'Physical state is not contract-declared.'
}
$fonts = @($spec.fonts)
$expectedCount = if ($spec.physicalState -eq 'none-related') { 0 } else { 1 }
if ($fonts.Count -ne $expectedCount) {
  throw "Physical state requires $expectedCount managed font(s)."
}

$fontRegistry = 'Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts'
$installed = @()
foreach ($font in $fonts) {
  $source = (Resolve-Path -LiteralPath ([string]$font.source)).Path
  $expected = ([string]$font.sha256).ToLowerInvariant()
  if ($expected -notmatch '^[0-9a-f]{64}$') {
    throw 'Managed font SHA-256 is invalid.'
  }
  $observed = Get-Sha256 $source
  if ($observed -ne $expected) {
    throw "Managed font SHA-256 mismatch: $observed"
  }
  $extension = [System.IO.Path]::GetExtension($source).ToLowerInvariant()
  if (@('.otf', '.ttf') -notcontains $extension) {
    throw 'Only OTF and TTF managed font sources are accepted.'
  }
  $short = $expected.Substring(0, 16)
  $leaf = "rhwp-4963-$short$extension"
  $destination = Join-Path (Join-Path $env:WINDIR 'Fonts') $leaf
  $technology = if ($extension -eq '.otf') { 'OpenType' } else { 'TrueType' }
  $registryName = "rhwp-4963-$short ($technology)"
  if (Test-Path -LiteralPath $destination) {
    throw 'Managed font destination already exists; restore the baseline checkpoint.'
  }
  $existing = (Get-ItemProperty -LiteralPath $fontRegistry -Name $registryName -ErrorAction SilentlyContinue)
  if ($null -ne $existing) {
    throw 'Managed font registry value already exists; restore the baseline checkpoint.'
  }
  Copy-Item -LiteralPath $source -Destination $destination
  $registryParameters = @{
    LiteralPath = $fontRegistry
    Name = $registryName
    Value = $leaf
    PropertyType = 'String'
  }
  New-ItemProperty @registryParameters | Out-Null
  $installed += [ordered]@{
    sha256 = $expected
    destinationLeaf = $leaf
    registryValueName = $registryName
  }
}

if ($installed.Count -gt 0) {
  if (-not ('RhwpFontStateNative' -as [type])) {
    Add-Type @'
using System;
using System.Runtime.InteropServices;

public static class RhwpFontStateNative {
    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr SendMessageTimeout(
        IntPtr window, uint message, UIntPtr wParam, IntPtr lParam,
        uint flags, uint timeout, out UIntPtr result);
}
'@
  }
  $broadcastResult = [UIntPtr]::Zero
  [void][RhwpFontStateNative]::SendMessageTimeout(
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

$result = [ordered]@{
  schemaVersion = 1
  kind = 'font-oracle-hyperv-font-state-result'
  issue = 4963
  physicalState = [string]$spec.physicalState
  installedFontSha256 = @($installed | ForEach-Object sha256)
  installedCount = $installed.Count
  hwpProcessCount = @(Get-Process -Name Hwp -ErrorAction SilentlyContinue).Count
  checkpointRestoreAttested = $true
  privacy = [ordered]@{
    absolutePathIncluded = $false
    fontBytesIncluded = $false
    privateCorpusAccessed = $false
  }
}
$outputPath = Get-OutputPath $ResultOutput
$parent = Split-Path -Parent $outputPath
if ($parent -and -not (Test-Path -LiteralPath $parent)) {
  New-Item -ItemType Directory -Force -Path $parent | Out-Null
}
[System.IO.File]::WriteAllText(
  $outputPath,
  ($result | ConvertTo-Json -Depth 6 -Compress),
  $Utf8NoBom
)
$result | ConvertTo-Json -Depth 6 -Compress
