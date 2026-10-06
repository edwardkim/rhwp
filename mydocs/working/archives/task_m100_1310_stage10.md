# Task #1310 Stage 10 - 연속 수식 경계 중복 caret stop 보정

## 1. 발견 현상

Stage 9 이후 9쪽 미주 수식 문단에서 커서 좌표가 줄 시작으로 되감기는 문제는 해소되었다.
하지만 수식 뒤 위치에서 오른쪽 방향키를 누르면 같은 화면 x 위치에 한 번 더 머무는 경우가 남았다.

재현 위치:

- `samples/3-09월_교육_통합_2022.hwp`
- 9쪽 미주 영역
- `pi=479` 수식-only 문단

증상:

- offset 1과 2가 같은 x
- offset 3과 4가 같은 x
- offset 5와 6이 같은 x
- 사용자는 수식 다음으로 이동하려면 오른쪽 방향키를 한 번 더 눌러야 함

## 2. 원인

수식만 연속된 문단에서는 이전 수식의 끝 x와 다음 수식의 시작 x가 동일하다.
`navigateNextEditable` 은 다음 offset이 인라인 컨트롤 시작이면 해당 offset을 그대로 반환한다.

이 때문에 논리적으로는 전진했지만 화면상 caret 위치는 같은 x에 머무는 중복 stop이 노출되었다.

단, 같은 현상이 모든 TAC 문단에 적용되면 안 된다.
`eq-002`처럼 문단 텍스트에 강제 줄바꿈(`\n`)이나 탭이 있고 다음 visual line의 첫 수식 앞으로
진입해야 하는 경우에는 해당 offset을 그대로 보존해야 한다.

## 3. 수정

`doc_tree_nav` forward 이동에서 아래 조건을 모두 만족할 때만 중복 경계를 건너뛴다.

- 문단에 Equation 컨트롤이 있음
- 문단의 편집 가능한 인라인 컨트롤이 모두 Equation임
- 문단 텍스트가 비어 있거나 object replacement placeholder만 있음
- 현재 offset이 이전 Equation 직후임
- 다음 offset이 다음 Equation 시작임

이 경우 `next_offset` 대신 `next_offset + 1`을 반환해 같은 화면 x를 한 번 더 밟지 않는다.

혼합 텍스트/수식 문단과 강제 줄바꿈 수식 문단은 기존 이동 규칙을 유지한다.

수정 파일:

- `src/document_core/queries/doc_tree_nav.rs`
- `tests/issue_1139_inline_picture_duplicate.rs`

## 4. 회귀 테스트

추가 테스트:

```text
issue_1139_endnote_equation_right_arrow_skips_duplicate_boundary_stop
```

테스트 내용:

- `pi=471` 텍스트 혼합 수식 문단: offset 1 -> 2 유지
- `pi=479` 수식-only 문단: offset 1 -> 3, 3 -> 5, 5 -> 7

기존 #1308 테스트도 함께 유지한다.

- `eq-002` 강제 줄바꿈 다음 줄 첫 수식 진입: offset 3 -> 4 유지

## 5. 검증

통과:

```bash
cargo fmt --all -- --check
cargo test --test issue_1139_inline_picture_duplicate issue_1139_endnote_equation_right_arrow_skips_duplicate_boundary_stop -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate issue_1139_endnote_equation_cursor_rects_do_not_rewind_to_line_start -- --nocapture
cargo test --test issue_1308_forced_break_hanging_indent -- --nocapture
cargo check
cargo check --target wasm32-unknown-unknown --lib
docker compose --env-file .env.docker run --rm wasm
```
