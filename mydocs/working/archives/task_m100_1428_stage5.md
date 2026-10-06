# Task 1428 Stage 5

## 목표

- 누름틀을 복사한 뒤 바로 붙여넣기하면 캐럿/입력 상태가 마지막으로 붙은 누름틀 내부에 남는 문제를 수정한다.
- 한컴오피스처럼 붙여넣기 직후 캐럿은 붙여넣은 누름틀 바깥, 즉 붙여넣은 범위 뒤에 위치해야 한다.

## 문제 인식

- 현재 rhwp-studio에서는 누름틀 복사/붙여넣기 후 마지막 누름틀의 끝 경계가 활성 필드 상태로 남아, 이어지는 입력이 마지막 누름틀 값에 들어가는 것으로 보인다.
- 복사/붙여넣기 로직의 최종 캐럿 위치와 `active_field` 정리 시점이 분리되어 있을 가능성이 있다.

## 조사 범위

- Studio paste 명령 처리 경로
- Rust `paste_internal_native` 및 ClickHere field range 복제 후 반환 위치
- 붙여넣기 후 `active_field` 유지/해제 경로
- 기존 `tests/issue_258_clickhere_form_mode.rs`의 누름틀 복사/붙여넣기 회귀 테스트

## 구현

- `paste_internal_native` 계열 결과 JSON에 `containsField`를 추가해 내부 클립보드가 누름틀 field range를 포함하는지 Studio에 전달한다.
- Studio `pasteInternal` 처리에서 `containsField=true`이면 snapshot 실행 뒤 `markCurrentFieldEndOutside()`를 호출하도록 일회성 플래그를 둔다.
- 누름틀 복사/붙여넣기 후 이어서 입력한 글자가 마지막 누름틀 범위에 흡수되지 않는 회귀 테스트를 추가한다.

## 검증 계획

- 누름틀 `123`을 복사해 같은 위치 뒤에 붙여넣은 뒤 `123123`이 두 누름틀로 보이되, 캐럿은 마지막 누름틀 바깥에 위치하는지 검증한다.
- 붙여넣기 직후 추가 입력이 마지막 누름틀 내부가 아니라 일반 본문으로 들어가는지 회귀 테스트를 추가한다.
- `cargo test --test issue_258_clickhere_form_mode`, `npx tsc --noEmit`, `npm test`, `git diff --check`를 수행한다.

## 검증 결과

- `cargo fmt --check`: 통과
- `cargo test --test issue_258_clickhere_form_mode`: 통과
- `npx tsc --noEmit`: 통과
- `npm test`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- `git diff --check`: 통과
