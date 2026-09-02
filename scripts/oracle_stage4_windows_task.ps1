<#
.SYNOPSIS
  Bind a local-only JSON state spec to the interactive Issue #4963 runner.

.DESCRIPTION
  This wrapper is the Scheduled Task entry point. JSON keeps localized font
  names and arrays out of cmd.exe/Task Scheduler argument tokenization. It does
  not install fonts or restore checkpoints.
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)][string]$StateSpec
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$specPath = (Resolve-Path -LiteralPath $StateSpec).Path
$spec = Get-Content -LiteralPath $specPath -Raw -Encoding UTF8 | ConvertFrom-Json
if (
  $spec.schemaVersion -ne 1 -or
  $spec.kind -ne 'font-oracle-hyperv-task-spec' -or
  $spec.issue -notin @(4963, 4968)
) {
  throw 'Interactive task spec identity is invalid.'
}
$runner = (Resolve-Path -LiteralPath ([string]$spec.runner)).Path
$arguments = @{
  Issue = [int]$spec.issue
  Source = [string]$spec.source
  PdfOutput = [string]$spec.pdfOutput
  ResultOutput = [string]$spec.resultOutput
  DocumentFace = [string]$spec.documentFace
  QueueRank = [int]$spec.queueRank
  ExpectedSourceSha256 = [string]$spec.expectedSourceSha256
  ProbeFaces = @($spec.probeFaces)
  FontResourceFiles = @($spec.fontResourceFiles)
  SecurityModuleName = [string]$spec.securityModuleName
}
if ($spec.PSObject.Properties.Name -contains 'hwpmlOutput') {
  $arguments.HwpmlOutput = [string]$spec.hwpmlOutput
}

& $runner @arguments
exit $LASTEXITCODE
