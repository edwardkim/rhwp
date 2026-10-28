# Task #1310 Stage 6 - 수식-only TAC 한정 60.5pt 내어쓰기 적용

## 1. 작업지시자 피드백

Stage 5/7 후보는 수식-only TAC 문단의 첫 시각 줄/후속 시각 줄 구분은 맞췄지만,
후속 줄 내어쓰기 폭을 resolved ParaShape `indent` 값 그대로 적용했다.

작업지시자는 한컴 UI에서 확인된 내어쓰기 `60.5 pt` 전체가 적용되어야 한컴에디터와
유사한 피델리티가 나온다고 판단했다.

## 2. 구현 범위

전역 ParaShape 해석은 변경하지 않는다.

적용 범위는 다음 조건을 만족하는 수식-only TAC 흐름으로 한정한다.

- `composed.lines` 의 모든 line `runs` 가 비어 있음
- TAC 컨트롤이 존재함
- 모든 TAC 컨트롤이 `Control::Equation`
- `compute_equation_only_tac_line_flow()` 가 `Some` 을 반환하는 경로

따라서 일반 텍스트 문단, 텍스트+TAC+고정탭 혼합 문단, 셀 내부 정렬 경로는 기존 규칙을
유지한다.

## 3. 구현 규칙

수식-only TAC 문단의 시각 줄 기준:

- 첫 번째 시각 수식 줄: 문단 첫 줄 원점 `margin_left`
- 자동 줄넘김 row 및 후속 수식 줄: `margin_left + abs(indent * 2.0)`

현재 `style_resolver` 는 ParaShape margin/indent 를 2배 스케일 저장값으로 보고
`hwpunit_to_px(raw) / 2` 로 resolved 값에 담는다. 이번 이슈의 수식-only TAC 경로에서는
한컴 UI 지정값 `60.5 pt` 전체를 후속 줄 원점으로 써야 하므로, 해당 경로에서만
`indent_scale = 2.0` 을 적용한다.

## 4. 산출물

디버그 SVG:

- `output/poc/task1310/stage8_equation_only_full_indent/3-09월_교육_통합_2022_010.svg`

일반 SVG:

- `output/poc/task1310/stage8_equation_only_full_indent_plain/3-09월_교육_통합_2022_010.svg`

render-tree:

- `output/poc/task1310/stage8_equation_only_full_indent_tree/render_tree_010.json`

비교 이미지:

- `output/poc/task1310/visual_compare/hancom_stage8_plain_q12_formula_side_by_side_3x.png`

## 5. render-tree 좌표

| 항목 | x | y | width | right |
|---|---:|---:|---:|---:|
| 첫 수식 row TAC 1 | 402.5 | 592.5 | 78.2 | 480.7 |
| 첫 수식 row TAC 2 | 480.7 | 593.1 | 102.2 | 582.9 |
| 자동 wrap row | 483.2 | 630.3 | 182.8 | 666.0 |
| 세 번째 물리 row TAC 1 | 483.2 | 668.2 | 112.2 | 595.4 |
| 세 번째 물리 row TAC 2 | 595.5 | 668.2 | 117.9 | 713.4 |
| 네 번째 물리 row TAC 1 | 483.2 | 703.6 | 102.2 | 585.4 |
| 네 번째 물리 row TAC 2 | 585.5 | 703.6 | 41.9 | 627.4 |
| 네 번째 물리 row TAC 3 | 627.4 | 711.6 | 17.5 | 644.9 |

첫 줄 x=402.5, 후속 줄 x=483.2 로 차이는 약 80.7px이다.
이는 60.5pt 를 96DPI px 로 환산한 값과 일치한다.

## 6. 검증

통과:

```bash
cargo check
cargo check --target wasm32-unknown-unknown --lib
cargo fmt --all -- --check
cargo test --test issue_1308_forced_break_hanging_indent -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate issue_1256_2022_sep_page10_question12_keeps_between_notes_gap -- --nocapture
```

## 7. 판정 요청

작업지시자 시각 판정이 필요하다.

판정 포인트:

- 첫 수식 row가 문단 첫 줄 원점에서 시작하는가?
- 자동 wrap row와 후속 물리 row가 한컴 UI 내어쓰기 60.5pt 전체 기준으로 시작하는가?
- 문13 시작 위치가 과도하게 벌어지거나 겹치지 않는가?

## 8. 추가 보정 - 조판부호 위치

작업지시자 확인 중 수식-only TAC 흐름의 디버그 SVG에서 강제 줄넘김/문단 끝 조판부호가
수식 행 끝이 아니라 empty-run 앵커의 기존 x 위치에 남는 현상이 확인되었다.

조판 결과 자체가 아니라 `--show-control-codes` 표시 노드 문제이므로, empty-run 앵커는
유지하되 수식-only TAC 경로에서는 조판부호 마커를 별도 `TextRun` 으로 생성하여 마지막
visual row의 수식 끝 x에 붙이도록 보정했다.

추가 산출물:

- `output/poc/task1310/stage9_control_marker_fixed/3-09월_교육_통합_2022_010.svg`
- `output/poc/task1310/stage9_control_marker_fixed_tree/render_tree_010.json`

확인 좌표:

- 첫 수식 block 강제 줄넘김: `x=666.05`
- 다음 수식 block 강제 줄넘김: `x=713.40`
- 마지막 수식 block 문단 끝: `x=644.88`
