# Task M100 #1179 Stage 1

## 목적

`cargo test renderer::height_cursor::tests::compact_endnote -- --nocapture` 실행 시 출력되는 Rust warning을 제거한다.

## 시작 기준

- 이슈: [#1179](https://github.com/edwardkim/rhwp/issues/1179)
- 기준 브랜치: 최신 `upstream/devel`
- 작업 브랜치: `local/task_m100_1179`

## 경고 목록

- `src/renderer/equation/parser.rs`: 중복 `#[test]` attribute
- `src/renderer/layout/integration_tests.rs`: 불필요한 괄호
- `src/serializer/hwpx/field.rs`: non-snake-case 테스트 함수명
- `src/wasm_api/tests.rs`: non-snake-case 테스트 함수명
- `src/wasm_api/tests.rs`: `insert_text_native` 반환 `Result` 미사용
- `src/wasm_api/tests.rs`: `convert_to_editable_native` 반환 `Result` 미사용

## 계획

1. 붙여넣은 로그 기준 경고 위치를 확인한다.
2. 테스트 의미를 바꾸지 않고 attribute, 함수명, unused result만 정리한다.
3. 대상 재현 명령과 포맷을 검증한다.

## 현재 상태

- 2026-05-30: 작업지시자가 warning 제거를 위한 새 이슈 등록과 `upstream/devel` 기준 해결을 지시했다.
- 2026-05-30: 이슈 #1179를 생성하고 최신 `upstream/devel`에서 `local/task_m100_1179` 브랜치를 만들었다.
- 2026-05-30: 재현 명령에서 warning 6개가 출력되는 것을 확인했다.
- 2026-05-30: 중복 `#[test]`, 불필요한 괄호, non-snake-case 테스트명 2개, unused `Result` 2개를 정리했다.
- 2026-05-30: `cargo fmt --all --check` 통과.
- 2026-05-30: `cargo test renderer::height_cursor::tests::compact_endnote -- --nocapture` 재실행 결과 warning 없이 통과.
- 2026-05-31: 작업지시자 승인 후 커밋하고 PR #1180을 생성했다.

## 수정 파일

- `src/renderer/equation/parser.rs`
- `src/renderer/layout/integration_tests.rs`
- `src/serializer/hwpx/field.rs`
- `src/wasm_api/tests.rs`

## PR

- 커밋: `task 1179: Rust 테스트 경고 정리`
- PR: [#1180](https://github.com/edwardkim/rhwp/pull/1180)
