# task 1293 stage93 - p18 equation tail 후보 과검출 분리

## 목적

stage92 후 `2024-09-between20`은 `flagged=2/24`까지 줄었다. 남은 p18 후보는 `문29`
`pi=922`의 `[EQ]` render-tree frame tail overflow이며, sweep metrics 기준 실제 픽셀 bleed는
tolerance 안에 있다. stage93에서는 이 후보가 렌더 pagination 문제인지 검출기 bbox 과검출인지
분리하고, 과검출이면 sweep 판정을 좁힌다.

## 시작 기준

- 브랜치: `local/task_m100_1293`
- 시작 커밋: `ad352f07 task 1293: visible separator equation tail 이월 보정`
- stage92 targeted sweep v2:
  - `2024-09-between20`: `flagged=2/24`, `tail=[18]`, `question=[]`
  - p18 candidate: `pi=922`, text=`[EQ]`, `overflow_px=24.5`
  - p18 actual bleed: `frame_overflow_tolerated_bleed=true`, `rhwp_outside_frame_bleed_px=6`, `pdf_outside_frame_pixels=0`

## 처리 방향

- render-tree bbox가 equation logical box를 frame 밖으로 크게 잡는지 확인한다.
- 실제 픽셀 bleed가 tolerance 안이고 question/marker drift가 없으면 pagination 보정으로 넘기지 않는다.
- 필요한 경우 visual sweep의 render-tree tail 후보 suppression 조건을 equation-only/tolerated bleed에 한해 좁게 추가한다.

## 검증 계획

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage93_targeted
git diff --check
```

CI 전체 테스트와 PR은 작업지시자 지시에 따라 수행하지 않는다.

## 작업지시자 승인

2026-06-14 작업지시자가 "자동 승인 할테니 계속 커밋하고 완료될때 까지 계속 수행. CI 전체 테스트나 PR 은 하지 않는다."라고 지시했다.

## 구현 기록

- `render_tree_frame_tail_candidates`가 잡은 bbox overflow와 실제 raster bottom bleed를 함께 보도록 suppression 입력을 확장했다.
- `equation_logical_box_bleed` 조건을 추가했다.
  - render-tree text에 `[EQ]`가 포함된다.
  - bbox overflow가 해당 line-height + bottom glyph bleed tolerance 안에 있다.
  - rhwp 실제 outside-frame bleed가 `FRAME_BOTTOM_GLYPH_BLEED_TOLERANCE_PX` 이하이다.
  - PDF outside-frame bleed도 같은 tolerance 이하이다.
  - question marker drift가 없다.
  - content bottom drift가 큰 drift 기준보다 작다.
- 이 조건은 pagination을 바꾸지 않고, 실제 픽셀과 PDF 기준으로 허용 가능한 수식 logical bbox tail만 sweep active 후보에서 제외한다.

## 검증 결과

```bash
python3 -m py_compile scripts/task1274_visual_sweep.py
python3 scripts/task1274_visual_sweep.py \
  --target 2024-09-between20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --target 2024-11-practice-no-separator-above20-between20-below20 \
  --out output/task1293_stage93_targeted
```

- `python3 -m py_compile scripts/task1274_visual_sweep.py`: 통과
- targeted sweep:
  - `2024-09-between20`: `flagged=1/24`, `tail=[]`, `question=[]`, 남은 페이지는 p11 하나
  - `2024-11-practice-shape987`: `flagged=0/21`
  - `2024-11-practice-above0-between0-below0`: `flagged=0/21`
  - `2024-11-practice-no-separator-above20-between20-below20`: `flagged=0/23`
- suppressed 확인:
  - `2024-09-between20` p18 `pi=922` `[EQ]`: `suppressed_reason=small_visual_tail_bleed`
  - `2024-11-practice-shape987` p12 `pi=592`: `suppressed_reason=small_visual_tail_bleed`

## 남은 후보

- `2024-09-between20` p11:
  - flags: `line_order_overlap`, `line_band_drift`, `column_line_band_drift`, `large_ink_region_drift`
  - candidate: `pi=573` text `따라서` 뒤 `pi=574` `[VISUAL]`이 더 위 y로 렌더되는 visual tail/order 후보
  - 다음 stage에서 render-tree line order 후보가 실제 pagination 문제인지 visual/TAC 저장 vpos 과검출인지 분리한다.
