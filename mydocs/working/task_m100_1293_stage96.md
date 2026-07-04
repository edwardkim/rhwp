# task 1293 stage96 - question title overlap 후보 분류

## 목적

stage95 full sweep에서 여러 target에 `question_title_text_overlap` 후보가 반복됐다. stage96에서는
이 후보들이 실제 문항 제목/본문 겹침인지, equation/visual logical bbox 또는 line box 과검출인지
분류한다.

## 시작 기준

- 브랜치: `local/task_m100_1293`
- 시작 커밋: `3705a019 task 1293: 전체 visual sweep 잔여 정리`
- stage95 full sweep:
  - 0 target: `2024-09-between20`, `2024-11-practice-shape987`,
    `2024-11-practice-above0-between0-below0`, `2024-11-practice-above20-between0-below20`,
    `2024-11-practice-no-separator-above20-between20-below20`
  - 반복 후보: `question_title_text_overlap`

## 처리 방향

- `question_title_text_overlap_candidates`의 title/next bbox와 text를 target별로 모은다.
- 실제 PNG compare/annotated에서 title과 다음 line이 겹치는지 확인한다.
- 과검출이면 sweep 조건을 좁히고, 실제 겹침이면 typeset/render 흐름을 수정한다.

## 검증 계획

```bash
python3 scripts/task1274_visual_sweep.py --target all --out output/task1293_stage96_full_sweep
git diff --check
```

CI 전체 테스트와 PR은 작업지시자 지시에 따라 수행하지 않는다.

## 작업지시자 승인

2026-06-14 작업지시자가 자동 승인과 연속 커밋 진행을 지시했다.

## 조사 기록

`question_title_text_overlap_candidates`를 모은 결과, 모든 후보가 title line box와 다음 line box의
1.9~2.0px 접촉이었다.

예:

| target/page | title | next | overlap |
| --- | --- | --- | --- |
| `2023-09` p14 | `문21）` | `[EQ]이 등차수열...` | ratio 0.158 |
| `2024-09-below20-above20` p14 | `문22）` | `[EQ][EQ]` | ratio 0.167 |
| `2022-11-practice` p11 | `문12）` | `(ⅰ) [EQ]` | ratio 0.167 |

compare/annotated 기준 실제 제목과 본문이 겹친다기보다 HWP line box가 2px 정도 접촉하는 정상 배치로
판단했다.

## 구현 기록

- `QUESTION_TITLE_OVERLAP_MIN_PX = 3.0`을 추가했다.
- `render_tree_question_title_overlap_candidates`가 ratio뿐 아니라 실제 y-overlap 높이도 보게 했다.
- title/next line box의 overlap height가 3px 미만이면 question title overlap 후보로 보지 않는다.
- 후보 기록에는 `overlap_px`를 함께 남긴다.

## 검증 결과

```bash
python3 -B -c "import ast, pathlib; ast.parse(pathlib.Path('scripts/task1274_visual_sweep.py').read_text())"
python3 scripts/task1274_visual_sweep.py --target all --out output/task1293_stage96_full_sweep
```

- AST 문법 확인: 통과
- full sweep:
  - stage95: 총 flagged page 38
  - stage96: 총 flagged page 26
  - `question_title_text_overlap`: 전부 제거
- 새로 0이 된 target:
  - `2022-11-practice`: `2/21 -> 0/21`
  - `2024-11-practice-above0-between7-below2`: `2/21 -> 0/21`
  - `2024-11-practice-above20-between7-below2`: `2/21 -> 0/21`
- 계속 0 유지:
  - `2024-09-between20`
  - `2024-11-practice-shape987`
  - `2024-11-practice-above0-between0-below0`
  - `2024-11-practice-above20-between0-below20`
  - `2024-11-practice-no-separator-above20-between20-below20`

## 남은 후보

| target | flagged | pages | 주 flags |
| --- | ---: | --- | --- |
| `2022-09` | 2 | 16, 20 | tail/question/red/line/large |
| `2023-09` | 3 | 11, 13, 19 | tail/question/red/line/column/large |
| `2024-09-below20` | 4 | 9, 13, 16, 20 | equation/tail/question/red/line/column/large |
| `2024-09-below20-above20` | 3 | 19, 20, 22 | tail/question/red/line/column/large |
| `2022-10` | 2 | 12, 14 | tail/question/red/column/large |
| `2024-09-below20above20` | 3 | 19, 20, 22 | `2024-09-below20-above20` 중복 key와 동일 |
| `2024-11-practice-above0-between7-below20` | 2 | 14, 20 | tail/question/red/column/large |
| `2024-11-practice-above0-between20-below2` | 7 | 13, 15, 17, 18, 20, 21, 22 | tail/question/content-bottom/line/column/large |

## 판단

반복 title overlap 후보는 sweep 과검출로 정리됐다. 다음 stage에서는 남은 후보 대부분에 공통으로 포함되는
`render_tree_frame_tail_overflow + question_marker_drift` 패턴을 우선 분류한다.
