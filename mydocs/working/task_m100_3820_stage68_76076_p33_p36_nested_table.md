---
kind: analysis
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 68 — 76076 nested-table p33--p36 PDF fidelity

## 범위

Issue #3820에 후속 후보로 등록한 `76076_regulatory_analysis.hwp`의 연속·중첩 표를
원본 PDF와 직접 대조한다. 사용자가 확인한 p33→34의 표 내부 문단 절단과 p34 첫 표의
우측선 침범, p36의 비정상 빈 표 페이지를 먼저 하나의 physical-owner 흐름으로 분석한다.

| 항목 | 경로 |
| --- | --- |
| 입력 HWP | `samples/76076_regulatory_analysis.hwp` |
| PDF oracle | `samples/issue1891/76076_regulatory_analysis-2024.pdf` |
| 우선 범위 | PDF p33--p36 |

## 원칙

1. PDF p33--p36의 표 fragment, 선, 실제 표시 글자를 기준으로 owner와 clip을 판정한다.
2. page count 또는 기존 baseline만으로 통과 처리하지 않는다.
3. 결함이 재현되면 HWP 저장 좌표/중첩 표 layout 경로를 source와 render tree로 좁힌 뒤,
   한 원인만 고치는 최소 보정과 focused 회귀를 다음 구현 단계에 남긴다.

## 결과 — p34 source owner 확정

현 head의 release-test binary로 p33--p36을 PDF와 직접 비교했다. 자동 visual
detector는 이 범위를 flag하지 않았지만, page별 패널은 서로 다른 표 fragment와 줄바꿈을
보인다. 기준 패널과 지표는 아래에 보관한다.

- [p33--p36 contact sheet](../pr/assets/task_m100_3820_stage68_76076_p33_p036_nested_table/review_contact_sheet.png)
- [p34 review](../pr/assets/task_m100_3820_stage68_76076_p33_p036_nested_table/review_034.png)
- [metrics](../pr/assets/task_m100_3820_stage68_76076_p33_p036_nested_table/metrics.json)
- [summary](../pr/assets/task_m100_3820_stage68_76076_p33_p036_nested_table/summary.json)

p34의 문제 표는 outer table `pi=325`, 오른쪽 cell의 non-TAC nested table이다. 해당
inner cell의 우측선은 `x=710.6`인데, 그 안의 continuation `TextLine`(`pi=10`)은
가용 폭 `442.3px`로 기록되면서도 `TextRun`의 실제 폭은 `453.0px`까지 나가 우측선을
`10.7px` 침범한다. 다른 줄은 최대 `13.7px`까지 초과한다. 따라서 단순 raster 차이가
아니라 cell reflow의 측정과 paint가 불일치한 결함이다.

HWP를 HWPX로 추출해 source를 대조한 결과, 이 문단은 `paraPrIDRef=107`의 negative
indent와 실제 `charPrIDRef` 9/136--141의 서로 다른 자간을 갖는다. 그러나
`compose_lines`의 line-seg 부재 fallback은 첫 char shape(9) 하나로 합성하고,
셀 경로는 `recompose_for_body_width`와 달리 `restyle_fallback_runs_by_char_shapes`를
호출하지 않는다. 그 결과 분할 시의 style/폭과 최종 `TextRun`의 폭이 분리된다.

다음 구현 Stage에서는 이 HWP nested-cell fallback에 실제 char shape를 적용할 수 있는
범위를 76076 source 특성 및 기존 80168 회귀 gate와 함께 좁힌다. 전 셀에 무차별 적용하지
않고, p33→36 PDF fragment와 overflow baseline을 focused 회귀로 사용한다.
