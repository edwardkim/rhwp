# Task 1293 Stage 68: 공식 미주 샘플 잔여 drift 우선순위 분석

## 목적

Stage67에서 visual sweep이 공식 미주 설정값, 구분선 없음, marker 간접 gap을 기록하도록
보강되었다. 이번 단계에서는 2024-11 공식 미주 설정 샘플 전체를 새 metric으로 다시 돌려
남은 drift를 원인 수정 우선순위로 정리한다.

## 대상

- `2024-11-practice-shape987`
- `2024-11-practice-above0-between0-below0`
- `2024-11-practice-above0-between7-below2`
- `2024-11-practice-above0-between7-below20`
- `2024-11-practice-above0-between20-below2`
- `2024-11-practice-above20-between0-below20`
- `2024-11-practice-above20-between7-below2`
- `2024-11-practice-no-separator-above20-between20-below20`

## 검증 명령

```sh
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-above0-between7-below2 \
  --target 2024-11-practice-above0-between7-below20 \
  --target 2024-11-practice-above0-between20-below2 \
  --target 2024-11-practice-above20-between0-below20 \
  --target 2024-11-practice-above20-between7-below2 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage68_official_metric \
  --rhwp-bin target/debug/rhwp
```

## 확인 항목

- page count 1:1 유지 여부
- `question_marker_flow_drift_pages`
- `large_ink_region_drift_pages`
- `endnote_separator_gap_drift_pages`
- `between_notes_marker_gap`의 최대 delta 후보
- no-separator target의 `endnote_separator_observed_pages=[]` 유지 여부

## 실행 결과

실행 완료:

```sh
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-above0-between7-below2 \
  --target 2024-11-practice-above0-between7-below20 \
  --target 2024-11-practice-above0-between20-below2 \
  --target 2024-11-practice-above20-between0-below20 \
  --target 2024-11-practice-above20-between7-below2 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage68_official_metric \
  --rhwp-bin target/debug/rhwp
```

산출물:

- `output/task1293_stage68_official_metric/summary.json`

공통 결과:

- 8개 target 모두 SVG/PDF/render tree page count가 1:1로 일치했다.
- 8개 target 모두 renderer `LAYOUT_OVERFLOW` 로그는 없다.
- 8개 target 모두 `endnote_separator_gap_drift_pages=[]`이다.
- `2024-11-practice-no-separator-above20-between20-below20`은
  `endnote_separator_observed_pages=[]`를 유지한다.

잔여 flow 후보:

| target | page count | qflow pages | red pages | line pages | large pages |
|---|---:|---|---|---|---|
| `2024-11-practice-shape987` | 21/21/21 | `16,17,18,20,21` | `10,11,12,14,16,17,18,19,20,21` | `9,10,11,13,14,15,17,19,20,21` | `9..21` |
| `2024-11-practice-above0-between0-below0` | 21/21/21 | `11,12,13,17,19,20` | `10,11,12,13,14,16,17,19,20,21` | `9,11,13,14,15,16,17,20` | `9..17,19,20,21` |
| `2024-11-practice-above0-between7-below2` | 21/21/21 | `17,20` | `10,11,16,17,19,20,21` | `9,10,11,12,13,15,17,18,19,20` | `9,10,11,12,13,15,16,17,18,19,20,21` |
| `2024-11-practice-above0-between7-below20` | 21/21/21 | `14,17,20` | `10,11,14,16,17,19,20,21` | `9,10,11,13,15,17,18,19,20` | `9,10,11,13,14,15,16,17,18,19,20,21` |
| `2024-11-practice-above0-between20-below2` | 22/22/22 | `21,22` | `10,13,15,18,20,21,22` | `9,10,13,14,16,17,18,19,20,22` | `9,10,13,14,15,16,17,18,19,20,21,22` |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | `11,16,17,18,19,20,21` | `10,11,12,14,15,16,17,18,19,20,21` | `9,10,12,14,16,17,18,19,20,21` | `9,10,11,12,14,15,16,17,18,19,20,21` |
| `2024-11-practice-above20-between7-below2` | 21/21/21 | `17,20` | `10,11,16,17,18,19,20,21` | `9,11,13,15,17,18,19,20` | `9,10,11,13,15,16,17,18,19,20,21` |
| `2024-11-practice-no-separator-above20-between20-below20` | 23/23/23 | `18,22` | `11,12,13,14,16,17,18,19,20,21,22` | `9,10,15,18,19,22` | `9..22` |

marker gap 최대 후보:

| max delta | target | page | 판단 |
|---:|---|---:|---|
| `511.5px` | `2024-11-practice-above0-between0-below0` | 20 | 실제 시각 차이가 크다. RHWP와 PDF가 문제 30/도형 풀이 분배를 다르게 잡는다. |
| `447.5px` | `2024-11-practice-above0-between0-below0` | 11 | 실제 flow 후보로 보인다. 다음 stage에서 함께 본다. |
| `437.0px` | `2024-11-practice-above20-between7-below2` | 20 | 7mm 계열 공통 후보지만 red marker fragment 영향도 있다. |
| `419.0px` | `2024-11-practice-above0-between20-below2` | 21 | 이미지상 RHWP/PDF가 거의 유사하고, RHWP red marker가 3~4px 간격으로 쪼개져 gap이 과장된 오탐 후보다. |

시각 확인:

- `output/task1293_stage68_official_metric/2024-11-practice-above0-between0-below0/analysis/annotated_020.png`
  - 실제 flow mismatch가 뚜렷하다.
  - `문30` 주변 도형/풀이 분배가 PDF와 다르다.
- `output/task1293_stage68_official_metric/2024-11-practice-above0-between20-below2/analysis/annotated_021.png`
  - 눈으로는 대체로 맞지만 red marker가 조각나 metric delta가 커졌다.
  - `between_notes_marker_gap`은 red band fragment를 먼저 cluster해야 한다.

## 다음 단계

Stage69에서는 source layout 수정 전에 sweep red marker metric을 보강한다.

- 인접 red band가 8px 이하로 붙어 있으면 하나의 marker cluster로 합친다.
- cluster 후에도 `above0-between0-below0` page 20/11이 큰 drift로 남는지 확인한다.
- 오탐이 줄어든 뒤 실제 source 수정 대상 page를 확정한다.
