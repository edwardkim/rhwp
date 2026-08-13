# #2148 3-sum 오라클 Windows 실행 러너북

리눅스 세션(2026-08-13)에서 준비. 목적 — 비사선 NO_LS 라벨 셀 클램프 착수 판단에 필요한
**한글 실측 행높이 ↔ rhwp 행높이** 대조를 Windows+한컴에서 수행한다.

## 0. 전제

- Windows + 한컴오피스 + `pip install pyhwpx`
- 이 브랜치의 `tools/hangul_row_heights.py` — **구버전 사용 금지**: 옛 버전은
  `cut_rows=[...]` 를 찾는데 현행 바이너리는 `TABLE_DRIFT: pi=N ... mt_row_heights=[...]`
  를 내므로 rhwp 쪽이 전부 n/a 로 나온다. 이 브랜치 버전은 현행 표기를 파싱하고
  `--pi` 로 표를 지목한다.
- rhwp release 빌드: `cargo build --release` → `target/release/rhwp.exe`
- 한글 다이얼로그 차단(FilePathCheckerModule) 등록 상태

## 1. 좌표계 주의 — --table-index ↔ --pi

- `--table-index N`: **한글 쪽** HeadCtrl 순회 순번 (0-based)
- `--pi P`: **rhwp 쪽** TABLE_DRIFT 의 문단 인덱스
- 두 좌표계 대응은 `rhwp info <파일>` 의 `표N [구역S:문단P]` 로 읽는다
  (표N 의 N-1 = table-index, 문단P = pi). 중첩 표가 있으면 HeadCtrl 순서가
  info 순서와 어긋날 수 있으므로, 행수 불일치가 나면 인접 인덱스를 ±2 탐색한다.

## 2. 실행 대상 (우선순위순)

### A. 36404953 — #2148 원 재현 (FIXED 목록 대표)

```
python tools/hangul_row_heights.py samples/task2279/36404953_gyeoljae.hwpx ^
    --exe target\release\rhwp.exe --table-index 0 --pi 0
```

- 리눅스 선측정 rhwp 값: `[32.2, 79.3, 17.1, 31.7, 31.7, 30.0]` (6행, 합 222.0px)
- 현행 devel 쪽수는 이미 1쪽(한컴 정합)이다. 한글 행높이가 위와 일치하면 이 문서
  계열은 클램프 없이 종결 — 어긋나면 그 diff 가 #2148 의 실증거.

### B. 76076 — 반대 진실 게이트 (회귀 피해자 대표)

```
python tools/hangul_row_heights.py samples/issue1891/76076_regulatory_analysis.hwpx ^
    --exe target\release\rhwp.exe --table-index 3 --pi 10
```

- 표4 [구역0:문단10] 5행×7열 부터. RHWP_DIAG_LABEL 클램프 대상이던 일반 라벨 셀 표들
  (구분/장점/할인율…)을 추가로 돌리려면 `rhwp info` 로 표 목록(113개)에서 골라
  `--table-index`/`--pi` 를 맞춘다.
- 현행 devel 쪽수 82 = 한컴 2024 PDF 82. 한글이 이 표들을 선언높이 이상으로 키우는지가
  "성장 계열" 판별의 실측.

### C. 21761835 — 사선 셀 기준 문서 (#2146 계열, 줄높이 공식 축)

```
python tools/hangul_row_heights.py samples/task2146/21761835_jeonjik_exemption_table.hwp ^
    --exe target\release\rhwp.exe --table-index 0 --pi 0
```

- 이슈 본문의 "한글 실측 ≈24.3px vs 재합성 37.76px" 를 이 도구로 재확인.

### D. sample1-repro — PDF 괘선 측정과 3중 대조 (교차 검증용)

```
python tools/hangul_row_heights.py samples/issue4514/sample1-repro.hwp ^
    --exe target\release\rhwp.exe --table-index 63 --pi 763
```

- 표64 [구역0:문단763] 75행×10열. 리눅스 선측정: rhwp 중앙 32.12px,
  한컴 2020 PDF 괘선 중앙 32.0px. COM 값까지 32.0 이면 "PDF 괘선 추출 = COM"이
  성립해, 이후 행높이 전수 대조를 COM 없이 PDF 로 할 수 있다는 방법 검증이 된다.

## 3. 결과 보존

```
mkdir output\task2148 2>nul
python ... > output\task2148\36404953.txt 2>&1
```

각 실행의 표 전문(`row 한글_px rhwp_px diff`)과 `누적 diff` 를 남긴다.
판정: |diff| 합이 행수×0.5px 이내면 정합, 특정 행에 몰린 큰 diff 는 그 행 셀의
콘텐츠(그림/여러 줄)와 함께 기록.

## 4. 이 러너북의 배경

- #2148 코멘트(2026-08-13): 저장소 상주 앵커 2건(36404953·76076)이 현행 devel 에서
  이미 정답 — 클램프 착수 전 코호트 재계측 필요.
- #4568 코멘트(같은 날): sample1-repro 75×10 표의 행 피치는 rhwp↔한컴 PDF 일치(32.1↔32.0)
  — 행높이 팽창 가설의 반례. 쪽수 +2 는 전부 "꼬리 조각 뒤 사다리 되감기" 축.
