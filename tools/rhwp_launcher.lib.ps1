Set-StrictMode -Version Latest

function Write-Utf8NoBomFile {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Path,
    [Parameter(Mandatory = $true)]
    [string]$Content
  )

  $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
  [System.IO.File]::WriteAllText($Path, $Content, $utf8NoBom)
}

function Get-RhwpLauncherConfig {
  $repoRoot = Split-Path -Parent $PSScriptRoot
  $npm = (Get-Command npm.cmd -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source -First 1)

  [pscustomobject]@{
    RepoRoot      = $repoRoot
    StudioDir     = Join-Path $repoRoot 'rhwp-studio'
    LauncherSourceDir = Join-Path $repoRoot 'tools\rhwp-launcher'
    LauncherExePath = Join-Path $repoRoot 'tools\RHWPLauncher.exe'
    DistIndexPath = Join-Path $repoRoot 'rhwp-studio\dist\index.html'
    PreviewUrl    = 'http://127.0.0.1:7702'
    PreviewHost   = '127.0.0.1'
    PreviewPort   = 7702
    ChromeProfile = 'Default'
    ChromePath    = 'C:\Program Files\Google\Chrome\Application\chrome.exe'
    NpmCommand    = $npm
    ShortcutPath  = Join-Path $repoRoot 'RHWP_바로가기.lnk'
    IconPath      = Join-Path $repoRoot 'assets\logo\favicon.ico'
    ApplicationKeyName = 'RHWPLauncher.exe'
    FriendlyAppName = 'RHWP'
  }
}

function Get-RhwpLauncherCommand {
  param(
    [Parameter(Mandatory = $true)]
    $Config
  )

  return "`"$($Config.LauncherExePath)`" `"%1`""
}

function Get-RhwpOpenWithRegistration {
  param(
    [Parameter(Mandatory = $true)]
    $Config
  )

  [pscustomobject]@{
    ApplicationKeyName = $Config.ApplicationKeyName
    ApplicationProgId = "Applications\$($Config.ApplicationKeyName)"
    FriendlyAppName = $Config.FriendlyAppName
    DefaultIconPath = $Config.IconPath
    SupportedExtensions = @('.hwp', '.hwpx')
    Command = Get-RhwpLauncherCommand -Config $Config
  }
}

function Get-RhwpChromeArguments {
  param(
    [Parameter(Mandatory = $true)]
    $Config,
    [string]$AppUrl
  )

  if (-not $AppUrl) {
    $AppUrl = $Config.PreviewUrl
  }

  @(
    "--profile-directory=$($Config.ChromeProfile)"
    "--app=$AppUrl"
  )
}

function Get-RhwpLaunchStage {
  param(
    [Parameter(Mandatory = $true)]
    $Config,
    [Parameter(Mandatory = $true)]
    [string]$FilePath,
    [string]$Token
  )

  $extension = [System.IO.Path]::GetExtension($FilePath).ToLowerInvariant()
  if ($extension -notin @('.hwp', '.hwpx')) {
    throw "지원하지 않는 파일 형식: $FilePath"
  }

  if (-not $Token) {
    $Token = [System.Guid]::NewGuid().ToString('N')
  }

  $stageDir = Join-Path $Config.StudioDir 'dist\__opened'
  $stageFileName = "$Token$extension"
  $manifestFileName = "$Token.json"
  $relativeUrlPath = "__opened/$stageFileName"
  $saveRelativePath = "__rhwp_save/$Token"

  [pscustomobject]@{
    Token            = $Token
    OriginalFilePath = $FilePath
    OriginalFileName = [System.IO.Path]::GetFileName($FilePath)
    StageDir         = $stageDir
    StageFileName    = $stageFileName
    StagePath        = Join-Path $stageDir $stageFileName
    ManifestFileName = $manifestFileName
    ManifestPath     = Join-Path $stageDir $manifestFileName
    RelativeUrlPath  = $relativeUrlPath
    SaveRelativePath = $saveRelativePath
  }
}

function Publish-RhwpLaunchFile {
  param(
    [Parameter(Mandatory = $true)]
    $StageInfo
  )

  if (-not (Test-Path $StageInfo.OriginalFilePath)) {
    throw "열 파일을 찾을 수 없습니다: $($StageInfo.OriginalFilePath)"
  }

  New-Item -ItemType Directory -Force -Path $StageInfo.StageDir | Out-Null
  Copy-Item -LiteralPath $StageInfo.OriginalFilePath -Destination $StageInfo.StagePath -Force
  $manifest = [ordered]@{
    token = $StageInfo.Token
    originalFilePath = $StageInfo.OriginalFilePath
    originalFileName = $StageInfo.OriginalFileName
    stageFileName = $StageInfo.StageFileName
  }
  Write-Utf8NoBomFile -Path $StageInfo.ManifestPath -Content ($manifest | ConvertTo-Json -Compress)
  return $StageInfo.StagePath
}

function Get-RhwpAppUrl {
  param(
    [Parameter(Mandatory = $true)]
    $Config,
    $StageInfo
  )

  if (-not $StageInfo) {
    return $Config.PreviewUrl
  }

  $fileUrl = "$($Config.PreviewUrl)/$($StageInfo.RelativeUrlPath)"
  $saveUrl = "$($Config.PreviewUrl)/$($StageInfo.SaveRelativePath)"
  $encodedFileUrl = [System.Uri]::EscapeDataString($fileUrl)
  $encodedFileName = [System.Uri]::EscapeDataString($StageInfo.OriginalFileName)
  $encodedSaveUrl = [System.Uri]::EscapeDataString($saveUrl)

  return "$($Config.PreviewUrl)/?url=$encodedFileUrl&filename=$encodedFileName&save=$encodedSaveUrl"
}

function Get-RhwpPreviewArguments {
  param(
    [Parameter(Mandatory = $true)]
    $Config
  )

  @(
    'run'
    'preview'
    '--'
    '--host'
    $Config.PreviewHost
    '--port'
    [string]$Config.PreviewPort
  )
}

function Test-RhwpPreviewHtml {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Html
  )

  return $Html -match '<title>\s*rhwp-studio\s*</title>'
}

function Test-RhwpServerReady {
  param(
    [Parameter(Mandatory = $true)]
    $Config
  )

  try {
    $healthUrl = "$($Config.PreviewUrl)/__rhwp_health"
    $response = Invoke-WebRequest -UseBasicParsing -Uri $healthUrl -TimeoutSec 2
    return $response.StatusCode -eq 200 -and $response.Content -match '"ok":true' -and $response.Content -match '"saveBridge":true'
  } catch {
    return $false
  }
}

function Show-RhwpPopup {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Message,
    [string]$Title = 'RHWP Launcher'
  )

  $shell = New-Object -ComObject WScript.Shell
  [void]$shell.Popup($Message, 0, $Title, 0)
}
