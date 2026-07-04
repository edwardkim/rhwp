# Task 1293 Stage 35: 전체 미주 샘플 재 sweep

## 배경

Stage34에서 `2024-11-practice-above0-between20-below2`의 page 14 `pi=671` 하단 overflow를
정리했고, focused 세 target은 page count와 renderer overflow가 모두 통과했다.

Task 1293 goal은 미주 기능 완료이므로, Stage24 기준으로 남아 있던 전체 target을 다시 실행해
남은 문제를 재선정한다.

## 대상

- `scripts/task1274_visual_sweep.py --target all`
- 출력: `output/task1293_stage35_full_sweep`

## 확인 항목

1. SVG/PDF/render tree page count가 전 target에서 1:1인지 확인한다.
2. `LAYOUT_OVERFLOW` 로그가 남은 target을 우선순위로 정리한다.
3. hard 후보:
   - `frame_overflow_pages`
   - `question_title_text_overlap_pages`
   - `line_order_overlap_pages`
   - `equation_text_overlap_pages`
4. Stage33/34에서 정리한 세 target이 회귀하지 않았는지 확인한다.

## 상태

- 전체 sweep 실행 완료.

## 실행 결과

- 실행 결과: `output/task1293_stage35_full_sweep/summary.json`
- 전체 15개 target 모두 SVG/PDF/render tree page count가 1:1로 일치했다.
- 모든 target에서 hard 후보는 비어 있었다.
  - `frame_overflow_pages=[]`
  - `question_title_text_overlap_pages=[]`
  - `line_order_overlap_pages=[]`
  - `equation_text_overlap_pages=[]`
- Stage33/34 focused target은 회귀하지 않았다.
  - `2024-11-practice-no-separator-above20-between20-below20`: `23/23/23`, `overflow_lines=0`
  - `2024-11-practice-above0-between20-below2`: `22/22/22`, `overflow_lines=0`
  - `2024-11-practice-above20-between0-below20`: `21/21/21`, `overflow_lines=0`

## 남은 overflow target

| target | page count | overflow_lines | 대표 위치 |
|---|---:|---:|---|
| `2022-09` | 23/23/23 | 2 | page 17 `pi=948` |
| `2023-09` | 20/20/20 | 4 | page 12 `pi=695`, page 18 Shape `pi=934/951` |
| `2024-09-below20` | 23/23/23 | 2 | page 17 `pi=948` |
| `2024-09-below20-above20` | 23/23/23 | 6 | page 18 `pi=1006`, page 21 `pi=1156~1158` |
| `2024-11-practice-shape987` | 21/21/21 | 9 | page 11 `pi=571`, page 16 `pi=818~820` |
| `2024-11-practice-above0-between0-below0` | 21/21/21 | 16 | page 9 `pi=510`, page 10 `pi=537~539`, page 13 `pi=693`, page 16 `pi=853`, page 17 `pi=914` |
| `2024-11-practice-above20-between7-below2` | 21/21/21 | 15 | page 17 `pi=884~890` |

## 다음 우선순위

Stage36에서는 2024-11 새 설정 샘플 중 overflow 폭이 가장 큰
`2024-11-practice-above20-between7-below2` page 17 `pi=884~890` 연쇄를 먼저 분석한다.

- `구분선 위=20mm`, `미주 사이=7mm`, `구분선 아래=2mm`
- page 17 우측 단에서 FullParagraph 여러 개가 연속 overflow한다.
- 같은 설정군의 `above20-between0-below20`은 통과하므로, 단순 `구분선 위=20mm` 문제가 아니라
  기본 근방 `미주 사이=7mm`와 작은 `구분선 아래=2mm` 조합의 단 하단 flow 문제로 본다.
