---
kind: implementation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-04
---

# Task #3820 Stage 2 — RowBreak paint-footprint 보정

## 입력 분석과 변경 경계

Stage 1 commit `38cb3f802`의 p94/p106 trace를 근거로 한다. 대상은 native HWP5의
비-TAC `TopAndBottom`·`Para` `RowBreak` 표 중 다음 source contract를 모두 만족하는
형상으로 한정한다.

- 현재 흐름이 body height의 후반부이고, 다음 문단 stored `vpos`가 table anchor보다 위로
  rewind된다.
- cell은 모두 `row_span=1`이고 table cell footnote가 없다.
- `MeasuredTable`의 row footprint가 current `FormattedTable::effective_height`보다 크다.

변경은 two-space model을 명시적으로 분리한다.

1. whole-fit gate와 full-row fragment fit은 `max(cut_row_height, measured_row_height)`를
   사용한다. p94의 축소된 `322.6px` whole-fit와 p106의 `655.0px` scanner footprint를
   renderer paint footprint에 맞춘다.
2. 이미 잘린 row의 cell-unit offset과 intra-row continuation은 기존 `cut_row_height`를
   유지한다. 즉 row cut source ownership을 임의로 바꾸지 않는다.
3. 이 범위 밖(HWPX, page-top, rowspan, table-cell footnote, no-rewind)은 기존 code path를
   유지한다.

## 구현 중간 관찰 — first-fragment paint footer guard

첫 probe에서 p94는 PDF와 같이 row 0–2만 p94에 두고 row 3을 p95로 이월했다. p106도
기존의 row 4 partial paint와 `38.8px` footer overflow는 제거했다. 그러나 p106의
painted row 0–3 합은 `363.7px`, first fragment available은 `367.6px`여서 불과
`3.9px`의 slack으로 row 3을 여전히 p106에 보존한다. 기준 PDF는 row 0–2 뒤에
fragment를 끝낸다.

따라서 same native HWP5 source contract의 **first, whole-row fragment**에만 `4px`
paint-footer guard를 적용한다. 이는 전역 margin이 아니라 paint footprint를 기준으로
footer 경계에 남겨야 하는 source-local slack이다. continuation fragment, partial row,
HWPX, rowspan, table cell footnote에는 적용하지 않는다. 이 guard 뒤 p106 row 3이 p107로
이월되는지와 p94가 변하지 않는지를 직접 render-tree로 확인한다.

## 수용 기준

1. p94는 표 28의 마지막 `Unspecified Donation` row를 p95 continuation으로 이월한다.
2. p106은 footer/body 영역을 넘는 `LAYOUT_OVERFLOW` 없이 PDF row owner와 같아진다.
3. p107–108에서 body tail과 그림 52의 owner를 다시 확인한다. 남아 있으면 해결로
   표시하지 않고 Stage 3 분석으로 이월한다.
4. #3821 p156 Square wrap regression과 p127 pre-existing resolved case가 계속 통과한다.

## 검증 순서

```text
cargo fmt --check
cargo test --profile release-test --lib issue_3821_square_picture_wrap_band_is_bounded_and_contiguous
cargo test --profile release-test --lib issue_3821_page_tail_square_picture_wrap_reaches_visible_text_after_guides
rhwp export-render-tree ... --page 93,94,105,106,107
python3 scripts/visual_sweep.py ... --pages 94,106-108,127,156
git diff --check
```

모든 cargo 명령은 `CARGO_TARGET_DIR=target/task-3820-3821-fidelity` 및
`CARGO_INCREMENTAL=0`으로 실행한다. 결과를 확인한 뒤 이 문서에 evidence와 판정을
추가하고 Stage 2 코드와 함께 커밋한다.

## 결과와 증적 (2026-08-04)

release-test binary로 human p94/p95/p106/p107 render-tree를 다시 산출했다. `-p`는
0-based이므로 각각 `93, 94, 105, 106`을 지정했다.

| 기준 | render-tree 결과 | 판정 |
| --- | --- | --- |
| p94 표 28 (`pi=1000`) | row `0,1,2` | PDF와 일치 |
| p95 표 28 (`pi=1000`) | row `3` | PDF와 일치 |
| p106 표 29 (`pi=1136`) | row `0,1,2` | PDF와 일치 |
| p107 표 29 (`pi=1136`) | row `3,4,5,6,7` | PDF와 일치 |

직접 비교 PNG는 다음에 보관했다. 좌측은 rhwp SVG raster, 우측은 한컴 PDF raster다.

- `output/task-3820-3821-fidelity/stage2-review-pairs/p94_rhwp_pdf.png`
- `output/task-3820-3821-fidelity/stage2-review-pairs/p106_rhwp_pdf.png`
- `output/task-3820-3821-fidelity/stage2-review-pairs/p107_rhwp_pdf.png`

focused regression은 다음 summary를 확인했다.

```text
issue_3820_rewinding_rowbreak_uses_painted_first_fragment_boundary ... ok
1 passed; 0 failed

issue_3821_square_picture_wrap_band_is_bounded_and_contiguous ... ok
issue_3821_page_tail_square_picture_wrap_reaches_visible_text_after_guides ... ok
각각 1 passed; 0 failed
```

### 잔여 결함 — 해결로 표시하지 않음

p108에서 그림 52의 caption은 있으나 이미지가 rhwp raster에 없다. 한컴 PDF에는 그림과
caption이 모두 p108에 있다. 따라서 p107–108 전체 경계는 수용하지 않으며, 이 결과와
p156의 그림 64 본문 침범을 Stage 3의 picture placement/wrap geometry 분석으로 이월한다.
`p108_rhwp_pdf.png`가 직접 비교 증적이다.
