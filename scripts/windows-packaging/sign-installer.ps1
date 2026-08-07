#Requires -Version 5.1
<#
.SYNOPSIS
  RHWP Studio NSIS 설치 파일에 자체 서명 인증서로 서명한다.

.DESCRIPTION
  Windows SDK의 signtool.exe로 tauri build가 생성한 .exe 설치 파일에 Authenticode 서명을 적용하고,
  결과를 검증한다. signtool은 PATH에 없을 수 있어 Windows SDK 설치 경로에서 자동으로 찾는다.

.PARAMETER InstallerPath
  서명할 설치 파일(.exe) 경로.

.PARAMETER PfxPath
  generate-self-signed-cert.ps1로 만든 개인키(.pfx) 경로.

.PARAMETER TimestampUrl
  타임스탬프 서버 URL. 인증서 만료 후에도 서명 시점의 유효성을 증명하기 위해 사용한다.
  사내 배포용이라도 넣어두는 편이 안전하다 — 실패해도 서명 자체는 유지된다.

.EXAMPLE
  ./sign-installer.ps1 -InstallerPath ..\..\rhwp-studio\src-tauri\target\release\bundle\nsis\RHWP Studio_0.8.2_x64-setup.exe -PfxPath C:\secure\rhwp-cert\rhwp-studio-cert.pfx
#>

param(
  [Parameter(Mandatory = $true)]
  [string]$InstallerPath,

  [Parameter(Mandatory = $true)]
  [string]$PfxPath,

  [string]$TimestampUrl = 'http://timestamp.digicert.com'
)

$ErrorActionPreference = 'Stop'

if (-not (Test-Path $InstallerPath)) {
  throw "설치 파일을 찾을 수 없습니다: $InstallerPath"
}
if (-not (Test-Path $PfxPath)) {
  throw "인증서(.pfx)를 찾을 수 없습니다: $PfxPath"
}

function Find-SignTool {
  $signtool = Get-Command signtool.exe -ErrorAction SilentlyContinue
  if ($signtool) { return $signtool.Source }

  $candidates = Get-ChildItem `
    -Path 'C:\Program Files (x86)\Windows Kits\10\bin' `
    -Filter 'signtool.exe' `
    -Recurse `
    -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -like '*\x64\*' } |
    Sort-Object FullName -Descending

  if ($candidates) { return $candidates[0].FullName }

  throw 'signtool.exe를 찾을 수 없습니다. Windows SDK가 설치되어 있는지 확인하세요.'
}

$signtoolPath = Find-SignTool
Write-Host "signtool: $signtoolPath"

Write-Host "인증서(.pfx) 암호를 입력하세요:"
$pfxPassword = Read-Host -AsSecureString
$pfxPasswordPlain = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto(
  [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($pfxPassword)
)

Write-Host "설치 파일 서명 중: $InstallerPath"
& $signtoolPath sign `
  /f $PfxPath `
  /p $pfxPasswordPlain `
  /fd SHA256 `
  /tr $TimestampUrl `
  /td SHA256 `
  $InstallerPath

if ($LASTEXITCODE -ne 0) {
  throw "signtool sign 실패 (exit code $LASTEXITCODE)"
}

Write-Host "서명 검증 중..."
& $signtoolPath verify /pa $InstallerPath

if ($LASTEXITCODE -ne 0) {
  # 여러 줄 문자열은 반드시 + 로 이어붙인다 — 백틱 줄바꿈만 쓰면 두 번째 줄부터
  # Write-Warning의 위치 매개 변수로 넘어가 ParameterBindingException이 난다.
  Write-Warning ("서명 검증에 실패했습니다 (exit code $LASTEXITCODE). 자체 서명 인증서는 이 PC에 " +
    "TrustedPublisher/Root로 등록되어 있지 않으면 verify가 실패할 수 있습니다 — 실제 설치 동작은 " +
    "installer-hooks.nsh가 처리하므로 이 경고만으로 서명 자체가 실패한 것은 아닙니다.")
} else {
  Write-Host "서명 검증 완료."
}

# 체크섬 생성 — RELEASE_RUNBOOK.md의 무결성 확인 절차용
$hash = Get-FileHash -Path $InstallerPath -Algorithm SHA256
$checksumPath = "$InstallerPath.sha256.txt"
"$($hash.Hash.ToLower())  $(Split-Path -Leaf $InstallerPath)" | Out-File -FilePath $checksumPath -Encoding ascii
Write-Host "SHA256 체크섬 저장: $checksumPath"
