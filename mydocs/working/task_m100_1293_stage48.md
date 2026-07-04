# Task 1293 Stage 48: 전체 미주 sweep 잔여 후보 재산정

## 목적

Stage47에서 9월 기본 계열의 compact 미주 tail line-box overflow 로그를 제거했다. 이번 단계에서는
전체 target을 다시 돌려 최신 기준의 잔여 후보를 재산정한다. Stage46의 잔여 목록은 Stage47 보정 전
결과이므로 그대로 다음 수정 대상으로 삼지 않는다.

## 확인 계획

- `python3 scripts/task1274_visual_sweep.py --target all --out output/task1293_stage48_full_sweep --rhwp-bin target/debug/rhwp`
- 각 target별로 다음 값을 확인한다.
  - SVG/PDF/render tree page count 1:1 여부
  - `overflow_lines`
  - `frame_overflow_pages`
  - `question_title_text_overlap_pages`
  - `line_order_overlap_pages`
  - `equation_text_overlap_pages`

## 상태

전체 sweep을 완료했다.

## 실행 결과

- 실행 완료: `output/task1293_stage48_full_sweep/summary.json`
- 전체 target 수: 15개
- 모든 target에서 SVG/PDF/render tree page count가 1:1로 일치했다.
- `frame_overflow_pages`, `question_title_text_overlap_pages`,
  `line_order_overlap_pages`, `equation_text_overlap_pages`는 전 target에서 비어 있었다.
- Stage47 보정 후 `2022-09`, `2024-09-below20`의 renderer overflow는 0건이 되었다.

| target | page count | overflow | frame | title | order | equation |
|---|---:|---:|---:|---:|---:|---:|
| `2022-09` | 23/23/23 | 0 | 0 | 0 | 0 | 0 |
| `2023-09` | 20/20/20 | 2 | 0 | 0 | 0 | 0 |
| `2024-09-below20` | 23/23/23 | 0 | 0 | 0 | 0 | 0 |
| `2024-09-between20` | 24/24/24 | 0 | 0 | 0 | 0 | 0 |
| `2024-09-below20-above20` | 23/23/23 | 3 | 0 | 0 | 0 | 0 |
| `2022-10` | 18/18/18 | 0 | 0 | 0 | 0 | 0 |
| `2022-11-practice` | 21/21/21 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-shape987` | 21/21/21 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-above0-between0-below0` | 21/21/21 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-above0-between7-below2` | 21/21/21 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-above0-between7-below20` | 21/21/21 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-above0-between20-below2` | 22/22/22 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-above20-between7-below2` | 21/21/21 | 0 | 0 | 0 | 0 | 0 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23/23/23 | 0 | 0 | 0 | 0 | 0 |

## 남은 후보

### `2023-09`

- page 18, col 0, `pi=934`, `Shape`, overflow 1081.5px
- page 18, col 1, `pi=951`, `Shape`, overflow 135.5px
- 큰 shape overflow 로그지만 frame 후보는 비어 있으므로, shape가 미주 frame 밖에 실제로 보이는지와
  `PageItem::Shape` overflow 판정이 문단 host line box를 과대 판정하는지 분리해야 한다.

### `2024-09-below20-above20`

- page 21, col 0, `pi=1156`, `FullParagraph`, overflow 10.0px
- page 21, col 0, `pi=1158`, `LAYOUT_OVERFLOW_DRAW`, overflow 64.0px
- page 21, col 0, `pi=1158`, `FullParagraph`, overflow 64.0px
- 핵심 visual 후보는 없지만 64px line box는 Stage47 허용폭을 넘으므로, 실제 PDF 대비 위치인지
  남은 line-box overreport인지 확인해야 한다.

## 다음 단계

Stage49에서는 `2023-09` shape overflow와 `2024-09-below20-above20` page 21 tail을 각각
compare/annotated/render tree로 확인한다. 실제 시각 문제가 아닌 로그 후보이면 판정 조건을 좁게
보정하고, 실제 배치 차이면 shape/tail flow 로직을 수정한다.
