# Task 1293 Stage 46: 전체 미주 sweep 재검증

## 배경

Stage45에서 `2024-11-practice-above0-between20-below2`의 page count 회귀와
`pi=671` overflow를 복구했다.

- `tac_picture_rewinds_before_column_base`는 compact 미주 사이 profile에서만 적용하도록 제한했다.
- internal rewind split-1 보정은 compact 흡수형과 흡수되지 않은 큰 미주 사이 profile에 모두
  적용하도록 정리했다.

## 목적

전체 target sweep을 다시 수행해 Stage45 수정이 전체 미주 설정 샘플과 기존 교육 통합 샘플에
어떤 영향을 줬는지 확인한다. 특히 Stage44에서 남았던 기존 교육 통합 계열 overflow 후보를
다음 stage에서 수정할 우선순위로 재분류한다.

## 검증 계획

```bash
python3 scripts/task1274_visual_sweep.py \
  --target all \
  --out output/task1293_stage46_full_sweep \
  --rhwp-bin target/debug/rhwp
```

확인 항목:

- SVG/PDF/render tree page count
- `overflow_lines`
- `frame_overflow_pages`
- `question_title_text_overlap_pages`
- `line_order_overlap_pages`
- `equation_text_overlap_pages`
- Stage45 target의 page count/overflow 회귀 여부

## 상태

## 실행 결과

- 실행 완료: `output/task1293_stage46_full_sweep/summary.json`
- 전체 15개 target 모두 SVG/PDF/render tree page count가 일치했다.
- `frame_overflow_pages`, `question_title_text_overlap_pages`,
  `line_order_overlap_pages`, `equation_text_overlap_pages`는 전 target에서 비어 있었다.
- Stage45 대상 `2024-11-practice-above0-between20-below2`는 `22/22/22`, overflow 0으로
  정상화 상태를 유지했다.
- `2024-09-between20`도 Stage44의 overflow 3건에서 0건으로 정리되어 있었다.

| target | page count | overflow_lines | frame/title/order/equation |
|---|---:|---:|---:|
| `2022-09` | 23/23/23 | 5 | 0/0/0/0 |
| `2023-09` | 20/20/20 | 4 | 0/0/0/0 |
| `2024-09-below20` | 23/23/23 | 5 | 0/0/0/0 |
| `2024-09-between20` | 24/24/24 | 0 | 0/0/0/0 |
| `2024-09-below20-above20` | 23/23/23 | 6 | 0/0/0/0 |
| `2022-10` | 18/18/18 | 0 | 0/0/0/0 |
| `2022-11-practice` | 21/21/21 | 0 | 0/0/0/0 |
| `2024-11-practice-shape987` | 21/21/21 | 0 | 0/0/0/0 |
| `2024-11-practice-above0-between0-below0` | 21/21/21 | 0 | 0/0/0/0 |
| `2024-11-practice-above0-between7-below2` | 21/21/21 | 0 | 0/0/0/0 |
| `2024-11-practice-above0-between7-below20` | 21/21/21 | 0 | 0/0/0/0 |
| `2024-11-practice-above0-between20-below2` | 22/22/22 | 0 | 0/0/0/0 |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 0 | 0/0/0/0 |
| `2024-11-practice-above20-between7-below2` | 21/21/21 | 0 | 0/0/0/0 |
| `2024-11-practice-no-separator-above20-between20-below20` | 23/23/23 | 0 | 0/0/0/0 |

## 남은 후보

### `2022-09` / `2024-09-below20`

두 target은 같은 구조의 overflow를 낸다.

- page 15 우측 단:
  - `pi=872` FullParagraph 17.6px
  - `pi=873` FullParagraph 35.7px
- page 16 우측 단:
  - `pi=931` PartialParagraph line 3, 43.3px

`2024-09-below20`은 `2022-09`와 같은 문항 흐름에 `구분선 아래`만 바꾼 샘플이므로,
다음 stage에서는 이 공통 축을 먼저 분석한다.

### `2023-09`

- page 12:
  - `pi=695` PartialParagraph 28.3px
- page 18:
  - `pi=934` Shape 1081.5px
  - `pi=951` Shape 135.5px

Shape overflow가 크므로 `2022-09` 공통 text/partial 후보 다음 우선순위로 둔다.

### `2024-09-below20-above20`

- page 18:
  - `pi=1006` FullParagraph 5.6px
- page 21:
  - `pi=1156` FullParagraph 10.0px
  - `pi=1157` FullParagraph 30.4px
  - `pi=1158` FullParagraph 64.0px

구분선 위/아래가 모두 큰 profile이라, 기존 9월 기본 계열을 먼저 정리한 뒤 별도 stage에서 본다.

## 다음 단계

Stage47에서는 `2022-09`와 `2024-09-below20`에 공통으로 남은 page 15/16 overflow를
분석한다. 우선 `pi=872`, `pi=873`, `pi=931`의 render tree/dump-pages/PDF compare를 대조한다.
