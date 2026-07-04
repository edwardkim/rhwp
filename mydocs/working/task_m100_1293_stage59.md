# Task 1293 Stage 59: 전체 미주 sweep 재산출

## 목적

Stage57은 no-separator large block의 stale vpos tail advance를 보정했고, Stage58은 qflow 오탐
조건을 좁혔다. 이번 단계에서는 `--target all`로 전체 미주 샘플을 다시 산출해 다른 target 회귀가
없는지 확인한다.

## 검증 명령

```bash
python3 scripts/task1274_visual_sweep.py \
  --target all \
  --out output/task1293_stage59_full_sweep \
  --rhwp-bin target/debug/rhwp
```

## 확인 항목

- SVG/PDF/render tree page count가 모든 target에서 일치하는지 확인한다.
- renderer `LAYOUT_OVERFLOW`가 0인지 확인한다.
- frame/title/order/equation overlap 후보가 없는지 확인한다.
- no-separator target의 qflow 후보가 실제 구조 차이인지 직접 compare PNG로 확인한다.

## 실행 결과

- 실행 완료: `output/task1293_stage59_full_sweep/summary.json`
- 전체 target 수: 15개
- 모든 target에서 SVG/PDF/render tree page count가 1:1로 일치했다.
- renderer `LAYOUT_OVERFLOW`는 전 target에서 0건이다.
- hard gate 후보는 전 target에서 0건이다.
  - `frame_overflow_pages`: 0
  - `question_title_text_overlap_pages`: 0
  - `line_order_overlap_pages`: 0
  - `equation_text_overlap_pages`: 0
- 다만 `visual_metrics.question_marker_flow_drift_pages`는 여러 target에 남아 있다. 따라서
  미주 기능 완료로 판정하지 않고 다음 단계에서 실제 한컴/PDF 흐름 차이와 sweep 오탐을 분리한다.

| target | page count | overflow | hard gate | qflow 후보 |
|---|---:|---:|---:|---|
| `2022-09` | 23/23/23 | 0 | 0 | 9, 16, 20 |
| `2023-09` | 20/20/20 | 0 | 0 | 11, 15, 20 |
| `2024-09-below20` | 23/23/23 | 0 | 0 | 16, 17, 20 |
| `2024-09-between20` | 24/24/24 | 0 | 0 | 17, 18, 21 |
| `2024-09-below20-above20` | 23/23/23 | 0 | 0 | 9 |
| `2022-10` | 18/18/18 | 0 | 0 | 12, 13, 15 |
| `2022-11-practice` | 21/21/21 | 0 | 0 | 17, 20 |
| `2024-11-practice-shape987` | 21/21/21 | 0 | 0 | 16, 17, 18, 20, 21 |
| `2024-11-practice-above0-between0-below0` | 21/21/21 | 0 | 0 | 10, 11, 12, 13, 17, 18, 19, 20 |
| `2024-11-practice-above0-between7-below2` | 21/21/21 | 0 | 0 | 17, 20 |
| `2024-11-practice-above0-between7-below20` | 21/21/21 | 0 | 0 | 14, 17, 20 |
| `2024-11-practice-above0-between20-below2` | 22/22/22 | 0 | 0 | 21, 22 |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 0 | 0 | 11, 16, 17, 18, 19, 20, 21 |
| `2024-11-practice-above20-between7-below2` | 21/21/21 | 0 | 0 | 17, 20 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23/23/23 | 0 | 0 | 18, 22 |

## 판단

Stage24 대비 가장 큰 진전은 renderer overflow가 모두 사라진 점이다. Stage24에서는 `미주 사이`
또는 `구분선 없음` 조합에서 `LAYOUT_OVERFLOW`가 반복됐지만, Stage59에서는 같은 15개 target의
overflow가 모두 0으로 수렴했다.

그러나 구현 계획서의 목적은 단순 overflow 제거가 아니라 한컴 공식 `미주 모양` 의미를 적용해
문항 흐름까지 맞추는 것이다. 현재 qflow 후보가 남아 있으므로, 다음 단계에서는 자동 후보를 직접
compare PNG/render tree로 분류한다.

우선순위는 다음과 같다.

1. `2024-11-practice-above0-between0-below0`
   - 0/0/0 compact 설정인데 qflow 후보가 8쪽으로 가장 많다.
   - `구분선 위/미주 사이/구분선 아래`가 모두 0일 때에도 문항 흐름이 한컴과 달라지는지 확인한다.
2. `2024-11-practice-above20-between0-below20`
   - 미주 사이 0mm, 구분선 위/아래 20mm 조합이다.
   - 구분선 위/아래 block 소비와 미주 내용 시작 위치가 문항 흐름에 주는 영향을 확인한다.
3. `2024-11-practice-no-separator-above20-between20-below20`
   - no-separator 조합은 Stage57/58에서 개선됐지만 page 18, 22 후보가 남아 있다.
   - 구분선 없음일 때 stale vpos guard와 실제 formatter fit 판단이 아직 충분한지 확인한다.

## 다음 단계

Stage60에서는 위 세 target의 qflow 후보를 직접 비교한다. 실제 page/column 흐름 차이로 확인되는
후보만 코드 수정 대상으로 삼고, marker count/large ink segmentation 오탐은 sweep 조건을 조정한다.
