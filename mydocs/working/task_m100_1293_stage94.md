# task 1293 stage94 - p11 visual order 후보 분리

## 목적

stage93 후 targeted sweep에서 남은 후보는 `2024-09-between20` p11 하나이다. 이 후보는
`pi=573` text `따라서` 뒤의 `pi=574` `[VISUAL]` bbox가 더 위 y로 잡히면서
`line_order_overlap`과 line/column/large drift를 함께 유발한다.

stage94에서는 이 visual/text 순서 후보가 실제 pagination/paint order 문제인지, treat-as-char
visual의 logical bbox가 텍스트보다 위쪽으로 확장되는 검출기 과검출인지 분리한다.

## 시작 기준

- 브랜치: `local/task_m100_1293`
- 시작 커밋: `6ef8838e task 1293: 수식 tail sweep 과검출 보정`
- stage93 targeted sweep:
  - `2024-09-between20`: `flagged=1/24`
  - 남은 page: p11
  - flags: `line_order_overlap`, `line_band_drift`, `column_line_band_drift`, `large_ink_region_drift`
  - candidate: prev `pi=573` text=`따라서`, next `pi=574` text=`[VISUAL]`, `y_delta=-19.4`
  - 나머지 기준 샘플 3개는 `flagged=0`

## 처리 방향

- `dump-pages`, render-tree, annotated/compare 출력으로 p11 `pi=573/574`의 실제 배치와 PDF 흐름을 비교한다.
- `[VISUAL]` logical bbox가 그림/수식 baseline 때문에 위로 확장된 경우에는 sweep의 line order 후보를 좁힌다.
- 실제 paint order나 pagination 문제이면 typeset/render 쪽을 좁게 수정한다.

## 검증 계획

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage94_targeted
git diff --check
```

CI 전체 테스트와 PR은 작업지시자 지시에 따라 수행하지 않는다.

## 작업지시자 승인

2026-06-14 작업지시자가 자동 승인과 연속 커밋 진행을 지시했다.

## 조사 기록

- `dump-pages -p 10` 기준 p11의 후보 지점:
  - `pi=573`: text `따라서`, line-height 900
  - `pi=574`: 4줄짜리 equation/visual 문단, 첫 TextLine bbox가 `y=133.6..165.0`
- render-tree 기준:
  - `pi=573` bbox: `[34.0, 153.0, 357.2, 12.0]`
  - `pi=574` 첫 visual-empty bbox: `[34.0, 133.6, 357.2, 31.4]`
- compare/annotated PNG 확인 결과, PDF와 rhwp의 페이지 흐름은 크게 다르지 않고 후보는 `따라서` 다음 수식 문단의
  logical bbox가 윗줄까지 확장되면서 생긴 text -> `[VISUAL]` 순서 과검출로 판단했다.

## 구현 기록

- `render_tree_line_order_overlap_candidates`에서 `text -> [VISUAL]` 전환 후보 중 아래 조건을 만족하면 제외한다.
  - 이전 line은 question title이 아닌 visible text이다.
  - 다음 line은 visual-empty line이다.
  - 다음 visual bbox의 y가 이전 text보다 위에 있지만, visual bbox bottom은 이전 text line bottom에서
    `LINE_ORDER_OVERLAP_MIN_PX` 이내에 머문다.
- 이 조건은 question title overlap이나 visual -> text 역순 후보에는 적용하지 않는다.

## 검증 결과

```bash
python3 -B -c "import ast, pathlib; ast.parse(pathlib.Path('scripts/task1274_visual_sweep.py').read_text())"
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage94_targeted
```

- AST 문법 확인: 통과
- targeted sweep:
  - `2024-09-between20`: `flagged=0/24`
  - `2024-11-practice-shape987`: `flagged=0/21`
  - `2024-11-practice-above0-between0-below0`: `flagged=0/21`
  - `2024-11-practice-no-separator-above20-between20-below20`: `flagged=0/23`
- p11 metrics:
  - `flags=[]`
  - `line_order_overlap_candidates=[]`

## 판단

Stage90~94의 visible separator 20mm 잔여 후보는 targeted 기준에서 모두 정리됐다. 이후에는 전체 sweep 또는
최종 문서 정리 단계에서 새 잔여 후보가 있는지 확인한다.
