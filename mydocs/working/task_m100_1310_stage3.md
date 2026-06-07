# Task #1310 Stage 3 - TAC 자동 줄바꿈 후 문단 들여쓰기/내어쓰기 적용

## 1. 추가 피드백

Stage 2 SVG에서 연속 TAC 수식 3개의 자동 줄넘김은 확인되었다.
다만 자동 줄넘김으로 만들어진 virtual row가 문단의 들여쓰기/내어쓰기 기준 x를 적용하지 않았다.

작업지시자 판정:

- 자동 줄넘김 처리는 됨
- 문단 내어쓰기/들여쓰기 처리가 누락됨

## 2. 원인

Stage 2 구현은 `equation_tac_flow`에서 TAC 수식들을 virtual row로 나누었지만,
`paragraph_layout`의 empty-run TAC 렌더링 경로는 모든 row의 시작 x를 기존 physical line의
`effective_margin_left`로 고정했다.

따라서 새 row는 한컴 기준의 "후속 줄"처럼 취급되어야 하는데, 실제로는 첫 row와 같은 x에서
시작했다.

## 3. 수정

수정 파일:

- `src/renderer/equation_tac_flow.rs`
- `src/renderer/layout/paragraph_layout.rs`
- `src/renderer/typeset.rs`
- `src/renderer/height_measurer.rs`
- `tests/issue_1139_inline_picture_duplicate.rs`

주요 변경:

- `equation_tac_flow` helper가 첫 row 폭과 후속 row 폭을 구분하도록 확장했다.
- 문단 line index 기준의 들여쓰기/내어쓰기 계산 helper를 공통화했다.
- 렌더링 시 virtual row `n`의 시작 x를 `line_idx + n` 기준 effective margin으로 다시 계산한다.
- 높이 측정 경로도 줄별 effective margin을 반영한 폭을 사용한다.
- #1310 회귀 테스트는 wrapped TAC가 다음 row로 이동하는 것뿐 아니라, 후속 row의 문단 내어쓰기 x도 적용되는지 검증하도록 변경했다.

## 4. 검증 결과

통과:

```bash
cargo check
cargo fmt --all -- --check
cargo test --test issue_1308_forced_break_hanging_indent -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate issue_1256_2022_sep_page10_question12_keeps_between_notes_gap -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate -- --nocapture
```

전체 `issue_1139_inline_picture_duplicate` 결과:

```text
68 passed; 0 failed
```

## 5. 산출물

시각 판정 요청 대상:

- `output/poc/task1310/stage3_indent_fixed/3-09월_교육_통합_2022_010.svg`
- `output/poc/task1310/stage3_indent_fixed_tree/render_tree_010.json`

render-tree 좌표:

| 항목 | x | y | width | right |
|---|---:|---:|---:|---:|
| 첫 row TAC 1 | 402.5 | 592.5 | 78.2 | 480.7 |
| 첫 row TAC 2 | 480.7 | 593.1 | 102.2 | 582.9 |
| wrapped TAC 3 | 442.9 | 630.3 | 182.8 | 625.7 |

wrapped TAC 3은 첫 row x=402.5가 아니라 후속 줄 내어쓰기 x=442.9에서 시작한다.

## 6. 현재 판정

가설 A는 "연속 TAC 수식 폭 기반 자동 줄넘김 + virtual row의 문단 effective margin 재계산"까지
확장해야 성립한다.

최종 채택 전에는 위 SVG에 대한 메인테이너 시각 판정이 필요하다.
