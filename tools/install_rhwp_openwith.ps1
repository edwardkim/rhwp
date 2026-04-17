[CmdletBinding()]
param(
  [string]$OpenAsFile
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

. (Join-Path $PSScriptRoot 'rhwp_launcher.lib.ps1')

$config = Get-RhwpLauncherConfig
$registration = Get-RhwpOpenWithRegistration -Config $config

function Set-RegistryDefaultValue {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Path,
    [Parameter(Mandatory = $true)]
    [string]$Value
  )

  New-Item -Path $Path -Force | Out-Null
  Set-Item -Path $Path -Value $Value
}

$applicationsKey = "HKCU:\Software\Classes\Applications\$($registration.ApplicationKeyName)"
New-Item -Path $applicationsKey -Force | Out-Null
New-ItemProperty -Path $applicationsKey -Name 'FriendlyAppName' -Value $registration.FriendlyAppName -PropertyType String -Force | Out-Null

Set-RegistryDefaultValue -Path (Join-Path $applicationsKey 'DefaultIcon') -Value $registration.DefaultIconPath
Set-RegistryDefaultValue -Path (Join-Path $applicationsKey 'shell\open\command') -Value $registration.Command

$supportedTypesKey = Join-Path $applicationsKey 'SupportedTypes'
New-Item -Path $supportedTypesKey -Force | Out-Null

foreach ($extension in $registration.SupportedExtensions) {
  New-ItemProperty -Path $supportedTypesKey -Name $extension -Value '' -PropertyType String -Force | Out-Null
}

# 이전 실험 흔적 정리: 사용자 기본값을 강제로 바꾸지 않도록 HKCU 기본값/커스텀 ProgID 제거
foreach ($cleanupPath in @(
  'HKCU:\Software\Classes\.hwp',
  'HKCU:\Software\Classes\.hwpx',
  'HKCU:\Software\Classes\RHWP.HwpFile',
  'HKCU:\Software\Classes\RHWP.HwpxFile'
)) {
  if (Test-Path $cleanupPath) {
    Remove-Item -LiteralPath $cleanupPath -Recurse -Force
  }
}

if ($OpenAsFile) {
  $resolved = (Resolve-Path -LiteralPath $OpenAsFile).Path
  Start-Process -FilePath 'rundll32.exe' -ArgumentList @('shell32.dll,OpenAs_RunDLL', "`"$resolved`"") | Out-Null
}

Write-Output "RHWP Open with registration installed."
