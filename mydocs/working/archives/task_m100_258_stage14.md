# Task M100-258 Stage 14 — 누름틀 확인 삭제 시 내용까지 제거

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-16
- 선행 커밋: `e26d47ed` (`task 258: 누름틀 삭제 컨트롤 제거`)

## 1. 문제

삭제 확인 대화상자의 `확인`을 누르면 내부 field/control은 제거되지만 본문 텍스트
`11223344`가 그대로 남아 사용자 입장에서는 누름틀이 실제로 삭제되지 않은 것처럼 보인다.

## 2. 수정 방향

- ClickHere 삭제 확인의 의미를 한컴오피스 동작에 맞춰 field/control 제거와 함께
  field range 내부 텍스트도 삭제하는 것으로 정리한다.
- 빈 ClickHere는 텍스트 삭제 없이 field/control만 제거한다.
- 삭제 후 다른 field range의 텍스트 위치와 control index가 정상 보정되는지 확인한다.

## 3. 검증 계획

- `cargo test --test issue_258_clickhere_form_mode`
- `cargo test --lib rebuild_`
- `wasm-pack build --target web --out-dir pkg`
- `http://localhost:7700/`에서 `누름틀-2024.hwp` 첫 누름틀 삭제 후 `11223344`도 사라지는지 확인

## 4. 수정 결과

- `remove_field_at()`이 ClickHere `FieldRange` 내부 텍스트를 먼저 삭제한 뒤
  `Control::Field`와 대응 `ctrl_data_records`를 제거하도록 수정했다.
- 첫 누름틀을 삭제해도 두 번째 ClickHere의 control index와 field value가 유지되는
  회귀 테스트를 `removing_clickhere_removes_field_text_and_control`로 갱신했다.
- rhwp-studio 입력 핸들러 주석도 실제 동작에 맞게 `누름틀 필드와 내용 제거`로 정리했다.

## 5. 검증 결과

- `cargo test --test issue_258_clickhere_form_mode removing_clickhere_removes_field_text_and_control -- --exact --nocapture`: 통과
- `cargo test --test issue_258_clickhere_form_mode`: 통과
- `cargo test --lib rebuild_`: 통과
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- `http://localhost:7700/` Playwright 검증 통과
  - 삭제 전 첫 field value/text: `11223344`
  - `Delete` 확인 대화상자에서 `확인` 클릭 후 첫 문단 text: 빈 문자열
  - field 목록에는 두 번째 `222212212` ClickHere만 유지
  - 렌더 SVG에 `11223344` 미존재
  - 스크린샷: `/tmp/task258-stage14-delete-confirm.png`
