---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3681.md
last_verified: 2026-08-04
---

# Task #3681 Stage 1 보고 — 하니스 차트 실행 경로 확립

## 결과

`scripts/visual_sweep.py` 의 **`--file-target KEY DOC PDF` 반복 지정**으로 차트
코퍼스를 preset 등록 없이 측정할 수 있음을 확립했다. 이슈 1단계의 "28종 프리셋"은
불필요 — 쌍 규칙은 `pdf/chart/<분류>/<이름>-2022.pdf` ↔ `samples/chart/<분류>/<이름>.{hwp,hwpx}`
(28↔28 이름 완전 대응 검증).

## 스모크 (묶은세로막대형, devel `44085cb36` + native-skia release)

| 지표 | 값 |
|---|---|
| ink_match | **32.62%** |
| pixel_match | **97.41%** |

이슈 인용(32.6/97.4)과 정확히 일치 — **#3546(차트 원형 보존) merge 는 이 지표를
움직이지 않았다**(저장 축 수정이므로 예상과 정합). 과거 수치가 새 기준선으로 유효.

## 산출물 구조 (문서당)

`pages/page-001.json`(ink/pixel_match) · `overlay/overlay_metrics.json` ·
`analysis/page_001.json`(성분 픽셀) · contact sheet 3종(compare/overlay/review) —
Stage 3 분해에 overlay/analysis 를 그대로 쓴다.

## 정정 기록

이슈의 측정기 지목(`tools/pdf_normalize_compare.py`)은 PDF 바이트 결정성 도구로
오기 — 실측정기는 visual_sweep. 최종 보고에 반영.

## Stage 2 진행

56 타깃(hwp 28 + hwpx 28, hwpx 키는 `_hwpx` 접미) 단일 프로세스 sweep 를 백그라운드
실행(`chart_base`). 완료 후 베이스라인 표 작성.
