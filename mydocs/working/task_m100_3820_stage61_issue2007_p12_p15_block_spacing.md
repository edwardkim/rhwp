---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 61 — issue2007 p12·p15 block-table 앞 간격

## 문제

Stage 60의 p14 재귀 clip·문단 간격 보정 뒤 PDF를 다시 직접 대조한 결과,
`samples/basic/issue2007_nested_cell_pagination_42065.hwp`의 다음 경계도 한컴오피스
2020 PDF보다 좁다.

- p12 `3 중앙선거관리위원회` 제목과 `공직선거법` 1×1 표 사이
- p15 `7 조달청` 제목과 `조달사업에 관한 법률` 1×1 표 사이

수정 전 증적:

- [p12 rhwp·PDF·overlay](../pr/assets/task_m100_3820_stage61_issue2007_p12_p15_block_spacing/review_p012_before.png)
- [p15 rhwp·PDF·overlay](../pr/assets/task_m100_3820_stage61_issue2007_p12_p15_block_spacing/review_p015_before.png)
- 기준 PDF: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- 재현: Stage 60 binary, `scripts/visual_sweep.py --pages 12,15 --dpi 180`

## 원본 저장 좌표

두 경계 모두 제목 문단과 다음 block-table host 사이에 동일한 저장 간격을 가진다.

| 페이지 | 제목 문단 | 제목 vpos/lh/ls | 표 host 문단 | host vpos | 저장 차이 |
|---|---:|---:|---:|---:|---:|
| p12 | p89 | `480 / 1300 / 780` | p90 | `2560` | `2080 HU` |
| p15 | p101 | `36564 / 1300 / 780` | p102 | `38644` | `2080 HU` |

따라서 제목 기준선 흐름은 `1300 + 780 = 2080 HWPUNIT`을 전진해야 한다.
현재 render tree는 다음과 같다.

- p12: 제목 top `124.7px`, 다음 표 top `143.9px`, delta 약 `19.2px`
- p15: 제목 top `600.1px`, 다음 표 top `619.4px`, delta 약 `19.3px`

두 페이지 모두 제목의 후행 `line_spacing=780HU`가 반영되지 않은 상태다.

## Stage 60과의 관계

Stage 60은 p14의 selected zero-width table 문단을 `(n,n), n>0`로 판별하면서
non-TAC `Control::Table`만 허용했다. p12·p15도 각각 `(1,1), mixed=true`로 현재
cut에 선택됐지만, HWP 원본의 내부 표가 `treat_as_char=true`라 이 조건에서
제외됐다.

두 표는 빈 host 문단에서 mixed nested fragment로 분해돼 실제 block처럼
배치된다. source cut의 `(n,n), n>0` 선택 표식이 owner의 근거이며 TAC 속성은
제외 근거가 아니다. 따라서 선택된 zero-width `Control::Table` 전체를 마지막 렌더
owner 후보로 인정한다. `(0,0)`인 미선택 뒤 문단은 계속 제외한다.

## 완료 조건

- p12·p15 제목→표 간격 회귀가 수정 전 실패하고 수정 후 저장된 780HU를 보존
- p14 Stage 60 간격·ancestor clip 회귀 유지
- p12 중앙선거관리위원회, p15 조달청의 페이지 소유권과 17쪽 수 유지
- issue2007 focused 전체 회귀 통과
- p12·p14·p15를 PDF와 페이지별로 재대조한 최종 증적 보관
- 완료 커밋 뒤 새 Stage에서 전체 PR 게이트를 처음부터 순차 실행

## 수정 결과

`last_rendered_para_idx`의 selected zero-width table 판정을 TAC/non-TAC 공통으로
일반화했다. pagination cursor, CellUnit 높이, 페이지 소유권은 바꾸지 않고 공통
문단 렌더러가 이미 계산한 trailing `line_spacing`을 버리지 않게 했다.

- 회귀 red p12: heading bottom `142.000px`, table top `143.880px`, gap `1.880px`
- 수정 후 p12: heading bottom `142.0px`, table top `154.3px`, gap `12.3px`
- 수정 후 p15: heading bottom `617.4px`, table top `629.8px`, gap `12.4px`
- 기대: trailing spacing `780HU = 10.4px` + 표 외곽 `141HU = 1.88px`
- issue2007 focused integration: 15/15 통과, 17쪽 유지
- visual sweep: requested/completed/missing `3/3/0`, SVG/render-tree `17/17`
- p14 Stage 60 제목 간격·ancestor clip 및 p15 항목 ⑧ 소유권 유지

수정 후 증적:

- [p12 rhwp·PDF·overlay](../pr/assets/task_m100_3820_stage61_issue2007_p12_p15_block_spacing/review_p012_after.png)
- [p14 rhwp·PDF·overlay](../pr/assets/task_m100_3820_stage61_issue2007_p12_p15_block_spacing/review_p014_after.png)
- [p15 rhwp·PDF·overlay](../pr/assets/task_m100_3820_stage61_issue2007_p12_p15_block_spacing/review_p015_after.png)

Stage 61 focused 보정을 커밋한 뒤 전체 PR 게이트는 새 Stage에서 전용 target으로
처음부터 순차 실행한다.
