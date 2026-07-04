# Task 1293 Stage 44: 전체 미주 설정 sweep 재검증

## 배경

Stage43에서 0/0/0 미주 profile의 render-tree overflow 6건을 renderer bbox bleed 판정으로
정리했다. focused 4종에서는 page count와 overflow 후보가 모두 안정적이었다.

## 목적

script에 등록된 전체 target을 다시 실행해 Stage43의 profile-specific renderer 보정이 다른
미주 설정 샘플에 영향을 주지 않았는지 확인한다. 동시에 남은 미주 기능 후보를 다음 stage의
우선순위로 재분류한다.

## 검증 계획

```bash
python3 scripts/task1274_visual_sweep.py \
  --target all \
  --out output/task1293_stage44_full_sweep \
  --rhwp-bin target/debug/rhwp
```

확인 항목:

- SVG/PDF/render tree 페이지 수 1:1 여부
- `overflow_lines`
- `frame_overflow_pages`
- `question_title_text_overlap_pages`
- `line_order_overlap_pages`
- `equation_text_overlap_pages`
- 미주 설정 샘플의 note shape 정규화값

## 상태

## 실행 결과

- 실행 완료: `output/task1293_stage44_full_sweep/summary.json`
- 전체 target 수: 15개
- `frame_overflow_pages`, `question_title_text_overlap_pages`,
  `line_order_overlap_pages`, `equation_text_overlap_pages`는 전 target에서 비어 있었다.
- Stage43에서 수정한 `2024-11-practice-above0-between0-below0`은 page count와
  render-tree overflow 모두 정상화되었다.
- 그러나 전체 sweep 기준으로는 아직 미주 기능 완료가 아니다.

| target | page count | overflow_lines | 판단 |
|---|---:|---:|---|
| `2022-09` | 23/23/23 | 5 | 기본 7mm 계열 page 15/16 하단 overflow 잔여 |
| `2023-09` | 20/20/20 | 4 | page 12 partial + page 18 Shape overflow 잔여 |
| `2024-09-below20` | 23/23/23 | 5 | `2022-09`와 같은 pi=872/873/931 계열 잔여 |
| `2024-09-between20` | 24/24/24 | 3 | 마지막 page partial tail overflow 잔여 |
| `2024-09-below20-above20` | 23/23/23 | 6 | page 18/21 하단 overflow 잔여 |
| `2022-10` | 18/18/18 | 0 | 정상 |
| `2022-11-practice` | 21/21/21 | 0 | 정상 |
| `2024-11-practice-shape987` | 21/21/21 | 0 | 정상 |
| `2024-11-practice-above0-between0-below0` | 21/21/21 | 0 | Stage43 보정 유지 |
| `2024-11-practice-above0-between7-below2` | 21/21/21 | 0 | 정상 |
| `2024-11-practice-above0-between7-below20` | 21/21/21 | 0 | 정상 |
| `2024-11-practice-above0-between20-below2` | 23/23/22 | 2 | page count mismatch. 최우선 수정 대상 |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 0 | 정상 |
| `2024-11-practice-above20-between7-below2` | 21/21/21 | 0 | 정상 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23/23/23 | 0 | 정상 |

## 주요 잔여 후보

### `2024-11-practice-above0-between20-below2`

- 대상: `samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwp`
- PDF: `pdf/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.pdf`
- note shape:
  - 구분선 위: 0mm
  - 미주 사이: 20mm
  - 구분선 아래: 2mm
- page count: SVG/render tree 23쪽, PDF 22쪽
- overflow:
  - `LAYOUT_OVERFLOW_DRAW: section=0 pi=671 line=2 y=1140.5 col_bottom=1092.3 overflow=48.2px`
  - `LAYOUT_OVERFLOW: page=14, sec=0, col=0, para=671, type=PartialParagraph, first=false, y=1140.5, bottom=1092.3, overflow=48.2px`

이 target은 page count 자체가 맞지 않으므로, 후속 stage에서 가장 먼저 분석한다.
미주 사이 20mm가 큰데 구분선 위가 0mm이고 구분선 아래가 2mm인 profile이라,
큰 `between-notes` gap을 어느 경계에서 pagination 높이로 소비하고 어느 경계에서
저장 vpos에 흡수해야 하는지 다시 확인해야 한다.

## 다음 단계

Stage45에서는 `2024-11-practice-above0-between20-below2`를 단일 target으로 분석한다.

- PDF 22쪽에 비해 rhwp가 23쪽으로 늘어나는 첫 분기 위치를 찾는다.
- `pi=671` 전후 문항/미주 흐름을 render tree, compare PNG, note shape 값으로 대조한다.
- 수정은 개별 paragraph가 아니라 `구분선 위=0 + 미주 사이=20 + 구분선 아래=2`에서
  큰 `미주 사이` 값이 단/쪽 분기 계산에 들어가는 공통 조건으로 적용한다.
