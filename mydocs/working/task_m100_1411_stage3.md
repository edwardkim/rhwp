# Task 1411 Stage 3 — `2022-10` p14 textless equation tail gap 보정

## 목적

Stage 2에서 분류한 `2022-10` p14의 실제 layout 결함을 보정한다.

문23의 textless tall equation tail 뒤 문24 제목 배치에서 `HeightCursor`가 visible content
bottom 기준 gap을 이미 확보했는데, layout 단계가 직전 `line_spacing`을 다시 note gap으로
보존해 문24 전체를 아래로 밀고 문25 본문과 겹치게 하는 문제를 좁게 막는다.

## 수정 지점

- `src/renderer/layout.rs`
  - `should_preserve_endnote_title_gap` 계산 직전
  - 직전 문단이 보이는 텍스트 없는 equation tail이고, 단 하단부에서 현재 제목 위치가
    `prev_item_content_bottom_y + prev_endnote_title_gap_px` 이상이면 추가 gap 보존을 생략한다.
  - 같은 미주 본문까지는 이 생략을 유지하되, 다음 미주 제목을 만나면 생략한 폭만큼
    `HeightCursor` vpos base를 지연 복원해 후속 문항 전체가 같이 위로 당겨지지 않게 한다.

## 기대 효과

- `2022-10` p14에서 문24 제목과 본문 tail을 위로 회복해 문25 영역 침범을 제거한다.
- 일반 미주 제목 gap 보존 로직은 유지한다.
- 단 하단부가 아니거나 content bottom 기준으로 충분한 gap이 없는 케이스는 기존 보존 경로를 그대로 탄다.

## 구현 기록

1차 보정은 textless equation tail 뒤 gap 보존만 생략했다. 이 경우 `2022-10` p14의
`equation_text_overlap`은 사라졌지만, 같은 7mm compact profile의 p10 문12가 아래로 밀려
`render_tree_frame_tail_overflow` 후보가 새로 생겼다.

최종 보정은 조건을 단 하단부(`y_before_vpos > col_area.y + col_area.height * 0.65`)로
좁혔다. p10은 기존 위치로 돌아왔고, p14만 보정된다.

## 결과

`output/task1411_stage3_after_fix_v2` 기준:

| target | baseline | Stage 3 | 비고 |
| --- | ---: | ---: | --- |
| `2022-10` | 1/18 | 0/18 | p14 `equation_text_overlap` 해소 |
| `2024-09-below20-above20` | 3/23 | 3/23 | baseline 잔여 유지 |
| `2024-11-practice-above0-between20-below2` | 3/22 | 3/22 | baseline 잔여 유지 |

`2022-10` p14 주요 위치:

| pi | 내용 | Stage 1 y | Stage 3 y |
| --- | --- | ---: | ---: |
| 773 | 문24 제목 | 841.1 | 822.7 |
| 775 | 문24 tail | 939.3 | 920.8 |
| 776 | 문25 제목 | 940.3 | 947.9 |
| 777 | 문25 첫 본문 | 958.4 | 965.9 |

문24 tail과 문25 첫 본문 사이의 수식/쉼표 bbox 교차가 사라졌다.

## 검증

- `cargo fmt --check`
- `cargo build --bin rhwp`
- targeted sweep:

```bash
python3 scripts/task1274_visual_sweep.py \
  --target 2022-10 \
  --target 2024-09-below20-above20 \
  --target 2024-11-practice-above0-between20-below2 \
  --out output/task1411_stage3_after_fix_v2 \
  --rhwp-bin target/debug/rhwp
```

- `git diff --check`

완료:

- `cargo fmt --check`: 통과
- `cargo build --bin rhwp`: 통과
- `cargo test --lib compact_endnote_question_title_after_tall_tail_limited_backtrack`: 통과
- `python3 scripts/task1274_visual_sweep.py ... --out output/task1411_stage3_after_fix_v2`: 통과
- `git diff --check`: 통과
