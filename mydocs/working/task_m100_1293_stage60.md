# Task 1293 Stage 60: 잔여 qflow 후보 실제성 분류

## 목적

Stage59 전체 sweep에서 renderer overflow와 hard gate 후보는 모두 0이 되었지만,
`visual_metrics.question_marker_flow_drift_pages`가 여러 target에 남았다. 이번 단계에서는
자동 qflow 후보를 직접 compare PNG, annotated PNG, metrics, render tree로 확인해 실제 한컴/PDF
흐름 차이와 sweep 오탐을 분리한다.

## 우선 확인 대상

1. `2024-11-practice-above0-between0-below0`
   - qflow 후보: 10, 11, 12, 13, 17, 18, 19, 20
   - 0/0/0 compact 설정에서 흐름 차이가 누적되는지 확인한다.
2. `2024-11-practice-above20-between0-below20`
   - qflow 후보: 11, 16, 17, 18, 19, 20, 21
   - 구분선 위/아래 20mm와 미주 사이 0mm 조합에서 block 소비가 맞는지 확인한다.
3. `2024-11-practice-no-separator-above20-between20-below20`
   - qflow 후보: 18, 22
   - 구분선 없음 + 20/20/20에서 stale vpos tail guard 보정 후에도 남은 구조 차이를 확인한다.

## 검토 기준

- 문항 marker의 y drift가 커도 marker count/segmentation 차이만이면 sweep 오탐으로 분류한다.
- PDF와 rhwp의 page/column 시작 문항 또는 tail 문단 배치가 다르면 실제 결함으로 분류한다.
- 실제 결함은 먼저 최초 drift page를 찾고, 해당 page의 마지막 fit/advance 판단을 추적한다.
- hard gate가 0이어도 한컴 흐름이 다르면 목표 미완료로 본다.

## 산출물

- 전체 sweep: `output/task1293_stage59_full_sweep`
- 각 target의 비교 이미지:
  - `compare/compare_XXX.png`
  - `annotated/annotated_XXX.png`
  - `analysis/metrics.json`
  - `render_tree/render_tree_XXX.json`

## 확인 결과

### `2024-11-practice-above0-between0-below0`

Stage59 기준 qflow 후보는 10, 11, 12, 13, 17, 18, 19, 20쪽이었다. 먼저 10쪽의
실제성을 확인했다.

- `output/task1293_review_current_zero/.../compare/compare_010.png`
  - rhwp와 PDF 모두 page count는 21/21/21로 일치한다.
  - page 10에서 문7/문11 제목 tail은 0/0/0 compact 설정에서 같은 쪽/단에 남는 것이
    PDF 흐름에 더 가깝다.
- `output/task1293_review_current_zero/.../compare/compare_011.png`
  - 문13 제목을 왼쪽 단 하단에 남기려는 경로에서는 `pi=539`가 renderer VPOS 기준으로
    `LAYOUT_OVERFLOW_DRAW`와 `LAYOUT_OVERFLOW`를 만들었다.
  - `pi=539` 자체는 한 줄 제목이지만, 직전 `pi=538` 수식 line box가 하단부를 차지한 뒤
    제목이 순차 y로 배치되어 frame 밖으로 내려간다.

임시 계측으로 확인한 핵심 수치는 다음과 같다.

| pi | 문항 | col | current_height | available | line_advance | 판단 |
|---:|---:|---:|---:|---:|---:|---|
| 480 | 문7 | 0/2 | 981.9 | 1001.6 | 18.0 | 첫 단이지만 전체 line advance까지 들어가므로 유지 가능 |
| 512 | 문11 | 1/2 | 1007.2 | 1001.6 | 18.0 | 마지막 단 tail이라 한컴식 작은 bleed 허용 대상 |
| 539 | 문13 | 0/2 | 985.7 | 1001.6 | 18.0 | 첫 단이고 line advance까지는 들어가지 않아 다음 단으로 넘겨야 overflow가 사라짐 |

이에 따라 0/0/0 미주 제목 tail guard를 다음처럼 좁혔다.

- 기존: `height_for_fit` 한 줄 기준으로만 tail 허용
- 변경: 첫 단에서는 `line_advance`까지 현재 단에 들어갈 때만 tail 허용
- 마지막 단에서는 기존처럼 작은 bottom bleed를 허용

이렇게 하면 `pi=539`를 frame 밖에 남기지 않고, 문7/문11처럼 PDF 흐름에 도움이 되는 tail은 유지한다.

## Stage60 target sweep

실행:

```sh
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-above0-between0-below0 \
  --out output/task1293_stage60_zero_line_advance_guard \
  --rhwp-bin target/debug/rhwp
```

결과:

- page count: SVG/PDF/render tree `21/21/21`
- `overflow_lines`: 0
- `frame_overflow_pages`: `[]`
- `question_title_text_overlap_pages`: `[]`
- `line_order_overlap_pages`: `[]`
- qflow 후보: `[11, 12, 13, 17, 19, 20]`

Stage59 대비 qflow 후보에서 10쪽과 18쪽은 빠졌지만, 11쪽은 아직 실제 흐름 차이가 남아 있다.
특히 `compare_011.png` 기준 PDF는 문13 제목이 왼쪽 단 하단에 남고 rhwp는 문13이 오른쪽 단
상단으로 이동한다. Stage60은 hard overflow를 제거하고 0/0/0 제목 tail guard를 더 안전하게
좁힌 단계이며, 문12/문13의 실제 한컴 흐름 정합은 다음 stage에서 계속 분석한다.
