# #2148 3-sum 오라클 Windows 실행 러너북

리눅스 세션(2026-08-13)에서 준비. 목적 — 비사선 NO_LS 라벨 셀 클램프 착수 판단에 필요한
**한글 실측 행높이 ↔ rhwp 행높이** 대조를 Windows+한컴에서 수행한다.

> **실행 완료(2026-08-13, 한컴 2022) — 결과와 정정은
> [`task2148_3sum_results_windows.md`](task2148_3sum_results_windows.md).**
> 이 러너북의 좌표 가정 중 B(`--pi 10`)와 C(`--pi 0`)는 틀렸고, 오라클 자체에 결함 3건이
> 있어 수선한 뒤 측정했다. 아래 명령은 **`--pi` 기준으로 갱신됨**.
>
> **읽기 전 주의(결과 문서 9절)**: 이 오라클이 재는 `mt_row_heights` 는 **측정 경로**의 값이고,
> 조판·렌더가 쓰는 값과 다를 수 있다(76076 에서 실제로 갈렸다). 행높이 차를 발견하면
> **렌더에도 나타나는지**를 텍스트 y 누적 또는 PDF 괘선으로 반드시 교차 확인한다 —
> 그러지 않으면 조판에 영향 없는 차이를 결함으로 보고하게 된다.

## 0. 전제

- Windows + 한컴오피스 + `pip install pyhwpx`
- 이 브랜치의 `tools/hangul_row_heights.py` — **구버전 사용 금지**: 옛 버전은
  `cut_rows=[...]` 를 찾는데 현행 바이너리는 `TABLE_DRIFT: pi=N ... mt_row_heights=[...]`
  를 내므로 rhwp 쪽이 전부 n/a 로 나온다. 이 브랜치 버전은 현행 표기를 파싱하고
  `--pi` 로 표를 지목한다.
- rhwp release 빌드: `cargo build --release` → `target/release/rhwp.exe`
- 한글 다이얼로그 차단(FilePathCheckerModule) 등록 상태

## 1. 좌표계 — `--pi` 하나로 통일 (실측으로 확정)

- `--pi P` **만 쓴다.** 도구가 양쪽을 같은 좌표로 고른다:
  rhwp 는 TABLE_DRIFT 의 `pi=P`, 한글은 앵커가 `(List=0, Para=P)` 인 tbl.
- 근거: **한글 tbl `GetAnchorPos(0)` 의 `List==0` 일 때 `Para` == rhwp `pi`** (76076 상위
  60개 표에서 36 OK / 4 MISMATCH / 20 미해당으로 검증).
- `--table-index N`(한글 HeadCtrl 순번)은 레거시다. HeadCtrl 순서는 **문서 순서와 어긋난다**
  (중첩 표가 끼어든다) — 76076 에서 `--table-index 3` 은 표4 가 아니라 엉뚱한 표를 잡았다.
- `rhwp info` 의 `표N [구역S:문단P]` 에서 **문단P 가 곧 `--pi`** 다. 표N 의 N-1 은
  table-index 가 **아니다**.
- MISMATCH 는 **1×1 래퍼 표 + 중첩 표** 형태에서 난다(pi 충돌). 도구는 info 행수와
  drift 행수가 다르면 건너뛴다.

## 2. 실행 대상 (우선순위순)

### A. 36404953 — #2148 원 재현 (FIXED 목록 대표)

```
python tools/hangul_row_heights.py samples/task2279/36404953_gyeoljae.hwpx ^
    --exe target\release\rhwp.exe --pi 0
```

- 리눅스 선측정 rhwp 값: `[32.2, 79.3, 17.1, 31.7, 31.7, 30.0]` (6행, 합 222.0px)
- 현행 devel 쪽수는 이미 1쪽(한컴 정합)이다. 한글 행높이가 위와 일치하면 이 문서
  계열은 클램프 없이 종결 — 어긋나면 그 diff 가 #2148 의 실증거.
- **결과: 총합 −0.02px, 이탈 행 0/6 — 완전 일치.** 클램프 없이 종결 쪽.

### B. 76076 — 반대 진실 게이트 (회귀 피해자 대표)

```
python tools/hangul_row_heights.py samples/issue1891/76076_regulatory_analysis.hwpx ^
    --exe target\release\rhwp.exe --all
```

- **정정**: 원래 지목한 `표4 [구역0:문단10]` 은 **TABLE_DRIFT 를 내지 않아 측정 불가**다
  (최상위 표 19개가 같다). 표를 손으로 고르는 대신 `--all` 로 측정 가능한 표를 전수 대조한다.
- 현행 devel 쪽수 82 = 한컴 2024 PDF 82 (재확인 ✓).
- **결과: 85개 표 / 414행. 58개 표가 0.5px 이내, 중앙 +0.02px. |차|>1px 27개인데 방향이
  쏠린다 — rhwp 큼 23 vs 한글 큼 4, 누적 +163.8px.** 형태별 상수 결함:
  1×1 표 13개가 전부 +3.8px, 5×2 표 4개가 전부 +6.4px (둘이 누적의 46%).

### C. 21761835 — 사선 셀 기준 문서 (#2146 계열, 줄높이 공식 축)

```
python tools/hangul_row_heights.py samples/task2146/21761835_jeonjik_exemption_table.hwp ^
    --exe target\release\rhwp.exe --pi 4
```

- **정정**: `--pi 0` 이 아니라 **`--pi 4`**. 이 문서에는 표가 하나뿐이다
  (`표1 [구역0:문단4]` 78행×5열, 셀 296개).
- 이슈 본문의 "한글 실측 ≈24.3px vs 재합성 37.76px" 를 이 도구로 재확인.
- **결과: 측정 불가.** 78행 전부 값이 나오지만 복원 합이 rhwp 합과 −711.76px 어긋난다
  (A·D 는 각각 −0.02/+8.68px). 병합이 심해 rowspan=1 셀이 없는 행에서 복원 휴리스틱이
  깨진다 — 큰 diff 다수가 병합 잔재다. 이슈 본문 수치는 확인하지 못했다.

### D. sample1-repro — PDF 괘선 측정과 3중 대조 (교차 검증용)

```
python tools/hangul_row_heights.py samples/issue4514/sample1-repro.hwp ^
    --exe target\release\rhwp.exe --pi 763
```

- 표64 [구역0:문단763] 75행×10열. 리눅스 선측정: rhwp 중앙 32.12px,
  한컴 2020 PDF 괘선 중앙 32.0px. COM 값까지 32.0 이면 "PDF 괘선 추출 = COM"이
  성립해, 이후 행높이 전수 대조를 COM 없이 PDF 로 할 수 있다는 방법 검증이 된다.
- **결과: 방법 검증 성립.** COM 32.13 ↔ rhwp 32.12 ↔ PDF 괘선 32.0 (0.13px = 0.4% 안).
  단 2022 COM ↔ 2020 PDF 라 그 0.13px 이 버전차인지 추출 오차인지는 미분리.
- 72/75 행이 ±0.01px 일치. 이탈 3행(7/35/62)은 **전부 rhwp 가 32.12 = 최빈 피치**로
  찍는다(한글 21.20/47.96/18.07). 팽창이 아니라 **균질화**가 오차원이다.

## 3. 결과 보존

`output/` 은 **gitignore 대상**이다(`.gitignore:15 /output/`). 실행 원문은 커밋되지 않으니
판정 요약은 [`task2148_3sum_results_windows.md`](task2148_3sum_results_windows.md) 에 남긴다.

```
mkdir output\task2148 2>nul
python ... > output\task2148\36404953.txt 2>&1
```

각 실행의 표 전문(`row 한글_px rhwp_px diff`)과 표 총합차를 남긴다.
판정 게이트 — **복원 합 == rhwp 합** 인지 먼저 본다. 어긋나면 그 표의 행별 diff 는
믿을 수 없다(병합 잔재). 방문 셀 수가 `rhwp info` 의 셀 개수와 같은지도 함께 본다.

## 4. 이 러너북의 배경

- #2148 코멘트(2026-08-13): 저장소 상주 앵커 2건(36404953·76076)이 현행 devel 에서
  이미 정답 — 클램프 착수 전 코호트 재계측 필요.
- #4568 코멘트(같은 날): sample1-repro 75×10 표의 행 피치는 rhwp↔한컴 PDF 일치(32.1↔32.0)
  — 행높이 팽창 가설의 반례. 쪽수 +2 는 전부 "꼬리 조각 뒤 사다리 되감기" 축.

## 5. 실행 전 확인 (Windows)

- 한컴 COM 버전은 `HWPFrame.HwpObject` 의 CLSID `{2291CF00-…}` 를 **HKCU** 가 이긴다.
  버전별 ProgID 분기가 없으므로 버전을 바꾸려면 HKCU 등록을 갈아끼워야 한다.
  실제 붙은 버전은 `hwp.Version` 으로 확인한다(2022 = `[12,0,0,535]`).
- 도구는 실행 시 `taskkill /F /IM Hwp.exe` 를 먼저 때린다 — 측정 중 한글을 열지 말 것.
- 스윕 도중 `RPC 서버를 사용할 수 없습니다` 가 나면 직전 COM 서버 잔재다.
  `taskkill /F /IM Hwp.exe` 후 재시도하면 풀린다.
- 브랜치는 `origin`(edwardkim)이 아니라 **`myfork`(planet6897)** 에 있다 —
  `git fetch myfork` 가 필요하다.
