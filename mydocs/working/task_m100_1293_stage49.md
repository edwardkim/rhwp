# Task 1293 Stage 49: 잔여 shape/text overflow 후보 분석

## 목적

Stage48 전체 sweep 기준 남은 renderer overflow 후보는 두 target에 한정된다. 이번 단계에서는
각 후보가 실제 PDF 대비 시각 문제인지, 아니면 line box/shape host 기반 overflow 판정 문제인지
분리한다.

## 대상

### `2023-09`

- output: `output/task1293_stage48_full_sweep/2023-09`
- render log `page=18`은 0-based page index이므로 실제 확인 파일은 `render_tree_019.json`,
  `compare_019.png`, `annotated_019.png`이다.
  - `pi=934`, `Shape`, overflow 1081.5px
  - `pi=951`, `Shape`, overflow 135.5px

### `2024-09-below20-above20`

- output: `output/task1293_stage48_full_sweep/2024-09-below20-above20`
- render log `page=21`은 0-based page index이므로 실제 확인 파일은 `render_tree_022.json`,
  `compare_022.png`, `annotated_022.png`이다.
  - `pi=1156`, `FullParagraph`, overflow 10.0px
  - `pi=1158`, `LAYOUT_OVERFLOW_DRAW`, overflow 64.0px
  - `pi=1158`, `FullParagraph`, overflow 64.0px

## 확인 계획

1. 각 target의 `compare_018.png`, `annotated_018.png`, `compare_021.png`, `annotated_021.png`를
   열어 실제 frame 밖 ink/shape 여부를 확인한다.
2. `render_tree_018.json`, `render_tree_021.json`에서 해당 `pi`의 bbox와 node type을 확인한다.
3. 실제 시각 문제가 아니면 overflow 판정 조건을 공통 미주/shape tail 조건으로 보정한다.
4. 실제 시각 문제이면 위치 계산 또는 shape/tail flow 로직을 수정한다.

## 상태

완료했다.

## 분석

### `2023-09` page 19

`pi=934`, `pi=951`은 화면상 frame 안에 정상 배치되어 있었고, render tree에서도 실제
`Image` bbox 하단이 column bottom 안에 들어왔다.

- `pi=934`: 실제 `Image` bbox는 `y=806.6`, `h=259.8`로 하단이 약 `1066.4px`
- `pi=951`: 실제 `Image` bbox는 `y=872.9`, `h=191.7`로 하단이 약 `1064.6px`

하지만 `PageItem::Shape` 후처리 경로에서 paragraph_layout이 이미 emit한 인라인 그림의
`registered_inline_pos`는 사용하면서도, content bottom 높이는 `shape_attr.current_height`까지
`max`한 값을 사용했다. 그 결과 실제 그림 높이보다 큰 logical height가 `last_item_content_bottom`
에 기록되어 renderer overflow 로그만 남았다.

수정은 이미 등록된 inline picture에 한정했다. paragraph_layout이 emit한 실제 bbox 높이는
`common.height` 기준이므로, registered inline picture의 overflow 판정용 content bottom과 caption
기준 y도 `registered_y + common.height` 경로로 계산한다. 미등록 picture 경로는 기존
`common.height/current_height` 기준을 유지했다.

### `2024-09-below20-above20` page 22

`pi=1156`은 같은 미주 묶음의 후속 문단을 가진 tail cluster였고, `10px` bottom bleed는 실제
draw overflow 없이 다음 항목과 같은 미주 흐름으로 이어지는 경우였다. 기존 tail 판정은 column
마지막 또는 마지막 직전 항목 중심이라 같은 미주 후속 문단을 가진 중간 tail을 놓쳤다.

`pi=1158`은 공백 텍스트와 수식만 있는 마지막 미주 line-box였다. 실제 ink보다 line box가 큰
수식-only TAC 줄이라 `64px` overflow 로그가 남았지만, PDF/PNG 비교에서는 실제 frame 밖 ink가
없었다.

수정은 두 가지로 나눴다.

- 같은 미주 후속 문단이 있는 항목도 tail item으로 인정해 작은 bottom bleed를 허용한다.
- 미주 마지막 줄이 공백 텍스트 + 수식 TAC만 가진 경우 별도 line-box tolerance를 사용한다.

## 검증

- `cargo fmt --all -- --check`: 통과
- `cargo test --lib compact_endnote -- --nocapture`: 29개 통과
- `cargo build --bin rhwp`: 통과
- focused sweep:
  - `python3 scripts/task1274_visual_sweep.py --target 2023-09 --target 2024-09-below20-above20 --out output/task1293_stage49_resweep --rhwp-bin target/debug/rhwp`
  - 두 target 모두 SVG/PDF/render tree page count가 1:1이다.
  - 두 target 모두 `render_tree.log`의 `LAYOUT_OVERFLOW`가 0건이다.
- full sweep:
  - `python3 scripts/task1274_visual_sweep.py --target all --out output/task1293_stage49_full_sweep --rhwp-bin target/debug/rhwp`
  - 전체 15개 target에서 SVG/PDF/render tree page count가 모두 1:1이다.
  - 전체 15개 target에서 renderer `LAYOUT_OVERFLOW`가 총 0건이다.

## 판단

Stage24에서 발견된 renderer overflow 잔여 후보는 Stage49 기준 모두 사라졌다. 자동 sweep의
red/line/large drift 후보는 계속 표시되지만, 이번 단계의 수정 범위였던 renderer overflow
후보와 page count 회귀는 해소됐다.
