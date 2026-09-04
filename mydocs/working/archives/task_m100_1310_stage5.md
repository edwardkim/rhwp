# Task #1310 Stage 5 - 문단 내어쓰기 원점 기준 재정리

## 1. 작업지시자 피드백

Stage 4 이후 작업지시자가 다음 문제를 지적했다.

- 수식 3, 4줄의 시작 위치가 여전히 틀림
- 수식 줄 맞춤을 Codex가 자체 판정할 수 있어야 함
- 한컴 문단 속성 중 이번 이슈에서 지켜야 할 핵심은 들여쓰기/내어쓰기 원점임

이에 따라 Stage 4의 "행 폭 기준 가운데 정렬" 가설은 폐기했다.

## 2. 수정 방향

채택 기준:

```text
수식-only TAC 자동 줄넘김으로 만들어진 virtual row도
같은 문단의 후속 시각 줄로 보고 문단 들여쓰기/내어쓰기 원점을 적용한다.
```

구체적으로 다음을 정리했다.

- compact endnote 수식-only 줄의 x 예외 처리 제거
- non-cell 수식-only row의 행별 가운데 정렬 제거
- `row_base_x(row)`는 `paragraph_effective_margin_left(margin_left, indent, line_idx + row)`만 사용
- 셀 내부 수식-only 줄은 기존 cell alignment 동작 유지
- 강제 줄넘김/혼합 TAC 경로는 `runs.is_empty()` 가드로 분리 유지

## 3. 산출물

시각 판정 후보:

- 디버그 SVG: `output/poc/task1310/stage6_paragraph_indent_only/3-09월_교육_통합_2022_010.svg`
- 일반 SVG: `output/poc/task1310/stage6_paragraph_indent_only_plain/3-09월_교육_통합_2022_010.svg`
- 한컴/PDF 비교 crop: `output/poc/task1310/visual_compare/hancom_stage6_plain_q12_formula_side_by_side_3x.png`

render-tree 좌표:

| 항목 | x | y | width | right |
|---|---:|---:|---:|---:|
| 첫 수식 row TAC 1 | 442.9 | 592.5 | 78.2 | 521.1 |
| 첫 수식 row TAC 2 | 521.1 | 593.1 | 102.2 | 623.3 |
| 자동 wrap row | 442.9 | 630.3 | 182.8 | 625.7 |
| 세 번째 물리 row TAC 1 | 442.9 | 668.2 | 112.2 | 555.1 |
| 세 번째 물리 row TAC 2 | 555.1 | 668.2 | 117.9 | 673.0 |
| 네 번째 물리 row TAC 1 | 442.9 | 703.6 | 102.2 | 545.1 |
| 네 번째 물리 row TAC 2 | 545.1 | 703.6 | 41.9 | 587.0 |
| 네 번째 물리 row TAC 3 | 587.0 | 711.6 | 17.5 | 604.5 |

3, 4번째 수식 줄이 더 이상 각 행 폭에 따라 좌우로 흔들리지 않고 같은 문단 내어쓰기 원점에서 시작한다.

## 4. 검증

통과:

```bash
cargo check
cargo fmt --all -- --check
cargo test --test issue_1308_forced_break_hanging_indent -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate issue_1256_2022_sep_page10_question12_keeps_between_notes_gap -- --nocapture
```

## 5. 남은 판단

작업지시자 시각 판정이 필요하다.

판정 포인트:

- 문12 수식 블록의 3, 4번째 줄이 한컴처럼 같은 문단 내어쓰기 기준으로 시작하는가?
- 자동 wrap row와 후속 물리 row가 과도하게 왼쪽/오른쪽으로 흔들리지 않는가?
- 문13 시작 위치가 한컴 흐름과 비교해 과도하게 벌어지거나 겹치지 않는가?
