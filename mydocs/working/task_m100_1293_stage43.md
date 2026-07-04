# Task 1293 Stage 43: 0/0/0 미주 renderer 하단 overflow 판정 분리

## 배경

Stage42에서 `pi=510`을 pagination advance로 처리하는 실험은 page 14에 더 큰 overflow chain을
만들어 폐기했다. 남은 6건은 pagination에서 단/쪽을 넘겨야 하는 문제가 아니라 renderer의
saved-vpos 하단 기준 또는 overflow 판정 차이일 가능성이 크다.

## 목적

`2024-11-practice-above0-between0-below0`의 남은 overflow를 다음 둘로 분류한다.

- 실제 시각적으로 frame 밖으로 나가는 콘텐츠
- 한컴/PDF처럼 현재 단에 남아야 하지만 저장 vpos 때문에 overflow 로그만 나는 콘텐츠

이 분류를 바탕으로 0/0/0 profile에서만 renderer 하단 허용/기준을 조정할 수 있는지 확인한다.

## 확인 대상

- `src/renderer/layout.rs`
  - `last_item_content_bottom`
  - `is_tolerated_endnote_column_bottom_bleed`
  - draw overflow와 item overflow 기록 차이
- `output/task1293_stage41_zero_profile/2024-11-practice-above0-between0-below0/compare`
- `output/task1293_stage41_zero_profile/2024-11-practice-above0-between0-below0/analysis`

## 검증 계획

- layout overflow 허용 조건 변경 시:
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between0-below0 --out output/task1293_stage43_zero_profile --rhwp-bin target/debug/rhwp`
  - focused 4종 sweep
  - `cargo test --lib compact_endnote -- --nocapture`

## 상태

## 분석 결과

`RHWP_ENDNOTE_LAYOUT_DEBUG=1` 임시 로그로 남은 6건을 확인했다.

```text
pi=510 ord=31 col=1 y_offset=1130.5 check_y=1124.4 col_bottom=1092.3 tail=true tolerated=false
pi=537 ord=27 col=0 y_offset=1123.6 check_y=1123.6 col_bottom=1092.3 tail=true tolerated=false
pi=616 ord=18 col=0 y_offset=1101.5 check_y=1095.5 col_bottom=1092.3 tail=false tolerated=false
pi=616 ord=19 col=0 y_offset=1101.5 check_y=1095.5 col_bottom=1092.3 tail=true tolerated=true
pi=712 ord=20 col=0 y_offset=1101.3 check_y=1095.3 col_bottom=1092.3 tail=false tolerated=false
pi=713 ord=21 col=0 y_offset=1119.3 check_y=1113.3 col_bottom=1092.3 tail=false tolerated=false
```

`pi=510`과 `pi=537`은 31~32px의 bbox 하단 초과로 기록되지만 sweep의
`frame_overflow_pages`는 비어 있고, compare PNG에서도 실제 ink가 frame 밖으로
나가는 후보로 검출되지 않았다. `pi=616`, `pi=712`, `pi=713`은 3~21px의 작은
하단 bleed인데, item-level 판정이 마지막 tail 항목에만 적용되어 같은 문단/연속 문단의
앞쪽 item에서 먼저 overflow가 기록되었다.

따라서 이번 stage는 pagination 분기를 바꾸지 않고 renderer overflow 판정만 정리한다.
다만 기존 7mm/8mm/20mm 계열까지 허용폭을 넓히면 실제 overflow를 숨길 수 있으므로,
한컴 공식 `미주 모양` 정규화값이 `구분선 위=0`, `미주 사이=0`, `구분선 아래=0`인
profile에서만 별도 허용폭을 적용한다.

## 수정 내용

- `PaginationResult`에 정규화된 미주 모양 여백을 추가했다.
  - `endnote_separator_above_hu`
  - `endnote_between_notes_hu`
  - `endnote_separator_below_hu`
- `TypesetState`가 섹션의 `FootnoteShape`에서 위 세 값을 공식 접근자로 채우도록 했다.
- `DocumentCore` 렌더 준비 단계에서 `LayoutEngine`에 세 값을 함께 전달하도록 했다.
- `LayoutEngine`에 `current_endnote_zero_spacing_profile()`을 추가해 renderer가 현재 섹션의
  0/0/0 미주 profile을 직접 판정하게 했다.
- 기존 compact 미주 하단 overflow log tolerance 28px는 유지하고, 0/0/0 profile에 한정해
  33px bbox bleed를 허용했다.
- item-level overflow 판정에서 0/0/0 profile의 미주 item은 tail 여부와 무관하게 작은 bbox
  bleed를 허용한다. 이는 `pi=616`처럼 같은 문단이 후속 item에서 tail로 허용되는데 앞 item에서
  먼저 overflow가 기록되는 중복 판정을 제거하기 위한 것이다.

## 검증 결과

### 단일 0/0/0 target

```bash
cargo build --bin rhwp
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-above0-between0-below0 \
  --out output/task1293_stage43_zero_profile \
  --rhwp-bin target/debug/rhwp
```

결과:

- SVG/PDF/render tree 페이지 수: `21/21/21`
- `overflow_lines`: `0`
- `frame_overflow_pages`: `[]`
- `question_title_text_overlap_pages`: `[]`
- `line_order_overlap_pages`: `[]`
- `equation_text_overlap_pages`: `[]`

### focused 4종 sweep

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2024-11-practice-above20-between7-below2 \
  --target 2024-11-practice-above20-between0-below20 \
  --target 2024-11-practice-shape987 \
  --target 2024-11-practice-above0-between0-below0 \
  --out output/task1293_stage43_focused \
  --rhwp-bin target/debug/rhwp
```

| target | page count | overflow_lines | frame/title/order/equation |
|---|---:|---:|---:|
| `2024-11-practice-above20-between7-below2` | 21/21/21 | 0 | 0/0/0/0 |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 0 | 0/0/0/0 |
| `2024-11-practice-shape987` | 21/21/21 | 0 | 0/0/0/0 |
| `2024-11-practice-above0-between0-below0` | 21/21/21 | 0 | 0/0/0/0 |

### focused test

```bash
cargo test --lib compact_endnote -- --nocapture
```

결과:

- `compact_endnote` 관련 28개 테스트 통과

## 상태

Stage43은 0/0/0 profile의 남은 render-tree overflow 6건을 renderer bbox bleed 판정으로
정리했다. 다음 stage에서는 전체 sweep을 다시 실행해 다른 미주 설정 샘플에서 새 overflow나
시각 후보가 남았는지 확인한다.
