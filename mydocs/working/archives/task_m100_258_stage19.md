# Task M100-258 Stage 19 — 누름틀 이전 입력 경계 보정

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-16
- 선행 커밋: `80dae055` (`task 258: 누름틀 값 복사와 경계 이동 보정`)

## 1. 문제

Stage18에서 누름틀 시작 이전 위치를 만들 수 있게 했지만, 그 위치에서 `ab` 같은 텍스트를
입력하면 일반 본문이 아니라 누름틀 값 앞쪽으로 들어간다.

## 2. 기대 동작

- 한컴처럼 누름틀 이전 위치에서 입력하면 일반 본문 텍스트가 된다.
- 누름틀 내부 시작 위치에서 입력하면 누름틀 값으로 들어간다.
- 빈 누름틀 첫 입력은 기존처럼 누름틀 값으로 들어간다.

## 3. 원인 후보

`Paragraph::insert_text_at()`은 `char_offset == FieldRange.start_char_idx`에서 삽입할 때
`end_char_idx`만 늘리고 `start_char_idx`는 유지한다. 따라서 프런트엔드가 같은 offset을
“누름틀 이전”으로 표시해도 코어는 필드 내부 시작 삽입으로 처리한다.

## 4. 수정 계획

- 활성 field가 아닌 ClickHere의 시작 offset에 삽입하는 경우를 기록한다.
- 삽입 후 해당 `FieldRange.start_char_idx`를 삽입 길이만큼 앞으로 밀어, 새 텍스트가
  필드 밖 일반 본문으로 남게 한다.
- 빈 누름틀은 예외로 유지한다.

## 5. 수정

- `inactive_field_start_insertions()`와 `keep_inactive_field_start_outside()`를 추가했다.
- 본문, 표 셀, 중첩 셀 path 입력 경로에 시작 경계 보정을 적용했다.
- 활성 ClickHere 시작 위치 입력은 기존처럼 필드 값으로 유지하고, 비활성 시작 경계 입력만
  일반 본문으로 남도록 했다.

## 6. 검증

- `cargo fmt`
- `git diff --check`
- `cargo test --test issue_258_clickhere_form_mode clickhere_start_boundary_insert_respects_active_field_state -- --nocapture`
- `cargo test --test issue_258_clickhere_form_mode`
- `wasm-pack build --target web --out-dir pkg`
