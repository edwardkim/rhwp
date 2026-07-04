# Task 1293 Stage 69: red marker fragment clustering

## 목적

Stage68에서 `between_notes_marker_gap`이 일부 page에서 실제 시각 차이보다 크게 튀는 것을 확인했다.
대표적으로 `2024-11-practice-above0-between20-below2` page 21은 RHWP/PDF가 대체로 유사하지만,
RHWP의 같은 빨간 문항 marker가 3~4px 간격의 여러 red band로 쪼개져 419px gap delta가 발생했다.

이번 단계는 source layout 수정 전에 sweep의 red marker metric을 보정해 실제 flow mismatch와
red glyph fragment 오탐을 분리한다.

## 수정 대상

- `scripts/task1274_visual_sweep.py`

## 구현 방향

- red marker gap 비교에는 `row_bands(..., predicate=is_red_marker_pixel)` 결과를 그대로 쓰지 않는다.
- metric 전용 `cluster_marker_bands`를 추가한다.
  - 인접 red band가 8px 이하로 붙어 있으면 같은 marker cluster로 병합한다.
  - 병합 시 `y0/y1/x0/x1/pixels/cy`를 갱신한다.
- cluster 뒤에 text-like marker filter를 적용한다.
  - 높이 6~24px, red pixel 30px 이상만 marker metric에 사용한다.
  - 도형 내부 빨간 보조선/화살표와 1~4px짜리 선 조각은 제외한다.
- `red_marker_drift`와 `between_notes_marker_gap`은 cluster된 marker를 사용한다.
  - 기존 annotation용 red band 검출 자체는 바꾸지 않는다.

## 검증 명령

```sh
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-above0-between20-below2 \
  --out output/task1293_stage69_marker_cluster \
  --rhwp-bin target/debug/rhwp
```

## 기대 결과

- `2024-11-practice-above0-between20-below2` page 21의 marker gap delta가 크게 줄어든다.
- `2024-11-practice-above0-between0-below0` page 20/11의 실제 flow 후보는 남는다.

## 구현 내용

- `RED_MARKER_CLUSTER_GAP_PX=8`을 추가했다.
- `cluster_marker_bands`를 추가해 인접 red band를 marker cluster로 병합했다.
- `marker_text_bands`를 추가해 도형 내부 빨간 보조선/화살표를 marker metric에서 제외했다.
  - 높이 6~24px
  - red pixel 30px 이상
- `red_marker_drift`와 `between_notes_marker_gap`은 filtered marker cluster를 사용한다.

## 검증 결과

실행 완료:

```sh
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-above0-between20-below2 \
  --out output/task1293_stage69_marker_cluster \
  --rhwp-bin target/debug/rhwp
```

summary 변화:

| target | Stage68 qflow | Stage69 qflow | 판단 |
|---|---|---|---|
| `2024-11-practice-above0-between0-below0` | `11,12,13,17,19,20` | `11,12,13,19,20` | page 17 오탐 후보 제거, page 20/11 실제 후보 유지 |
| `2024-11-practice-above0-between20-below2` | `21,22` | `10,20` | page 21/22 도형 red line 오탐 제거, page 10/20은 남은 후보 |

대표 page metric:

- `2024-11-practice-above0-between20-below2` page 21
  - `flags=[]`
  - `red_marker_drift.max_abs_delta_px=0.5`
  - `between_notes_marker_gap.max_abs_delta_px=0.0`
  - RHWP/PDF marker y가 각각 `810.5,960.5` / `811.0,961.0`으로 정렬됐다.
- `2024-11-practice-above0-between0-below0` page 20
  - `flags=content_bottom_drift,red_marker_drift,line_band_drift,large_ink_region_drift,question_marker_flow_drift`
  - RHWP marker y `131.0`, PDF marker y `924.0`
  - 실제 문항 흐름 mismatch 후보로 남는다.

## 다음 단계

Stage70에서는 cluster 보강 후 남은 실제 후보를 source layout 관점에서 분석한다.

- `2024-11-practice-above0-between0-below0` page 20/11
- `2024-11-practice-above0-between20-below2` page 10/20
