# Task 1293 Stage 67: no-separator와 between-notes sweep metric 보강

## 목적

Stage66 baseline에서 separator가 있는 샘플은 대표 시작 page의 `구분선 위/아래` gap을 비교할 수
있었다. 그러나 구분선 없음 샘플은 sweep이 본문/그림 가로선을 separator 후보로 오인했고,
`미주 사이`는 직접 측정하지 못했다.

이번 단계는 source layout을 수정하기 전에 visual sweep이 공식 미주 모양 값을 검증할 수 있도록
metric을 보강한다.

## 수정 대상

- `scripts/task1274_visual_sweep.py`

## 구현 방향

1. note-shape 공식 값 기록
   - page metric에 `endnote_shape_ui`를 넣는다.
   - `separator_visible`, `separator_above_mm`, `between_notes_mm`, `separator_below_mm`를 page별
     분석에서 바로 볼 수 있게 한다.

2. 구분선 없음 처리
   - `separator_visible=false`이면 separator candidate를 찾지 않는다.
   - `endnote_separator_gap` 대신 `endnote_no_separator_content_start` 후보를 기록한다.
   - no-separator에서 발견된 가로선 후보는 공식 separator로 취급하지 않는다.

3. between-notes 후보
   - PDF/RHWP 이미지의 red question marker y list를 사용해 인접 marker gap을 비교한다.
   - 정확한 note visible bottom은 아직 render tree/source 매칭이 필요하므로, Stage67에서는
     marker-to-marker gap 후보를 “betweenNotes 간접 지표”로 기록한다.

## 검증 명령

```sh
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --target 2024-11-practice-above0-between0-below0 \
  --out output/task1293_stage67_gap_metric \
  --rhwp-bin target/debug/rhwp
```

## 기대 결과

- no-separator target에서 `endnote_separator_observed_pages`가 본문 가로선 때문에 증가하지 않는다.
- page metric에 공식 미주 UI 값이 포함된다.
- `between_notes_marker_gap` 후보가 rhwp/pdf별로 기록된다.

## 구현 내용

- `endnote_shape_ui`를 page metric에 추가했다.
  - `separator_visible`
  - `separator_above_mm`
  - `between_notes_mm`
  - `separator_below_mm`
  - `separator_length_mm`
- `separator_visible=false`이면 `horizontal_rule_candidates`와 render tree separator 후보를 수집하지 않는다.
  - 이 경우 `endnote_separator_gap.rhwp/pdf.candidate_count=0`, `selected=null`로 기록한다.
  - 대신 `endnote_no_separator_content_start`에 하단 content/marker 시작 후보를 기록한다.
- red question marker 인접 y 간격을 `between_notes_marker_gap`으로 기록한다.
  - 아직 미주 본문 paragraph 경계 자체가 아니라 marker 기반 간접 지표다.
  - `expected_between_notes_mm/px`, rhwp/pdf marker y 목록, 인접 gap, gap delta를 함께 남긴다.

## 검증 결과

실행 완료:

```sh
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --target 2024-11-practice-above0-between0-below0 \
  --out output/task1293_stage67_gap_metric \
  --rhwp-bin target/debug/rhwp
```

산출물:

- `output/task1293_stage67_gap_metric/summary.json`
- `output/task1293_stage67_gap_metric/2024-11-practice-no-separator-above20-between20-below20/analysis/metrics.json`
- `output/task1293_stage67_gap_metric/2024-11-practice-above0-between0-below0/analysis/metrics.json`

summary 핵심값:

| target | page count | separator observed | separator gap pages | no-separator content pages | marker gap pages |
|---|---:|---:|---:|---:|---:|
| `2024-11-practice-no-separator-above20-between20-below20` | 23/23/23 | `[]` | `[]` | `1..22` | `10,11,12,13,15,16,17,18,20,21,22` |
| `2024-11-practice-above0-between0-below0` | 21/21/21 | `10,11,12,16,17,19,20` | `10` | `[]` | `10,11,12,14,16,17,18,19,20,21` |

검증 판단:

- no-separator target에서 공식 구분선 없음 상태가 metrics에 반영되었다.
  - page 1 기준 `endnote_shape_ui.separator_visible=false`
  - `endnote_separator_gap.rhwp/pdf.candidate_count=0`
  - `endnote_separator_observed_pages=[]`
- separator가 있는 0/0/0 target은 구분선 후보와 gap 비교가 계속 유지된다.
- 이 단계는 검증 metric 보강이며, 실제 남은 `question_marker_flow_drift`/`large_ink_region_drift`
  원인 수정은 다음 stage에서 진행한다.
