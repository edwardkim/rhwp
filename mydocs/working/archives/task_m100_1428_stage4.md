# Task 1428 Stage 4

## 목표

- 첫 번째 누름틀 값 뒤에 두 번째 빈 누름틀 guide가 붙어 있을 때, 두 번째 guide를 마우스로 클릭하면 두 번째 누름틀 내부로 진입하도록 한다.

## 문제 인식

- Stage 3의 TypeScript 보정은 hit-test가 반환한 문단/offset 기준으로 빈 guide를 찾는다.
- Rust `hit_test_native`의 ClickHere guide hit 분기는 guide를 맞힌 뒤 해당 문단의 첫 ClickHere field range를 반환하고 있다.
- 같은 문단에 여러 ClickHere가 있으면 두 번째 guide 클릭도 첫 번째 누름틀 위치로 반환되어, Studio 보정 이전에 잘못된 필드 위치가 들어온다.

## 구현 방침

- guide hit 분기에서 클릭한 guide 위치와 field range start를 매칭해 해당 ClickHere를 반환한다.
- 매칭 실패 시 기존 첫 필드 반환 대신 일반 hit-test 경로로 폴백하거나 안전하게 기존 동작을 유지한다.
- Stage 3의 Studio 경계 보정은 유지한다.

## 검증 계획

- 첫 번째 값 누름틀 `123` 뒤에 빈 guide `입력하세요`가 붙은 상황에서 두 번째 guide 클릭 진입을 확인한다.
- `npx tsc --noEmit`, `npm test`, 관련 Rust 테스트 또는 `cargo test --test issue_258_clickhere_form_mode`를 수행한다.
- `git diff --check`를 수행한다.

## 구현 결과

- `hit_test_native`의 guide hit 분기에서 guide와 같은 위치의 폭 0 anchor run을 찾아 실제 guide field start offset을 산출하도록 했다.
- `find_field_hit_for_guide`는 해당 offset의 빈 ClickHere를 우선 반환하고, anchor 매칭이 실패할 때도 첫 빈 ClickHere를 우선 사용하도록 정리했다.
- `adjacent_clickhere_input_prefers_new_empty_field_at_shared_boundary`에 실제 마우스 hit-test 경로 회귀 테스트를 추가했다.

## 검증 결과

- `cargo fmt --check`
- `cargo test --test issue_258_clickhere_form_mode`
- `npx tsc --noEmit`
- `npm test`
- `wasm-pack build --target web --out-dir pkg`
- `git diff --check`
