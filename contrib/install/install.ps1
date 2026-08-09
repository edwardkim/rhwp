# [#4375] rhwp Windows 설치 스크립트 — 릴리스 zip 을 받아 검증·배치한다.
#
#   irm https://raw.githubusercontent.com/edwardkim/rhwp/devel/contrib/install/install.ps1 | iex
#   # 또는: .\install.ps1 -Version v0.8.2 -InstallDir C:\tools\rhwp -NoPath
#
# 하는 일: ① 버전 해석(latest→GitHub API) ② zip+SHA256SUMS 다운로드
# ③ 해시 검증 ④ 해체(rhwp\rhwp.exe) ⑤ 사용자 PATH 등록(옵트아웃 -NoPath).
param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\rhwp",
    [switch]$NoPath
)

$ErrorActionPreference = "Stop"
$repo = "edwardkim/rhwp"

if ($Version -eq "latest") {
    $rel = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest"
    $Version = $rel.tag_name
}
$asset = "rhwp-$Version-windows-x86_64.zip"
$base = "https://github.com/$repo/releases/download/$Version"

$tmp = Join-Path ([System.IO.Path]::GetTempPath()) "rhwp-install-$([guid]::NewGuid().ToString('N').Substring(0,8))"
New-Item -ItemType Directory -Path $tmp -Force | Out-Null
try {
    Write-Host "다운로드: $base/$asset"
    Invoke-WebRequest "$base/$asset" -OutFile (Join-Path $tmp $asset)
    Invoke-WebRequest "$base/SHA256SUMS.txt" -OutFile (Join-Path $tmp "SHA256SUMS.txt")

    $expected = (Get-Content (Join-Path $tmp "SHA256SUMS.txt") | Where-Object { $_ -match [regex]::Escape($asset) }) -split '\s+' | Select-Object -First 1
    if (-not $expected) { throw "SHA256SUMS.txt 에 $asset 항목이 없습니다" }
    $actual = (Get-FileHash (Join-Path $tmp $asset) -Algorithm SHA256).Hash.ToLower()
    if ($actual -ne $expected.ToLower()) { throw "해시 불일치: 기대 $expected / 실제 $actual" }
    Write-Host "무결성 확인: SHA-256 일치"

    Expand-Archive (Join-Path $tmp $asset) -DestinationPath $tmp -Force
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item (Join-Path $tmp "rhwp\rhwp.exe") (Join-Path $InstallDir "rhwp.exe") -Force

    if (-not $NoPath) {
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($userPath -notlike "*$InstallDir*") {
            [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
            Write-Host "사용자 PATH 에 추가됨(새 터미널부터 적용): $InstallDir"
        }
    }
    & (Join-Path $InstallDir "rhwp.exe") --version
    Write-Host "설치 완료: $InstallDir\rhwp.exe — 첫 확인은 'rhwp capabilities'"
}
finally {
    Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
