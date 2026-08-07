#Requires -Version 5.1
<#
.SYNOPSIS
  RHWP Studio Windows 설치 파일 서명용 자체 서명 코드사이닝 인증서를 생성한다.

.DESCRIPTION
  사내 배포 전용 자체 서명 인증서를 만들고, 개인키를 포함한 .pfx와 공개 인증서만 담은 .cer을
  각각 내보낸다. .pfx는 signtool 서명에만 쓰고 안전하게 보관한다(저장소에 커밋하지 않음).
  .cer은 rhwp-studio/src-tauri/packaging/cert/rhwp-studio-cert.cer 로 복사해 설치 파일에 포함시킨다
  (installer-hooks.nsh가 설치 시 TrustedPublisher/Root 저장소에 등록한다).

  이미 유효한 인증서가 있다면 매 릴리스마다 새로 만들 필요는 없다 — 만료 전까지 재사용한다.

.PARAMETER OutDir
  .pfx/.cer 출력 폴더. 기본값은 스크립트와 같은 폴더의 out\ 이다.

.PARAMETER ValidityYears
  인증서 유효 기간(년). 기본값 3년.

.EXAMPLE
  ./generate-self-signed-cert.ps1
  ./generate-self-signed-cert.ps1 -OutDir C:\secure\rhwp-cert -ValidityYears 5
#>

param(
  [string]$OutDir = (Join-Path $PSScriptRoot 'out'),
  [int]$ValidityYears = 3
)

$ErrorActionPreference = 'Stop'

if (-not (Test-Path $OutDir)) {
  New-Item -ItemType Directory -Path $OutDir -Force | Out-Null
}

$subject = 'CN=RHWP Studio (Internal Distribution), O=rhwp, C=KR'
$notAfter = (Get-Date).AddYears($ValidityYears)

Write-Host "자체 서명 코드사이닝 인증서 생성 중... (Subject: $subject)"

$cert = New-SelfSignedCertificate `
  -Type CodeSigningCert `
  -Subject $subject `
  -KeyUsage DigitalSignature `
  -FriendlyName 'RHWP Studio Internal Code Signing' `
  -CertStoreLocation 'Cert:\CurrentUser\My' `
  -NotAfter $notAfter `
  -KeyExportPolicy Exportable `
  -KeyAlgorithm RSA `
  -KeyLength 2048

$pfxPath = Join-Path $OutDir 'rhwp-studio-cert.pfx'
$cerPath = Join-Path $OutDir 'rhwp-studio-cert.cer'

Write-Host "개인키 보호용 암호를 입력하세요 (.pfx 서명 시 필요합니다):"
$pfxPassword = Read-Host -AsSecureString

Export-PfxCertificate -Cert $cert -FilePath $pfxPath -Password $pfxPassword | Out-Null
Export-Certificate -Cert $cert -FilePath $cerPath | Out-Null

# CurrentUser\My 저장소에는 인증서를 남기지 않는다 — 개인키는 .pfx 파일로만 보관한다.
Remove-Item -Path "Cert:\CurrentUser\My\$($cert.Thumbprint)" -Force

Write-Host ""
Write-Host "완료:"
Write-Host "  개인키 (서명용, 비공개 보관): $pfxPath"
Write-Host "  공개 인증서 (설치 파일에 포함): $cerPath"
Write-Host ""
Write-Host "다음 단계:"
Write-Host "  1. $cerPath 를 rhwp-studio/src-tauri/packaging/cert/rhwp-studio-cert.cer 로 복사"
Write-Host "  2. $pfxPath 는 릴리스 담당자 로컬 또는 사내 비밀 저장소에만 보관 (커밋 금지)"
Write-Host "  3. sign-installer.ps1 로 빌드된 설치 파일에 서명"
