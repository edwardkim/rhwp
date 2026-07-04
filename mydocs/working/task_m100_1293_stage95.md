# task 1293 stage95 - 전체 visual sweep 및 최종 정리

## 목적

stage94 후 주요 targeted 샘플 4개는 모두 `flagged=0`이다. stage95에서는 전체 visual sweep으로
targeted 범위 밖의 잔여 후보가 있는지 확인하고, 결과에 따라 추가 stage 또는 최종 보고서 작성으로
넘어간다.

## 시작 기준

- 브랜치: `local/task_m100_1293`
- 시작 커밋: `7db2200e task 1293: visual order sweep 과검출 보정`
- stage94 targeted sweep:
  - `2024-09-between20`: `flagged=0/24`
  - `2024-11-practice-shape987`: `flagged=0/21`
  - `2024-11-practice-above0-between0-below0`: `flagged=0/21`
  - `2024-11-practice-no-separator-above20-between20-below20`: `flagged=0/23`

## 검증 계획

```bash
python3 scripts/task1274_visual_sweep.py --target all --out output/task1293_stage95_full_sweep
git diff --check
```

CI 전체 테스트와 PR은 작업지시자 지시에 따라 수행하지 않는다.

## 작업지시자 승인

2026-06-14 작업지시자가 자동 승인과 연속 커밋 진행을 지시했다.

## 검증 결과

```bash
python3 scripts/task1274_visual_sweep.py --target all --out output/task1293_stage95_full_sweep
```

- 실행 완료: `output/task1293_stage95_full_sweep/summary.json`
- targeted에서 다룬 핵심 4개는 full sweep에서도 유지:
  - `2024-09-between20`: `flagged=0/24`
  - `2024-11-practice-shape987`: `flagged=0/21`
  - `2024-11-practice-above0-between0-below0`: `flagged=0/21`
  - `2024-11-practice-no-separator-above20-between20-below20`: `flagged=0/23`
- 추가로 `2024-11-practice-above20-between0-below20`: `flagged=0/21`

## 전체 sweep 잔여 매트릭스

| target | flagged | pages | 주 flags |
| --- | ---: | --- | --- |
| `2022-09` | 2 | 16, 20 | tail/question/red/line/large |
| `2023-09` | 4 | 11, 13, 14, 19 | question/title/tail/line/column/large |
| `2024-09-below20` | 4 | 9, 13, 16, 20 | equation/tail/question/red/line/column/large |
| `2024-09-below20-above20` | 4 | 14, 19, 20, 22 | title/tail/question/red/line/column/large |
| `2022-10` | 3 | 10, 12, 14 | title/question/tail/red/column/large |
| `2022-11-practice` | 2 | 11, 16 | title/line/column/large |
| `2024-09-below20above20` | 4 | 14, 19, 20, 22 | `2024-09-below20-above20` 중복 key와 동일 |
| `2024-11-practice-above0-between7-below2` | 2 | 11, 16 | title/line/column/large |
| `2024-11-practice-above0-between7-below20` | 4 | 12, 14, 16, 20 | title/tail/question/red/column/large |
| `2024-11-practice-above0-between20-below2` | 7 | 13, 15, 17, 18, 20, 21, 22 | tail/question/content-bottom/line/column/large |
| `2024-11-practice-above20-between7-below2` | 2 | 12, 16 | title/line/column/large |

## 판단

- Stage90~94에서 정리한 `2024-09-between20`/`shape987`/zero/no-separator 핵심 targeted 범위는 안정화됐다.
- full sweep은 아직 `question_title_text_overlap` 반복 후보와 `render_tree_frame_tail_overflow + question_marker_drift`
  후보를 보여준다.
- 다음 stage에서는 여러 target에서 반복되는 `question_title_text_overlap` 후보를 먼저 분류한다.
- CI 전체 테스트와 PR은 작업지시자 지시에 따라 수행하지 않았다.
