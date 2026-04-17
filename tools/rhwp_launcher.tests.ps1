$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$libPath = Join-Path $here 'rhwp_launcher.lib.ps1'

if (-not (Test-Path $libPath)) {
  throw "launcher library missing: $libPath"
}

. $libPath

Describe 'rhwp launcher config' {
  It 'builds the expected localhost preview config' {
    $config = Get-RhwpLauncherConfig

    $config.RepoRoot | Should Be (Split-Path -Parent $here)
    $config.StudioDir | Should Be (Join-Path $config.RepoRoot 'rhwp-studio')
    $config.PreviewUrl | Should Be 'http://127.0.0.1:7702'
    $config.PreviewHost | Should Be '127.0.0.1'
    $config.PreviewPort | Should Be 7702
    $config.ChromeProfile | Should Be 'Default'
  }

  It 'builds app-mode chrome arguments' {
    $config = Get-RhwpLauncherConfig
    $args = Get-RhwpChromeArguments -Config $config

    $args.Count | Should Be 2
    $args[0] | Should Be '--profile-directory=Default'
    $args[1] | Should Be '--app=http://127.0.0.1:7702'
  }

  It 'builds preview server arguments' {
    $config = Get-RhwpLauncherConfig
    $args = Get-RhwpPreviewArguments -Config $config

    $args -join ' ' | Should Be 'run preview -- --host 127.0.0.1 --port 7702'
  }

  It 'builds a staged launch path for hwp files' {
    $config = Get-RhwpLauncherConfig
    $stage = Get-RhwpLaunchStage -Config $config -FilePath 'C:\Temp\sample.hwp' -Token 'launch-token'

    $stage.Token | Should Be 'launch-token'
    $stage.StageFileName | Should Be 'launch-token.hwp'
    $stage.RelativeUrlPath | Should Be '__opened/launch-token.hwp'
    $stage.StageDir | Should Be (Join-Path $config.StudioDir 'dist\__opened')
    $stage.StagePath | Should Be (Join-Path $stage.StageDir 'launch-token.hwp')
    $stage.ManifestPath | Should Be (Join-Path $stage.StageDir 'launch-token.json')
    $stage.SaveRelativePath | Should Be '__rhwp_save/launch-token'
  }

  It 'builds a launch url that passes staged file and original filename' {
    $config = Get-RhwpLauncherConfig
    $stage = [pscustomobject]@{
      RelativeUrlPath = '__opened/launch-token.hwp'
      OriginalFileName = '보고서.hwp'
      SaveRelativePath = '__rhwp_save/launch-token'
    }

    $url = Get-RhwpAppUrl -Config $config -StageInfo $stage

    $url | Should Match '^http://127.0.0.1:7702/\?url='
    $url | Should Match 'filename=%EB%B3%B4%EA%B3%A0%EC%84%9C.hwp'
    $url | Should Match '__opened%2Flaunch-token.hwp'
    $url | Should Match 'save='
  }

  It 'builds app-mode chrome arguments for a staged file url' {
    $config = Get-RhwpLauncherConfig
    $args = Get-RhwpChromeArguments -Config $config -AppUrl 'http://127.0.0.1:7702/?url=test'

    $args.Count | Should Be 2
    $args[0] | Should Be '--profile-directory=Default'
    $args[1] | Should Be '--app=http://127.0.0.1:7702/?url=test'
  }

  It 'recognizes rhwp-studio html by title' {
    Test-RhwpPreviewHtml -Html '<html><head><title>rhwp-studio</title></head></html>' | Should Be $true
    Test-RhwpPreviewHtml -Html '<html><head><title>other</title></head></html>' | Should Be $false
  }

  It 'builds an RHWP open-with registration with friendly name and icon' {
    $config = Get-RhwpLauncherConfig
    $registration = Get-RhwpOpenWithRegistration -Config $config

    $registration.ApplicationKeyName | Should Be 'RHWPLauncher.exe'
    $registration.ApplicationProgId | Should Be 'Applications\RHWPLauncher.exe'
    $registration.FriendlyAppName | Should Be 'RHWP'
    $registration.DefaultIconPath | Should Be $config.IconPath
    $registration.SupportedExtensions | Should Be @('.hwp', '.hwpx')
    $registration.Command | Should Match 'RHWPLauncher\.exe'
    $registration.Command | Should Match '%1'
  }

  It 'writes launch manifest without UTF-8 BOM' {
    $tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("rhwp-launcher-test-" + [System.Guid]::NewGuid().ToString('N'))
    $stageDir = Join-Path $tempRoot 'dist\__opened'
    $originalFilePath = Join-Path $tempRoot 'sample.hwp'
    $runnerScriptPath = Join-Path $tempRoot 'run-launch-stage.ps1'
    [System.IO.Directory]::CreateDirectory($stageDir) | Out-Null
    [System.IO.File]::WriteAllBytes($originalFilePath, [byte[]](1, 2, 3))

    $stageInfo = [pscustomobject]@{
      Token = 'bomless'
      OriginalFilePath = $originalFilePath
      OriginalFileName = 'sample.hwp'
      StageDir = $stageDir
      StageFileName = 'bomless.hwp'
      StagePath = Join-Path $stageDir 'bomless.hwp'
      ManifestFileName = 'bomless.json'
      ManifestPath = Join-Path $stageDir 'bomless.json'
      RelativeUrlPath = '__opened/bomless.hwp'
      SaveRelativePath = '__rhwp_save/bomless'
    }

    $script = @"
. '$libPath'
`$stageInfo = [pscustomobject]@{
  Token = 'bomless'
  OriginalFilePath = '$($stageInfo.OriginalFilePath -replace '\\', '\\')'
  OriginalFileName = '$($stageInfo.OriginalFileName)'
  StageDir = '$($stageInfo.StageDir -replace '\\', '\\')'
  StageFileName = '$($stageInfo.StageFileName)'
  StagePath = '$($stageInfo.StagePath -replace '\\', '\\')'
  ManifestFileName = '$($stageInfo.ManifestFileName)'
  ManifestPath = '$($stageInfo.ManifestPath -replace '\\', '\\')'
  RelativeUrlPath = '$($stageInfo.RelativeUrlPath)'
  SaveRelativePath = '$($stageInfo.SaveRelativePath)'
}
Publish-RhwpLaunchFile -StageInfo `$stageInfo | Out-Null
"@
    [System.IO.File]::WriteAllText($runnerScriptPath, $script, [System.Text.UTF8Encoding]::new($false))

    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $runnerScriptPath | Out-Null

    $bytes = [System.IO.File]::ReadAllBytes($stageInfo.ManifestPath)
    $bytes[0] | Should Not Be 0xEF
    $bytes[1] | Should Not Be 0xBB
    $bytes[2] | Should Not Be 0xBF
  }
}
