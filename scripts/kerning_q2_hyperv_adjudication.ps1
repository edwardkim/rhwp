<#
.SYNOPSIS
  Export the public Issue #4968 kerning fixture from an exact Hyper-V checkpoint.

.DESCRIPTION
  This host controller restores one Standard checkpoint, stages only tracked
  helpers plus the public fixture/font through PowerShell Direct, runs HWP in
  the existing interactive token, retrieves PDF evidence, and restores the
  same checkpoint in finally. It never installs or removes a system font.
#>
[CmdletBinding(SupportsShouldProcess = $true, ConfirmImpact = 'High')]
param(
  [Parameter(Mandatory = $true)][guid]$ExpectedVmId,
  [Parameter(Mandatory = $true)][guid]$ExpectedCheckpointId,
  [Parameter(Mandatory = $true)][string]$CredentialFile,
  [Parameter(Mandatory = $true)][string]$HostRepoRoot,
  [Parameter(Mandatory = $true)][string]$HostFixture,
  [Parameter(Mandatory = $true)][string]$HostFont,
  [Parameter(Mandatory = $true)][string]$HostOutputRoot,
  [Parameter(Mandatory = $true)][ValidatePattern('^[0-9a-fA-F]{64}$')]
  [string]$ExpectedFixtureSha256,
  [Parameter(Mandatory = $true)][ValidatePattern('^[0-9a-fA-F]{64}$')]
  [string]$ExpectedFontSha256,
  [string]$DocumentFace = 'Noto Sans KR',
  [string]$SecurityModuleName = 'FilePathCheckerModuleExample',
  [string]$GuestRoot = 'C:\rhwp-kerning-q2',
  [ValidateRange(30, 600)][int]$TimeoutSeconds = 180,
  [Parameter(Mandatory = $true)][switch]$CheckpointRestoreApproved
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)

function Get-Sha256([string]$Path) {
  return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Assert-Identity {
  $vm = Get-VM -Id $ExpectedVmId -ErrorAction Stop
  if ($vm.CheckpointType.ToString() -ne 'Standard') {
    throw 'VM checkpoint type must be Standard.'
  }
  if ($vm.AutomaticCheckpointsEnabled) {
    throw 'Automatic checkpoints must be disabled.'
  }
  $snapshots = @(Get-VMSnapshot -VM $vm | Where-Object Id -eq $ExpectedCheckpointId)
  if ($snapshots.Count -ne 1 -or $snapshots[0].SnapshotType.ToString() -ne 'Standard') {
    throw 'Exact Standard checkpoint identity is required.'
  }
  return [ordered]@{ vm = $vm; checkpoint = $snapshots[0] }
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
  if ($unique.Count -ne 1) { throw 'Exactly one interactive Explorer token is required.' }
  return $unique[0]
}

function Write-GuestJson($Session, [string]$Path, [object]$Value) {
  $json = $Value | ConvertTo-Json -Depth 8 -Compress
  Invoke-Command -Session $Session -ScriptBlock {
    param($Destination, $Json)
    $utf8 = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($Destination, $Json, $utf8)
  } -ArgumentList $Path,$json
}

if (-not $CheckpointRestoreApproved) {
  throw 'Explicit checkpoint restore approval is required.'
}
Import-Module Hyper-V -ErrorAction Stop
$script:Credential = Import-Clixml -LiteralPath $CredentialFile
if ($null -eq $script:Credential -or [string]::IsNullOrWhiteSpace($script:Credential.UserName)) {
  throw 'Credential import failed.'
}

$repo = (Resolve-Path -LiteralPath $HostRepoRoot).Path
$fixture = (Resolve-Path -LiteralPath $HostFixture).Path
$font = (Resolve-Path -LiteralPath $HostFont).Path
if ((Get-Sha256 $fixture) -ne $ExpectedFixtureSha256.ToLowerInvariant()) {
  throw 'Host fixture SHA-256 mismatch.'
}
if ((Get-Sha256 $font) -ne $ExpectedFontSha256.ToLowerInvariant()) {
  throw 'Host font SHA-256 mismatch.'
}
$interactiveScript = Join-Path (Join-Path $repo 'scripts') 'oracle_stage4_windows_interactive.ps1'
$taskScript = Join-Path (Join-Path $repo 'scripts') 'oracle_stage4_windows_task.ps1'
foreach ($path in @($interactiveScript, $taskScript)) {
  if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw 'Tracked guest helper is missing.' }
}

$outputRoot = [System.IO.Path]::GetFullPath($HostOutputRoot)
if (Test-Path -LiteralPath $outputRoot) { throw 'Output root already exists.' }
if (-not $PSCmdlet.ShouldProcess(
    "$ExpectedVmId/$ExpectedCheckpointId",
    'restore checkpoint, run one public kerning oracle, and restore checkpoint'
  )) { return }
New-Item -ItemType Directory -Path $outputRoot | Out-Null
$hostStage = Join-Path $outputRoot 'host-stage'
New-Item -ItemType Directory -Path $hostStage | Out-Null
$hostInteractive = Join-Path $hostStage 'oracle_stage4_windows_interactive.ps1'
$hostTask = Join-Path $hostStage 'oracle_stage4_windows_task.ps1'
$hostFixture = Join-Path $hostStage 'fixture.hwpx'
$hostFont = Join-Path $hostStage 'NotoSansKR-Regular.ttf'

$initialState = (Get-VM -Id $ExpectedVmId).State
$session = $null
$runError = $null
$taskName = 'rhwp-4968-kerning-q2-once'
$run = $null
try {
  # PowerShell Direct cannot pass a WSL UNC source directly to -ToSession on
  # every Windows build. Materialize the already hash-checked public inputs in
  # a host-local directory first, then copy that bounded set to the guest.
  Copy-Item -LiteralPath $interactiveScript -Destination $hostInteractive
  Copy-Item -LiteralPath $taskScript -Destination $hostTask
  Copy-Item -LiteralPath $fixture -Destination $hostFixture
  Copy-Item -LiteralPath $font -Destination $hostFont
  Restore-Baseline
  $session = New-DirectSession
  $interactive = Get-InteractiveIdentity $session
  $baseline = Invoke-Command -Session $session -ScriptBlock {
    [ordered]@{ hwpProcessCount = @(Get-Process Hwp -ErrorAction SilentlyContinue).Count }
  }
  if ($baseline.hwpProcessCount -ne 0) { throw 'Guest baseline contains Hwp.exe.' }

  Invoke-Command -Session $session -ScriptBlock {
    param($Root)
    if (Test-Path -LiteralPath $Root) { Remove-Item -LiteralPath $Root -Recurse -Force }
    New-Item -ItemType Directory -Path $Root | Out-Null
  } -ArgumentList $GuestRoot
  $guestInteractive = Join-Path $GuestRoot 'oracle_stage4_windows_interactive.ps1'
  $guestTask = Join-Path $GuestRoot 'oracle_stage4_windows_task.ps1'
  $guestFixture = Join-Path $GuestRoot 'fixture.hwpx'
  $guestFont = Join-Path $GuestRoot 'NotoSansKR-Regular.ttf'
  Copy-Item -LiteralPath $hostInteractive -Destination $guestInteractive -ToSession $session
  Copy-Item -LiteralPath $hostTask -Destination $guestTask -ToSession $session
  Copy-Item -LiteralPath $hostFixture -Destination $guestFixture -ToSession $session
  Copy-Item -LiteralPath $hostFont -Destination $guestFont -ToSession $session
  $guestHashes = Invoke-Command -Session $session -ScriptBlock {
    param($Fixture, $Font)
    [ordered]@{
      fixture = (Get-FileHash -LiteralPath $Fixture -Algorithm SHA256).Hash.ToLowerInvariant()
      font = (Get-FileHash -LiteralPath $Font -Algorithm SHA256).Hash.ToLowerInvariant()
    }
  } -ArgumentList $guestFixture,$guestFont
  if (
    $guestHashes.fixture -ne $ExpectedFixtureSha256.ToLowerInvariant() -or
    $guestHashes.font -ne $ExpectedFontSha256.ToLowerInvariant()
  ) { throw 'Guest staged input hash mismatch.' }

  $pdfPath = Join-Path $GuestRoot 'kerning-q2.pdf'
  $hwpmlPath = Join-Path $GuestRoot 'kerning-q2.readback.hml'
  $resultPath = Join-Path $GuestRoot 'kerning-q2.interactive.json'
  $specPath = Join-Path $GuestRoot 'task-spec.json'
  Write-GuestJson $session $specPath ([ordered]@{
      schemaVersion = 1
      kind = 'font-oracle-hyperv-task-spec'
      issue = 4968
      runner = $guestInteractive
      source = $guestFixture
      pdfOutput = $pdfPath
      hwpmlOutput = $hwpmlPath
      resultOutput = $resultPath
      documentFace = $DocumentFace
      queueRank = 1
      expectedSourceSha256 = $ExpectedFixtureSha256.ToLowerInvariant()
      probeFaces = @()
      fontResourceFiles = @($guestFont)
      securityModuleName = $SecurityModuleName
    })

  Invoke-Command -Session $session -ScriptBlock {
    param($TaskName, $UserId, $TaskScript, $TaskSpec)
    $arguments = '-NoProfile -ExecutionPolicy Bypass -File "' + $TaskScript +
      '" -StateSpec "' + $TaskSpec + '"'
    $action = New-ScheduledTaskAction -Execute 'powershell.exe' -Argument $arguments
    $principal = New-ScheduledTaskPrincipal -UserId $UserId -LogonType Interactive -RunLevel Highest
    Register-ScheduledTask -TaskName $TaskName -Action $action -Principal $principal -Force | Out-Null
    Start-ScheduledTask -TaskName $TaskName
  } -ArgumentList $taskName,$interactive.userId,$guestTask,$specPath

  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
  while ((Get-Date) -lt $deadline) {
    $ready = Invoke-Command -Session $session -ScriptBlock {
      param($Result, $Pdf, $Hwpml)
      (Test-Path -LiteralPath $Result -PathType Leaf) -and
        (Test-Path -LiteralPath $Pdf -PathType Leaf) -and
        (Test-Path -LiteralPath $Hwpml -PathType Leaf)
    } -ArgumentList $resultPath,$pdfPath,$hwpmlPath
    if ($ready) { break }
    Start-Sleep -Seconds 1
  }
  $run = Invoke-Command -Session $session -ScriptBlock {
    param($Result)
    if (-not (Test-Path -LiteralPath $Result -PathType Leaf)) {
      throw 'Interactive result was not created.'
    }
    Get-Content -LiteralPath $Result -Raw -Encoding UTF8 | ConvertFrom-Json
  } -ArgumentList $resultPath
  if (
    $run.issue -ne 4968 -or $run.status -ne 'observed' -or
    $run.inputSha256 -ne $ExpectedFixtureSha256.ToLowerInvariant() -or
    $run.documentFaceSelectable -ne $true -or
    $run.featureDetection.opened -ne $true -or $run.featureDetection.pageCount -ne 1 -or
    $run.environment.securityModuleRegistered -ne $true
  ) { throw 'Interactive kerning run envelope is invalid.' }
  if (
    $null -eq $run.hwpmlReadback -or
    [string]::IsNullOrWhiteSpace([string]$run.hwpmlReadback.sha256) -or
    [int64]$run.hwpmlReadback.bytes -lt 1
  ) { throw 'Interactive HWPML2X readback envelope is invalid.' }
  Copy-Item -FromSession $session -LiteralPath $resultPath -Destination (
    Join-Path $outputRoot 'kerning-q2.interactive.json'
  )
  Copy-Item -FromSession $session -LiteralPath $pdfPath -Destination (
    Join-Path $outputRoot 'kerning-q2.pdf'
  )
  Copy-Item -FromSession $session -LiteralPath $hwpmlPath -Destination (
    Join-Path $outputRoot 'kerning-q2.readback.hml'
  )
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
    $recovered = Invoke-Command -Session $recoverySession -ScriptBlock {
      param($Root)
      [ordered]@{
        hwpProcessCount = @(Get-Process Hwp -ErrorAction SilentlyContinue).Count
        stagingRootPresent = Test-Path -LiteralPath $Root
      }
    } -ArgumentList $GuestRoot
    if ($recovered.hwpProcessCount -ne 0 -or $recovered.stagingRootPresent) {
      throw 'Final restore did not recover the clean checkpoint.'
    }
  } finally {
    Remove-PSSession $recoverySession -ErrorAction SilentlyContinue
  }
  if ($initialState -eq 'Off') { Stop-VM -Id $ExpectedVmId -TurnOff -Confirm:$false }
  if (Test-Path -LiteralPath $hostStage) {
    Remove-Item -LiteralPath $hostStage -Recurse -Force
  }
}

if ($null -ne $runError) { throw $runError }
[ordered]@{
  ok = $true
  issue = 4968
  status = 'observed'
  baselineRecovered = $true
  inputSha256 = $ExpectedFixtureSha256.ToLowerInvariant()
  pdfSha256 = [string]$run.export.pdfSha256
  hwpmlSha256 = [string]$run.hwpmlReadback.sha256
  hancomVersion = [string]$run.environment.hancomVersion
  privacy = [ordered]@{
    rawVmIdentityIncluded = $false
    absolutePathIncluded = $false
    fontBytesIncluded = $false
    privateCorpusAccessed = $false
  }
} | ConvertTo-Json -Depth 5 -Compress
