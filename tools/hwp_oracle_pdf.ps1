<#
.SYNOPSIS
  한컴오피스(한글) COM 자동화로 HWP/HWPX 를 PDF 로 내보낸다 — 시각 오라클 생성용.

.DESCRIPTION
  Python/pyhwpx 가 없는 환경에서 `tools/task2195/to_pdf.py` 와 같은 일을 하는 PowerShell 판이다.
  대화상자·창 노출을 가능한 범위까지 억제한다.

  ## 대화상자 억제 (이 스크립트가 하는 것)

  1. `SetMessageBoxMode(mode)` — 한글이 띄우는 메시지 박스에 자동 응답한다. 기본값 0 은
     "사람이 답할 때까지 대기"라서 자동화가 그 자리에서 멈춘다.
  2. 창 숨김 — `XHwpWindows.Item(0).Visible = $false`. 새 COM 인스턴스는 보통 이미 숨김이지만,
     기존 Hwp.exe 에 붙는 경우가 있어 명시한다.
  3. 시작 전 잔여 `Hwp.exe` 종료 — 떠 있는 인스턴스에 붙으면 그 창의 상태(수정됨 문서 등)를
     물려받아 저장 확인 대화상자가 뜬다.
  4. `RegisterModule("FilePathCheckDLL", "FilePathCheckerModule")` — 보안 모듈이 등록돼 있으면
     파일 접근 확인 대화상자를 건너뛴다. 없으면 조용히 넘어간다(아래 한계 참고).

  ## 한계 — 이 스크립트로 못 막는 대화상자

  한글 자동화의 **파일 접근 보안 확인** 대화상자는 한컴이 배포하는 `FilePathCheckerModule`
  DLL 이 시스템에 등록돼 있어야만 사라진다. DLL 이 없으면 `RegisterModule` 이 실패하고 매
  `Open` 마다 확인 창이 뜬다. 등록은 DLL 설치 + `regsvr32` + `HKCU\Software\HNC\HwpAutomation\
  Modules\FilePathCheckerModule` 레지스트리 쓰기가 필요하므로 이 스크립트는 하지 않는다.

  현재 장비 상태 확인:
    Get-ChildItem C:\Windows\SysWOW64 -Filter FilePathChecker*
    (Get-Item 'HKCU:\Software\HNC\HwpAutomation\Modules').GetValueNames()

.PARAMETER Source
  입력 HWP/HWPX 경로.

.PARAMETER Output
  출력 PDF 경로.

.PARAMETER MessageBoxMode
  `SetMessageBoxMode` 값. 기본 0x00020000.

.PARAMETER KeepAlive
  끝난 뒤 Hwp.exe 를 강제 종료하지 않는다(디버깅용).

.EXAMPLE
  powershell -File tools/hwp_oracle_pdf.ps1 -Source "samples\a.hwp" -Output "output\a.oracle.pdf"
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)][string]$Source,
  [Parameter(Mandatory = $true)][string]$Output,
  [int]$MessageBoxMode = 0x00020000,
  [switch]$KeepAlive
)

$ErrorActionPreference = 'Stop'

$src = (Resolve-Path -LiteralPath $Source).Path

# 절대 경로를 그대로 `Join-Path (Get-Location)` 에 넣으면 형식 오류가 난다 — 상대 경로일
# 때만 현재 위치를 앞에 붙인다. 한글 COM 은 상대 경로를 자기 작업 폴더 기준으로 해석하므로
# 어느 쪽이든 절대 경로로 만들어 넘겨야 한다.
$out = if ([System.IO.Path]::IsPathRooted($Output)) {
  [System.IO.Path]::GetFullPath($Output)
} else {
  [System.IO.Path]::GetFullPath((Join-Path (Get-Location).Path $Output))
}
$outDir = Split-Path -Parent $out
if ($outDir -and -not (Test-Path -LiteralPath $outDir)) {
  New-Item -ItemType Directory -Force -Path $outDir | Out-Null
}
if (Test-Path -LiteralPath $out) { Remove-Item -LiteralPath $out -Force }

function Stop-HwpProcesses {
  Get-Process Hwp -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}

# (3) 잔여 인스턴스 정리 — 붙어버리면 그 창의 대화상자를 물려받는다.
Stop-HwpProcesses
Start-Sleep -Milliseconds 500

$hwp = $null
try {
  $hwp = New-Object -ComObject HWPFrame.HwpObject

  # (1) 메시지 박스 자동 응답. 0 이면 사람이 답할 때까지 멈춘다.
  $prev = $hwp.SetMessageBoxMode($MessageBoxMode)
  Write-Verbose ("SetMessageBoxMode: 0x{0:X} <- 0x{1:X}" -f $MessageBoxMode, $prev)

  # (4) 보안 모듈 — 등록돼 있을 때만 효과가 있다. 없으면 접근 확인 대화상자가 뜬다.
  try {
    $registered = $hwp.RegisterModule("FilePathCheckDLL", "FilePathCheckerModule")
    Write-Verbose "RegisterModule(FilePathCheckerModule) = $registered"
  } catch {
    Write-Warning "FilePathCheckerModule 미등록 — 파일 접근 확인 대화상자가 뜰 수 있다."
  }

  # (2) 창 숨김.
  try { $hwp.XHwpWindows.Item(0).Visible = $false } catch { }

  $null = $hwp.Open($src, "HWP", "forceopen:true")
  $pageCount = $hwp.PageCount
  Write-Output "opened: $src (PageCount=$pageCount)"

  $act = $hwp.CreateAction("FileSaveAsPdf")
  $set = $act.CreateSet()
  $null = $act.GetDefault($set)
  $set.SetItem("FileName", $out)
  $set.SetItem("Format", "PDF")
  $set.SetItem("Attributes", 0)
  $null = $act.Execute($set)

  # PDF 쓰기는 Execute 반환 뒤에도 이어질 수 있다 — 파일이 안정될 때까지 기다린다.
  $deadline = (Get-Date).AddSeconds(60)
  $lastSize = -1
  while ((Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 500
    if (-not (Test-Path -LiteralPath $out)) { continue }
    $size = (Get-Item -LiteralPath $out).Length
    if ($size -gt 0 -and $size -eq $lastSize) { break }
    $lastSize = $size
  }

  if (-not (Test-Path -LiteralPath $out)) { throw "PDF 생성 실패: $out" }
  Write-Output "saved: $out ($((Get-Item -LiteralPath $out).Length) bytes)"

  try { $hwp.Quit() } catch { }
} finally {
  if ($null -ne $hwp) {
    try { [System.Runtime.InteropServices.Marshal]::ReleaseComObject($hwp) | Out-Null } catch { }
  }
  if (-not $KeepAlive) { Stop-HwpProcesses }
}
