# loadsave_sweep — 한글 오라클 대조 load/save 전수검사 하네스

rhwp 의 HWP/HWPX **불러오기·저장하기**에 누락·오류가 있는지, 설치된 한글(2018/2022/2024)을
오라클로 삼아 전수검사한다. 판정자는 문서가 아니라 **설치된 한글**이다.

## 원리 — 경로 매트릭스가 곧 진단

문서마다 rhwp 저장 경로 두 갈래를 만들고, 한글이 원본과 산출물에서 각각 무엇을 보는지 대조한다.

| 입력 | route | rhwp 명령 | 검증 대상 |
|---|---|---|---|
| .hwp | h2h | `convert` | hwp 파싱 + hwp 저장 |
| .hwp | h2x | `export-hwpx` | hwp 파싱 + hwpx 저장 |
| .hwpx | x2h | `convert` | hwpx 파싱 + hwp 저장 |
| .hwpx | x2x | `export-hwpx` | hwpx 파싱 + hwpx 저장 |

한 입력의 **두 산출이 모두** 나쁘면 불러오기(파서) 쪽, **한쪽만** 나쁘면 그 저장 축 쪽이다.

측정값(모두 같은 한글 버전이 같은 코드 경로로 추출 — 비교가 공평하다):

- 본문 텍스트 (`GetTextFile("TEXT")`) → 텍스트 누락/변형
- 컨트롤 집계 (HeadCtrl 체인 CtrlID 카운트) → 표·그림 등 개체 누락
- 페이지 수 → 레이아웃 붕괴 신호 (참고)
- Open 성공 여부 → 저장 치명 결함

## 실행 절차

전제: Windows에서 동작하는 한글 COM 도구다. 클린 devel 에서 새로 빌드한 `rhwp.exe`
(기존 `target/release` 재사용 금지 — 출처 불명 exe 가 유령 회귀를 만든 전례가 있다),
Python 3.12, 한글 2018/2022/2024 설치가 필요하다. `FilePathCheckerModule`은 등록돼 있으면
파일 접근 확인 대화상자를 줄일 수 있지만 필수는 아니다. worker가 등록을 시도하며, 등록되지
않은 경우에도 결과 로그로 실제 열기 성공 여부를 판정한다.

아래는 **저장소 루트에서 PowerShell을 열었을 때의 예제**다. Corpus와 SweepRoot만 각 사용자의
실제 코퍼스와 쓰기 가능한 대용량 경로로 바꾼다. Repo가 저장소 루트가 아니라면 실제 rhwp
경로를 직접 넣는다.

```powershell
$Repo = (Get-Location).Path                 # rhwp 저장소 루트에서 실행
$Tools = Join-Path $Repo 'tools\loadsave_sweep'
$Corpus = 'C:\경로\to\HWP-HWPX-코퍼스'  # 입력 HWP/HWPX 코퍼스 루트
$SweepRoot = 'C:\경로\to\rhwp-loadsave-sweep' # 쓰기 가능한 대용량 경로 (산출물은 코퍼스의 약 2배)
$Rhwp = Join-Path $Repo 'target\release\rhwp.exe'
$Python = 'py'                            # Python Launcher. py -3.12 --version으로 확인
$HwpVersion = 2022                        # 아래 버전 확인 결과에 맞춰 2018/2022/2024 중 선택

# 준비 확인: 현재 devel에서 새 release 실행 파일을 만든다.
cargo build --release --bin rhwp
& $Python -3.12 --version
powershell -NoProfile -File "$Repo\tools\hangul_version_oracle\list_hangul_versions.ps1"

# 0. 마스터 목록 (파일럿은 --take-hwp 20 --take-hwpx 20)
& $Python -3.12 "$Tools\make_lists.py" --root $Corpus --out "$SweepRoot\master.tsv"

# 1. Phase A — rhwp 변환 매트릭스 (COM 불필요, 병렬, 재개 가능)
& $Python -3.12 "$Tools\rhwp_phase.py" --master "$SweepRoot\master.tsv" `
  --out "$SweepRoot\s1" --rhwp $Rhwp --jobs 6

# 2. Phase B — 한글 2022 오라클 패스 (단일 워커, 장시간, 재개 가능)
powershell -NoProfile -ExecutionPolicy Bypass -File "$Tools\oracle_run.ps1" `
  -HwpVersion $HwpVersion -TaskPath "$SweepRoot\s1\oracle_tasks.tsv" `
  -OutDir "$SweepRoot\s1\oracle_2022" -HideWindow

# 3. 판정
& $Python -3.12 "$Tools\judge.py" --master "$SweepRoot\master.tsv" `
  --phase-a "$SweepRoot\s1\phase_a.ndjson" --oracle "$SweepRoot\s1\oracle_2022\result.tsv" `
  --texts "$SweepRoot\s1\oracle_2022\texts" --out "$SweepRoot\s1\oracle_2022\verdicts"

# 4. 반드시: COM 기본 버전 복원
powershell -NoProfile -File "$Repo\tools\hangul_version_oracle\restore_com_default.ps1"
```

SweepRoot 아래에는 재개용 저널과 변환 산출물이 누적된다. 검증이 끝날 때까지 이 경로를
이동하거나 삭제하지 않는다.

### 2단계 (2018/2024 확대)

1단계(2022 전수)의 실패군 + 등간격 표본으로 축소 master 를 만들어 같은 절차를 반복한다.
Phase A 산출물은 버전 무관이므로 재사용 — Phase B 만 `-HwpVersion 2018` / `2024` 로
다시 돌린다 (`-OutDir` 을 버전별로 분리). **2018 은 `-HideWindow` 금지** (Open 데드락).

## 규약과 함정 (기존 오라클 인프라에서 상속)

- **동시 실행 금지**: 한글 COM 판정은 머신 전체에서 한 번에 하나. 워커도 하나.
- **버전 선택은 HKCU CLSID 오버라이드** — supervisor 가 걸고 시작 전에 실제 major 를
  검증한다. 키를 **삭제하지 말 것**(로그오프까지 오버라이드 무력화). 끝나면
  `restore_com_default.ps1`.
- 워커는 heartbeat 를 쓰고 supervisor 가 정지(stall)를 감지해 Hwp.exe 를 죽인다.
  강제종료 직후 첫 Open 은 수 분 블록될 수 있어 warmup 으로 흡수한다.
- **유령 성공 주의**: 대화상자 자동거부로 "빈 문서 열기 성공"이 나올 수 있다.
  judge 가 원본 텍스트 0자+1쪽을 `origSuspect` 로 표시한다 — 전수에서 이 비율이 높으면
  보안 모듈 등록 상태부터 의심할 것.
- 모든 산출은 재개 가능: Phase A 는 NDJSON 저널, Phase B 는 result.tsv 의 key 로 이어 쓴다.

## 산출물

```
<S>/master.tsv                 docid \t format \t abspath
<S>/s1/conv/                   rhwp 변환 산출물 (<docid>.<route>.hwp|hwpx)
<S>/s1/phase_a.ndjson          Phase A 저널 (exit 0/3/4/FAIL/TIMEOUT + rhwp 자기검증)
<S>/s1/oracle_tasks.tsv        Phase B 작업 목록 (key \t path)
<S>/s1/oracle_<ver>/result.tsv key, status, pages, textLen, textSha, ctrls, fileBytes, err
<S>/s1/oracle_<ver>/texts/     한글이 추출한 본문 텍스트 (판정·삼각측량용)
<S>/s1/oracle_<ver>/verdicts/  verdicts.tsv + summary.md
```

판정 어휘: `CONVERT_FAIL` > `OPEN_FAIL` > `MEASURE_FAIL` > `TEXT_MISMATCH` > `CTRL_DIFF` >
`PAGE_DIFF` > `OK`. 원본이 안 열리는 문서는 `ORACLE_ORIG_FAIL` 로 모수에서 제외.
