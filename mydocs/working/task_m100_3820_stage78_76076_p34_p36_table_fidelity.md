---
kind: implementation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 78 — 76076 p34--p36 표 fragment 직접 PDF 대조

## 입력 계약

대상은 `samples/76076_regulatory_analysis.hwp`와 한컴 기준 PDF
`samples/issue1891/76076_regulatory_analysis-2024.pdf`의 p34--p36이다.

Stage 77 (`4c2a80ea5`)은 p35의 line-seg 없는 control-only 내부 표를 render tree에
복원했다. 그러나 다음 직접 시각 대조 항목은 여전히 독립적으로 판정해야 한다.

- p34 첫 표의 긴 문단이 우측 외곽선을 침범하거나 clip되는지
- p35 내부 표가 PDF와 같은 셀 폭·row owner·외곽선을 가지는지
- p36 continuation의 빈 left rowspan 영역과 다음 행 시작이 PDF와 같은지

## 분석 질문과 완료 조건

1. PDF와 rhwp SVG의 table/cell bbox 및 실제 SVG line을 대조하여, 침범이 source 폭·폰트
   대체인지 renderer의 cell clip/width 오류인지 구분한다.
2. p34--p36에서 outer table과 nested table의 source `(pi, ci)` 소유, cell margin,
   fragment clip, right-edge stroke 가시 폭을 기록한다.
3. 결함이면 소실 지점을 layout → render tree → SVG paint 순서로 확정하고, 한 가지
   분할/clip 계약만 바꾸는 focused regression과 PDF 직접 비교 증적을 남긴다.
4. PDF와 구조가 같고 글꼴 paint 차이만 남으면 코드 수정을 만들지 않고 그 한계를 명시한다.

## 방법

- 격리 release-test binary로 `fidelity_compare` direct-pair p34--p36 sweep과
  `--layout-ledger`를 실행한다.
- PDF 180 DPI raster와 SVG paint를 페이지별로 사람 감사한다. 픽셀 지표는 후보 정렬에만
  사용하며, 최종 판정은 외곽선·텍스트·row owner의 직접 대조로 내린다.
- p34의 의심 셀에는 render-tree JSON과 SVG clip/line을 함께 보관한다.
- 이 단계에서 수정이 생기면 focused regression, 전체 release-test integration, clippy,
  fmt/diff 검사를 다시 실행한다.

## Stage 재지정 — p35 전체 직접 대조

작업지시자의 Stage 77 p35 review 재감사 결과, 중첩 표의 존재만 회복됐을 뿐 p35 전체의
row height·내부 표 vertical placement·glyph paint가 기준 PDF와 크게 다르다. 따라서 이
Stage는 p34/p36의 후보를 먼저 고치지 않고 **p35 outer table(para 347) 전체**의 row geometry와
control paint를 기준 PDF와 다시 대조한다. Stage 77의 존재 회귀는 유지하되, 그것을 PDF fidelity
완료 근거로 사용하지 않는다.

## 확정 분석 — p35 `주요내용` tail paint 누락

기준 PDF의 outer table vector border는 CSS px 환산으로 p35 `y=109.0`에서
`y=1043.2`까지 이어진다. 현재 RHWP render tree는 같은 table의 x/column/horizontal
row boundary를 1--2px 이내로 맞추지만 bottom은 `y=994.1`에서 끝난다. 즉 p35의
`주요내용` text는 남아 있어 기존 Stage 76 회귀를 통과해도, PDF가 보이는 약 50px의
빈 rowspan band가 paint되지 않는다.

`layout_partial_table_resolved`는 Stage 76가 전달한 `end_row_height_override`를
`row_heights[last].min(limit)`으로 적용했다. 이 fixture에서는 auto layout의 마지막 행
height가 내용 한 줄의 23.3px로 이미 줄어 있고, pagination이 계산한 p35 physical tail은
약 74.7px다. 따라서 `min(23.3, 74.7)`가 tail을 23.3px로 다시 축소했다. 이는 font 또는
cell width가 아니라 **fragment-local row height를 cap으로 오해한 paint 경로**다.

`start_row_height_override`와 `end_row_height_override`는 모두 pagination이 소유한
정확한 physical fragment height이므로 해당 행을 `limit`으로 대입한다. 새 regression은
text owner뿐 아니라 outer table bottom이 PDF의 `≈1043px` band까지 유지되는지 고정한다.

## 구현 및 직접 검증

`layout_partial_table_resolved()`는 마지막 행을 `74.7px`로 보존하도록 수정했다.
이때 저장된 마지막 `LINE_SEG`의 높이가 `5.3px`뿐이라 Center cell의 콘텐츠 높이를 그대로
사용하면 `주요내용`이 PDF보다 아래(`1005.0px`)로 밀렸다. 해당 **end-tail을 실제로 소유한
Center cell**에서만 visible glyph em을 최소 콘텐츠 높이로 사용해, text top을
`999.0px`으로 되돌렸다. 한컴 PDF의 같은 glyph top은 `996.8px`이다. 일반 cell의 저장
line-height/vertical-align 경로는 바꾸지 않는다.

focused regression은 p35의 `주요내용` top `995--1001px`, outer table bottom
`1040--1046px`, p36의 다음 `영향평가` 시작점을 함께 고정한다. 따라서 마지막 row의
내용 owner, blank tail, 다음 page 재개를 하나의 계약으로 확인한다.

현재 release-test binary로 PDF p35--p36을 180 DPI에서 다시 대조했다. p35 outer border와
row boundary는 PDF의 1--2px 범위에 있고 p35 tail은 더 이상 조기 종료되지 않는다. 다만
일부 한양계 table body의 글자 폭·줄바꿈 paint 차이는 남아 있으며, 이 수정만으로
pixel-identical을 주장하지 않는다. 그 잔여는 다음 Stage에서 font metric/line-wrap 계약으로
분리한다.

증적:

- [p35 review](../pr/assets/task_m100_3820_stage78_76076_p35_tail_paint/review_035.png)
- [p36 review](../pr/assets/task_m100_3820_stage78_76076_p35_tail_paint/review_036.png)
- [p35 side-by-side](../pr/assets/task_m100_3820_stage78_76076_p35_tail_paint/compare_035.png)
- [p35 overlay](../pr/assets/task_m100_3820_stage78_76076_p35_tail_paint/overlay_035.png)
- [180 DPI metrics](../pr/assets/task_m100_3820_stage78_76076_p35_tail_paint/overlay_metrics.json)
- [run manifest](../pr/assets/task_m100_3820_stage78_76076_p35_tail_paint/run_manifest.json)

## Stage 79 이관

p35 outer table의 물리 fragment는 회복됐지만 p35 본문 셀에서 PDF가 첫 줄을 `…반죽된`에서
끊는 반면 rhwp는 `…반죽된 용`까지 넣는 line-wrap 차이가 남아 있다. 이는 fragment tail과
독립된 text advance/available-width 문제다. 다음 Stage는 source char style·HWP line segment·PDF
glyph advance를 같은 cell에서 대조해, 전역 폭 보정이 아닌 재현 가능한 layout 계약으로 좁힌다.
