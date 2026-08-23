<#
.SYNOPSIS
  Run one Issue #4963 physical font state under an exact Hyper-V checkpoint.

.DESCRIPTION
  This Windows-host control plane verifies raw VM/checkpoint identities,
  restores a Standard checkpoint before and after the run, stages the tracked
  guest helpers through PowerShell Direct, runs HWP in an existing interactive
  user token, retrieves path-free evidence, and verifies baseline recovery.

  Invoke this script once for each of exact-only, subst-only, and none-related.
  Font removal is intentionally absent: only checkpoint restore may remove a
  managed state.
#>
[CmdletBinding(SupportsShouldProcess = $true, ConfirmImpact = 'High')]
param(
  [Parameter(Mandatory = $true)][guid]$ExpectedVmId,
  [Parameter(Mandatory = $true)][guid]$ExpectedCheckpointId,
  [Parameter(Mandatory = $true)][string]$CredentialFile,
  [Parameter(Mandatory = $true)][string]$HostRepoRoot,
  [Parameter(Mandatory = $true)][string]$HostFixture,
  [Parameter(Mandatory = $true)][string]$HostOutputRoot,
  [Parameter(Mandatory = $true)][ValidateSet('exact-only', 'subst-only', 'none-related')]
  [string]$PhysicalState,
  [Parameter(Mandatory = $true)][ValidateRange(1, 17)][int]$QueueRank,
  [Parameter(Mandatory = $true)][string]$DocumentFace,
  [Parameter(Mandatory = $true)][ValidatePattern('^[0-9a-fA-F]{64}$')]
  [string]$ExpectedFixtureSha256,
  [Parameter(Mandatory = $true)][ValidatePattern('^[0-9a-fA-F]{64}$')]
  [string]$BaselineManifestSha256,
  [Parameter(Mandatory = $true)][ValidatePattern('^[0-9a-fA-F]{64}$')]
  [string]$BaselineUnrelatedProjectionSha256,
  [Parameter(Mandatory = $true)][string]$GuestFontSourceRoot,
  [string]$ManagedFontSource,
  [ValidatePattern('^(?:|[0-9a-fA-F]{64})$')][string]$ManagedFontSha256 = '',
  [string[]]$ProbeFaces = @(),
  [string]$SecurityModuleName = 'FilePathCheckerModuleExample',
  [string]$GuestRoot = 'C:\rhwp-oracle-reproduction',
  [ValidateRange(30, 600)][int]$TimeoutSeconds = 180,
  [Parameter(Mandatory = $true)][switch]$CheckpointRestoreApproved
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)

function Get-Sha256([string]$Path) {
  return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Test-ExactStringArray([object[]]$Actual, [object[]]$Expected) {
  $left = @($Actual | ForEach-Object { [string]$_ } | Sort-Object)
  $right = @($Expected | ForEach-Object { [string]$_ } | Sort-Object)
  if ($left.Count -ne $right.Count) { return $false }
  for ($index = 0; $index -lt $left.Count; $index += 1) {
    if ($left[$index] -cne $right[$index]) { return $false }
  }
  return $true
}

function Assert-Identity {
  $vm = Get-VM -Id $ExpectedVmId -ErrorAction Stop
  if ($vm.Id -ne $ExpectedVmId) { throw 'VM identity drift.' }
  if ($vm.CheckpointType.ToString() -ne 'Standard') {
    throw 'VM checkpoint type must be Standard.'
  }
  if ($vm.AutomaticCheckpointsEnabled) {
    throw 'Automatic checkpoints must be disabled.'
  }
  $matches = @(Get-VMSnapshot -VM $vm | Where-Object Id -eq $ExpectedCheckpointId)
  if ($matches.Count -ne 1) { throw 'Checkpoint identity is not unique.' }
  if ($matches[0].SnapshotType.ToString() -ne 'Standard') {
    throw 'Baseline snapshot type must be Standard.'
  }
  return [ordered]@{ vm = $vm; checkpoint = $matches[0] }
}

function Restore-Baseline {
  $identity = Assert-Identity
  $identity.checkpoint | Restore-VMCheckpoint -Confirm:$false
  $vm = Get-VM -Id $ExpectedVmId
  if ($vm.State -eq 'Off') { Start-VM -VM $vm | Out-Null }
}

function New-DirectSession {
  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
  while ((Get-Date) -lt $deadline) {
    try {
      return New-PSSession -VMId $ExpectedVmId -Credential $script:Credential -ErrorAction Stop
    } catch {
      Start-Sleep -Seconds 2
    }
  }
  throw 'PowerShell Direct session timeout.'
}

function Get-InteractiveIdentity($Session) {
  $identities = @(Invoke-Command -Session $Session -ScriptBlock {
      Get-CimInstance Win32_Process |
        Where-Object Name -eq 'explorer.exe' |
        ForEach-Object {
          $owner = Invoke-CimMethod -InputObject $_ -MethodName GetOwner
          if ($owner.ReturnValue -eq 0) {
            [ordered]@{
              userId = if ([string]::IsNullOrWhiteSpace([string]$owner.Domain)) {
                [string]$owner.User
              } else {
                [string]$owner.Domain + '\' + [string]$owner.User
              }
              sessionId = [int]$_.SessionId
            }
          }
        }
    })
  $unique = @($identities | Sort-Object userId, sessionId -Unique)
  if ($unique.Count -ne 1 -or [string]::IsNullOrWhiteSpace($unique[0].userId)) {
    throw 'Exactly one interactive Explorer token is required.'
  }
  return $unique[0]
}

function Invoke-GuestManifest($Session) {
  $parameters = @{
    Session = $Session
    FilePath = $script:HostManifestScript
    ArgumentList = @($GuestFontSourceRoot, $false)
  }
  $json = Invoke-Command @parameters
  return ($json | Out-String).Trim() | ConvertFrom-Json
}

function Write-GuestJson($Session, [string]$Path, [object]$Value) {
  $json = $Value | ConvertTo-Json -Depth 8 -Compress
  Invoke-Command -Session $Session -ScriptBlock {
    param($Destination, $Json)
    $parent = Split-Path -Parent $Destination
    if (-not (Test-Path -LiteralPath $parent)) {
      New-Item -ItemType Directory -Force -Path $parent | Out-Null
    }
    $utf8 = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($Destination, $Json, $utf8)
  } -ArgumentList $Path,$json
}

function Copy-ToGuest($Session, [string]$Source, [string]$Destination) {
  Copy-Item -LiteralPath $Source -Destination $Destination -ToSession $Session -Force
}

function Save-Json([string]$Path, [object]$Value) {
  [System.IO.File]::WriteAllText(
    $Path,
    ($Value | ConvertTo-Json -Depth 8 -Compress),
    $Utf8NoBom
  )
}

if (-not $CheckpointRestoreApproved) {
  throw 'Explicit checkpoint restore approval is required.'
}
Import-Module Hyper-V -ErrorAction Stop
$script:Credential = Import-Clixml -LiteralPath $CredentialFile
if ($null -eq $script:Credential -or [string]::IsNullOrWhiteSpace($script:Credential.UserName)) {
  throw 'Credential import failed.'
}

$expectedCount = if ($PhysicalState -eq 'none-related') { 0 } else { 1 }
$managedSources = @()
if (-not [string]::IsNullOrWhiteSpace($ManagedFontSource)) {
  $managedSources = @([ordered]@{
      source = $ManagedFontSource
      sha256 = $ManagedFontSha256.ToLowerInvariant()
    })
}
if ($managedSources.Count -ne $expectedCount) {
  throw "Physical state requires $expectedCount managed font source(s)."
}

$repo = (Resolve-Path -LiteralPath $HostRepoRoot).Path
$fixture = (Resolve-Path -LiteralPath $HostFixture).Path
if ((Get-Sha256 $fixture) -ne $ExpectedFixtureSha256.ToLowerInvariant()) {
  throw 'Host fixture SHA-256 mismatch.'
}
$hostScripts = @(
  'oracle_stage4_windows_manifest.ps1',
  'oracle_stage4_windows_font_state.ps1',
  'oracle_stage4_windows_interactive.ps1',
  'oracle_stage4_windows_task.ps1'
)
foreach ($name in $hostScripts) {
  if (-not (Test-Path -LiteralPath (Join-Path (Join-Path $repo 'scripts') $name) -PathType Leaf)) {
    throw "Tracked guest helper is missing: $name"
  }
}
$script:HostManifestScript = Join-Path (Join-Path $repo 'scripts') 'oracle_stage4_windows_manifest.ps1'

$outputRoot = [System.IO.Path]::GetFullPath($HostOutputRoot)
$stateDirectoryName = "rank$QueueRank-$PhysicalState"
$stateOutput = Join-Path $outputRoot $stateDirectoryName
if (Test-Path -LiteralPath $stateOutput) {
  throw 'State output already exists; refusing to overwrite evidence.'
}

if (-not $PSCmdlet.ShouldProcess(
    "$ExpectedVmId/$ExpectedCheckpointId/$PhysicalState",
    'restore checkpoint, mutate disposable guest font state, and restore checkpoint'
  )) {
  return
}
if (-not (Test-Path -LiteralPath $outputRoot)) {
  New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null
}
New-Item -ItemType Directory -Path $stateOutput | Out-Null

$session = $null
$taskName = "rhwp-4963-$PhysicalState-once"
$runError = $null
try {
  Restore-Baseline
  $session = New-DirectSession
  $interactive = Get-InteractiveIdentity $session
  $baselineProbe = Invoke-Command -Session $session -ScriptBlock {
    [ordered]@{
      hwpProcessCount = @(Get-Process Hwp -ErrorAction SilentlyContinue).Count
      oneTimeTaskCount = @(Get-ScheduledTask -TaskName 'rhwp-4963-*' -ErrorAction SilentlyContinue).Count
    }
  }
  if ($baselineProbe.hwpProcessCount -ne 0 -or $baselineProbe.oneTimeTaskCount -ne 0) {
    throw 'Guest baseline contains Hwp.exe or a one-time task.'
  }

  Invoke-Command -Session $session -ScriptBlock {
    param($Root)
    if (Test-Path -LiteralPath $Root) { Remove-Item -LiteralPath $Root -Recurse -Force }
    New-Item -ItemType Directory -Path $Root | Out-Null
  } -ArgumentList $GuestRoot
  foreach ($name in $hostScripts) {
    Copy-ToGuest $session (Join-Path (Join-Path $repo 'scripts') $name) (Join-Path $GuestRoot $name)
  }
  $guestFixture = Join-Path $GuestRoot 'fixture.hwpx'
  Copy-ToGuest $session $fixture $guestFixture

  $baseline = Invoke-GuestManifest $session
  if (
    $baseline.manifestSha256 -ne $BaselineManifestSha256.ToLowerInvariant() -or
    $baseline.unrelatedProjectionSha256 -ne $BaselineUnrelatedProjectionSha256.ToLowerInvariant() -or
    @($baseline.managedInstalledByExactBytes).Count -ne 0 -or
    $baseline.hwpProcessCount -ne 0
  ) {
    throw 'Restored guest does not match the declared baseline.'
  }

  $fontStateSpec = [ordered]@{
    schemaVersion = 1
    kind = 'font-oracle-hyperv-state-spec'
    issue = 4963
    physicalState = $PhysicalState
    fonts = $managedSources
  }
  $fontStateSpecPath = Join-Path $GuestRoot 'font-state-spec.json'
  $fontStateResultPath = Join-Path $GuestRoot 'font-state-result.json'
  Write-GuestJson $session $fontStateSpecPath $fontStateSpec
  $fontState = Invoke-Command -Session $session -ScriptBlock {
    param($ScriptPath, $SpecPath, $ResultPath)
    Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass -Force
    & $ScriptPath -StateSpec $SpecPath -ResultOutput $ResultPath -CheckpointRestoreAttested
  } -ArgumentList (
    (Join-Path $GuestRoot 'oracle_stage4_windows_font_state.ps1'),
    $fontStateSpecPath,
    $fontStateResultPath
  ) | Out-String | ConvertFrom-Json
  if (
    $fontState.physicalState -ne $PhysicalState -or
    $fontState.installedCount -ne $expectedCount -or
    $fontState.hwpProcessCount -ne 0
  ) {
    throw 'Guest font state result is invalid.'
  }

  $stateManifest = Invoke-GuestManifest $session
  $expectedManaged = @($managedSources | ForEach-Object sha256)
  if (
    -not (Test-ExactStringArray -Actual @($stateManifest.managedInstalledByExactBytes) -Expected $expectedManaged) -or
    $stateManifest.unrelatedProjectionSha256 -ne $BaselineUnrelatedProjectionSha256.ToLowerInvariant()
  ) {
    throw 'Guest managed font state or unrelated projection drifted.'
  }

  $pdfPath = Join-Path $GuestRoot "$PhysicalState.pdf"
  $resultPath = Join-Path $GuestRoot "$PhysicalState.interactive.json"
  $taskSpec = [ordered]@{
    schemaVersion = 1
    kind = 'font-oracle-hyperv-task-spec'
    issue = 4963
    runner = Join-Path $GuestRoot 'oracle_stage4_windows_interactive.ps1'
    source = $guestFixture
    pdfOutput = $pdfPath
    resultOutput = $resultPath
    documentFace = $DocumentFace
    queueRank = $QueueRank
    expectedSourceSha256 = $ExpectedFixtureSha256.ToLowerInvariant()
    probeFaces = @($ProbeFaces)
    fontResourceFiles = @($managedSources | ForEach-Object source)
    securityModuleName = $SecurityModuleName
  }
  $taskSpecPath = Join-Path $GuestRoot 'task-spec.json'
  Write-GuestJson $session $taskSpecPath $taskSpec

  Invoke-Command -Session $session -ScriptBlock {
    param($TaskName, $UserId, $TaskScript, $TaskSpec)
    $arguments = '-NoProfile -ExecutionPolicy Bypass -File "' + $TaskScript +
      '" -StateSpec "' + $TaskSpec + '"'
    $action = New-ScheduledTaskAction -Execute 'powershell.exe' -Argument $arguments
    $principal = New-ScheduledTaskPrincipal -UserId $UserId -LogonType Interactive -RunLevel Highest
    Register-ScheduledTask -TaskName $TaskName -Action $action -Principal $principal -Force | Out-Null
    Start-ScheduledTask -TaskName $TaskName
  } -ArgumentList $taskName,$interactive.userId,(Join-Path $GuestRoot 'oracle_stage4_windows_task.ps1'),$taskSpecPath

  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
  while ((Get-Date) -lt $deadline) {
    $ready = Invoke-Command -Session $session -ScriptBlock {
      param($ResultPath, $PdfPath)
      (Test-Path -LiteralPath $ResultPath -PathType Leaf) -and
        (Test-Path -LiteralPath $PdfPath -PathType Leaf)
    } -ArgumentList $resultPath,$pdfPath
    if ($ready) { break }
    Start-Sleep -Seconds 1
  }
  $run = Invoke-Command -Session $session -ScriptBlock {
    param($ResultPath)
    if (-not (Test-Path -LiteralPath $ResultPath -PathType Leaf)) {
      throw 'Interactive result was not created.'
    }
    Get-Content -LiteralPath $ResultPath -Raw -Encoding UTF8 | ConvertFrom-Json
  } -ArgumentList $resultPath
  if (
    $run.status -ne 'observed' -or
    $run.inputSha256 -ne $ExpectedFixtureSha256.ToLowerInvariant() -or
    $run.featureDetection.opened -ne $true -or
    $run.featureDetection.pageCount -lt 1 -or
    $run.featureDetection.textLength -lt 1 -or
    $run.environment.securityModuleRegistered -ne $true
  ) {
    throw 'Interactive HWP run envelope is invalid.'
  }

  $postRun = Invoke-Command -Session $session -ScriptBlock {
    param($TaskName)
    $deadline = (Get-Date).AddSeconds(30)
    while ((Get-Date) -lt $deadline) {
      $runningHwp = @(Get-Process Hwp -ErrorAction SilentlyContinue).Count
      $taskState = (Get-ScheduledTask -TaskName $TaskName).State.ToString()
      if ($runningHwp -eq 0 -and $taskState -ne 'Running') { break }
      Start-Sleep -Milliseconds 500
    }
    $info = Get-ScheduledTaskInfo -TaskName $TaskName
    [ordered]@{
      hwpProcessCount = @(Get-Process Hwp -ErrorAction SilentlyContinue).Count
      taskResult = [int]$info.LastTaskResult
    }
  } -ArgumentList $taskName
  if ($postRun.hwpProcessCount -ne 0 -or $postRun.taskResult -ne 0) {
    throw 'Interactive task left Hwp.exe or returned a failure code.'
  }

  $afterManifest = Invoke-GuestManifest $session
  if (
    -not (Test-ExactStringArray -Actual @($afterManifest.managedInstalledByExactBytes) -Expected $expectedManaged) -or
    $afterManifest.unrelatedProjectionSha256 -ne $BaselineUnrelatedProjectionSha256.ToLowerInvariant() -or
    $afterManifest.hwpProcessCount -ne 0
  ) {
    throw 'Post-run font state drifted.'
  }

  Copy-Item -FromSession $session -LiteralPath $resultPath -Destination (
    Join-Path $stateOutput "$PhysicalState.interactive.json"
  )
  Copy-Item -FromSession $session -LiteralPath $pdfPath -Destination (
    Join-Path $stateOutput "$PhysicalState.pdf"
  )
  Save-Json (Join-Path $stateOutput "$PhysicalState.ambient-manifest.json") $afterManifest
  Save-Json (Join-Path $stateOutput "$PhysicalState.font-state.json") $fontState
} catch {
  $runError = $_
} finally {
  if ($null -ne $session) {
    try {
      Invoke-Command -Session $session -ScriptBlock {
        param($TaskName)
        Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
        Get-Process Hwp -ErrorAction SilentlyContinue | Stop-Process -Force
      } -ArgumentList $taskName
    } catch { }
    Remove-PSSession $session -ErrorAction SilentlyContinue
    $session = $null
  }

  Restore-Baseline
  $recoverySession = New-DirectSession
  try {
    $recovered = Invoke-GuestManifest $recoverySession
    Save-Json (Join-Path $stateOutput 'recovered.ambient-manifest.json') $recovered
    if (
      $recovered.manifestSha256 -ne $BaselineManifestSha256.ToLowerInvariant() -or
      $recovered.unrelatedProjectionSha256 -ne $BaselineUnrelatedProjectionSha256.ToLowerInvariant() -or
      @($recovered.managedInstalledByExactBytes).Count -ne 0 -or
      $recovered.hwpProcessCount -ne 0
    ) {
      throw 'Final restore did not recover the baseline.'
    }
  } finally {
    Remove-PSSession $recoverySession -ErrorAction SilentlyContinue
  }
}

if ($null -ne $runError) { throw $runError }
[ordered]@{
  ok = $true
  issue = 4963
  queueRank = $QueueRank
  physicalState = $PhysicalState
  baselineRecovered = $true
  privacy = [ordered]@{
    rawVmIdentityIncluded = $false
    absolutePathIncluded = $false
    fontBytesIncluded = $false
    privateCorpusAccessed = $false
  }
} | ConvertTo-Json -Depth 5 -Compress
