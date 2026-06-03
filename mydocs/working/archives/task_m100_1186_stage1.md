# Task M100 #1186 Stage1 — clippy all-targets baseline 정리

- 일시: 2026-06-03
- 브랜치: `local/task_m100_1186`
- 기준 브랜치: `devel`

## 1. 재현

최신 `devel` 기준 전용 브랜치 생성 후 다음 명령으로 실패를 재현했다.

```bash
cargo clippy --all-targets -- -D warnings
```

대표 실패 항목:

- `src/wasm_api/tests.rs`: `get_first`, `int_plus_one`, `expect_fun_call`, `print_literal`, `manual_range_patterns`, `manual_is_multiple_of`
- `src/document_core/helpers.rs`: `box_default`
- `src/document_core/commands/text_editing.rs`: `items_after_test_module`
- `src/serializer/hwpx/canonical_defaults.rs`: `assertions_on_constants`
- `src/renderer/layout/table_layout.rs`: `useless_vec`
- 여러 테스트 문서 주석: `doc_lazy_continuation`, `doc_overindented_list_items`

## 2. 자동 정리

반복적이고 기계적인 항목은 `cargo clippy --all-targets --fix --allow-dirty --allow-staged -- -D warnings` 로 정리했다.

자동 정리된 주요 항목:

- `get(0)` → `first()`
- `Box::new(Default::default())` → `Box::default()`
- `vec![]` → `&[]`
- `len()` 비교 → `is_empty()`
- format literal 정리
- `items_after_test_module` 해소를 위한 item 위치 이동

## 3. 수동 보정

자동 정리 후 남은 항목은 수동으로 보정했다.

- `src/model/shape.rs`: 항상 참인 assertion 을 `matches!` 검사로 변경
- `src/renderer/layout/integration_tests.rs`: `filter(...).next_back()` 을 `rfind(...)` 로 변경
- `src/serializer/hwpx/canonical_defaults.rs`: 상수 기본값 검증을 `const` assert 로 이동
- 테스트 파일 doc comment list indentation 정리

## 4. 현재 검증

통과:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
```

추가 통과:

```bash
cargo test --tests
```

## 5. 판정

`cargo clippy --all-targets -- -D warnings` baseline 정리는 완료됐다.
변경은 clippy 지적 정리에 한정되며, 전체 테스트 회귀도 통과했다.
