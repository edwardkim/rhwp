# pdf/ — 한글 편집기 PDF 오라클 단일 경로

이 폴더는 `samples/`의 HWP/HWPX를 한글 편집기(Hancom Office Hangul)로 PDF 변환한 결과를
영구 보존한다. rhwp의 SVG/WASM 출력과 비교하는 **한컴 환경 의존 오라클**이며, 생성한 한컴
버전·OS·폰트·출력 방법을 함께 기록해야 한다.

## 1. 폴더 구조

한컴 PDF 오라클은 버전이나 파일 크기와 무관하게 최상위 `pdf/**` 한 곳에서 관리한다.
`pdf-2010/`, `pdf-2020/`, `pdf-large/` 같은 별도 최상위 폴더를 만들지 않는다.

원본이 `samples/basic/`, `samples/hwpx/`처럼 하위 폴더에 있으면 PDF도 가능한 한 같은 상대 구조를
사용한다. 버전은 디렉터리가 아니라 파일명 suffix로 구분한다.

`samples/**`의 입력 fixture, `mydocs/**/assets`의 PR 판정 증적, 도구의 tiny test fixture처럼
소유 목적이 다른 PDF는 이 정본 오라클 경로 정책의 대상이 아니다. 그런 파일을 한컴 정본
오라클로 승격할 때만 `pdf/**`로 옮긴다.

## 2. 명명 규약

| 패턴 | 예시 | 의미 |
|------|------|------|
| `pdf/{원본 stem}-{한컴버전}.pdf` | `pdf/exam_kor-2022.pdf` | 기본 변환 |
| `pdf/{원본 stem}-{형식}-{한컴버전}.pdf` | `pdf/exam_kor-hwp-2020.pdf` | HWP/HWPX 입력 구분 |
| `pdf/{원본 stem}-{형식}-{폰트조건}-{한컴버전}.pdf` | `pdf/exam_kor-hwpx-kopub-2020.pdf` | 폰트 조건 구분 |

원본 파일명이 변환 시점의 한컴 버전을 이미 명확히 포함하는 기존 오라클은 basename을 보존할 수
있다. 새 파일은 최소한 한컴 버전 suffix를 포함하고 PR 본문에 생성 환경을 기록한다.

## 3. 저장소 정책

- 한컴 PDF 오라클은 일반 Git blob으로 커밋한다. Git LFS pointer를 사용하지 않는다.
- 파일 하나의 크기는 **50 MiB(52,428,800 bytes) 미만**이어야 한다. 정확히 50 MiB도 거부한다.
- 상한을 넘으면 그대로 커밋하지 않는다. 재현 범위를 보존하는 축소 fixture, 페이지 발췌 또는 외부
  증적 중 적절한 방식을 별도 이슈에서 승인받는다.
- 기존 파일을 교체할 때는 변환 환경과 변경 이유를 남기고, 필요한 경우 전후 SHA-256을 기록한다.
- LFS pointer, PDF magic 오류, 폐기된 최상위 경로, 크기 상한은 다음 검사로 확인한다.

```bash
python3 scripts/check_pdf_repository_policy.py
```

## 4. 권위 등급

본 PDF 영역의 권위 등급은 **컨트리뷰터 환경별** 로 다르다 (`reference_authoritative_hancom` 메모리 룰 정합).

### Windows + 한컴 편집기 환경
- **권위 정답지 1차**: 한컴 2010 / 2020 / 2022 **편집기** 직접 출력 (시각 판정)
- **권위 정답지 보조**: 본 폴더의 PDF (편집기 PDF 변환본)

### macOS / Linux 환경 (한컴 편집기 미접근)
- **권위 정답지 1차**: 본 폴더의 PDF (한글 **2020** 또는 **2022** 변환본)
- **권위 등급 미달**: 한글 2010 PDF (보조 자료, 정답지 등급 미달)

> macOS/Linux 컨트리뷰터 영역에서는 한컴 편집기 영역 부재로 본 PDF 가 정답지 역할 — **rhwp 의 외부 컨트리뷰터 다양성 인프라**.

### 권위 등급 미달 영역
- 한컴 뷰어 출력 — 정답지 아님
- macOS 인쇄 / 외부 변환 — 정답지 아님
- HWP5 v2024 변환본 등 한컴 변환 산출물 — 비교 보조 자료 (정답지 아님)

## 5. 변환 자동화 스크립트

| 파일 | 용도 |
|------|------|
| `_convert.ps1` | PowerShell 변환 스크립트 (크래시 자동 복구 포함) |
| `_watchdog.ps1` | Hang 감지 watchdog (10초마다 _convert.log mtime 감시 → 60초 무변동 시 Hwp.exe 강제 종료) |
| `_convert.log` | 본 변환 실행 전체 로그 (재현/디버깅 참고용) |

### 환경 구성

#### 5.1 Python + pyhwpx 설치
```powershell
winget install --id Python.Python.3.12 -e --accept-package-agreements --accept-source-agreements --silent --scope user
& "$env:LOCALAPPDATA\Programs\Python\Python312\python.exe" -m pip install pyhwpx
```

`FilePathCheckerModule.dll` 위치:
```
%LOCALAPPDATA%\Programs\Python\Python312\Lib\site-packages\pyhwpx\FilePathCheckerModule.dll
```

#### 5.2 보안 모듈 레지스트리 등록
```powershell
$dll = 'C:\Users\<USER>\AppData\Local\Programs\Python\Python312\Lib\site-packages\pyhwpx\FilePathCheckerModule.dll'
$key = 'HKCU:\Software\HNC\HwpAutomation\Modules'
if (-not (Test-Path $key)) { New-Item -Path $key -Force | Out-Null }
Set-ItemProperty -Path $key -Name 'FilePathCheckerModule' -Value $dll -Type String
```

이 등록 후 COM 코드의 `hwp.RegisterModule('FilePathCheckDLL', 'FilePathCheckerModule')` 가 True 반환 시 보안 다이얼로그 차단.

### PDF 변환 핵심 패턴
```powershell
$hwp = New-Object -ComObject 'HWPFrame.HwpObject'
$null = $hwp.RegisterModule('FilePathCheckDLL', 'FilePathCheckerModule')

$null = $hwp.Open($srcPath, '', '')

$pset = $hwp.HParameterSet.HFileOpenSave
$null = $hwp.HAction.GetDefault('FileSaveAs_S', $pset.HSet)
$pset.filename = $outPath
$pset.Format = 'PDF'
$pset.Attributes = 0
$ok = $hwp.HAction.Execute('FileSaveAs_S', $pset.HSet)

# fallback (FileSaveAs_S 실패 시)
if (-not $ok) { $ok = $hwp.SaveAs($outPath, 'PDF', '') }

$hwp.XHwpDocuments.Item(0).Close($false)
```

### 한글 2010 / 2020 추가 변환 시 참고
- COM ProgID 동일 (`HWPFrame.HwpObject`) — 시스템 마지막 등록 1개만 활성화 → 한글 2010 / 2020 / 2022 변환은 각 버전 단독 설치 환경 또는 가상 머신 / 별도 사용자 계정 분리 필요.
- `FilePathCheckerModule.dll` 등록은 한글 버전 무관.
- `FileSaveAs_S` 의 `Attributes` 매개변수 호환, 단 일부 옵션 (PDF/A 등) 버전 차이 — 변환 후 첫 1~2개 파일 시각 확인 권장.

## 6. PR #670 변환 결과 (한글 2022)

| 항목 | 값 |
|------|-----|
| 총 입력 (samples 재귀) | 207개 (.hwp + .hwpx) |
| 성공 | **199 (96.1%)** |
| 실패 | 8 (3.9%) |
| 크래시 자동 복구 | 3건 |
| 평균 변환 시간 | ~3.5초/파일 |
| 총 소요 시간 | ~15분 |

### 실패 8개 (한글 2022 자동 변환 영역)
| 파일 | 추정 원인 |
|------|-----------|
| `20250130-hongbo_saved.hwp` | RPC 0x800706BE |
| `20250130-hongbo-no.hwp` | SaveAs False |
| `basic\calendar_monthly.hwp` | 8분 hang |
| `honbo-save.hwp` | 한글 크래시 |
| `hwp_table_test_saved.hwp` | 한글 크래시 |
| `hwp-3.0-HWPML.hwp` | HWPML 형식, SaveAs 실패 |
| `hwpers_test4_complex_table.hwp` | 복잡한 테이블, 크래시 |
| `hwpspec.hwp` | 한컴 스펙 문서, 크래시 |

## 7. 출처

PR #670 — @planet6897 (Jaeuk Ryu), 2026-05-07.
