# task_m100_4947 stage 11: 기계 판독 계약 crate 경계

## 목표

`src/**`의 `#[cfg(test)]`를 기계적으로 외부 테스트로 옮기지 않고도 루트
`rhwp` lib 테스트 바이너리의 컴파일 단위를 장기적으로 분할한다. 두 번째
경계로 스키마·출처·온톨로지 계층을 내부 workspace crate로 분리한다.

## 경계 선정 근거

- `schema_registry`는 루트 생산 모듈에 의존하지 않는다.
- `ir_schema`와 `provenance`는 `schema_registry`에만 의존한다.
- `ontology`는 위 세 모듈만 참조한다.
- 네 모듈의 외부 의존성은 `serde_json`으로 닫혀 있다.
- 루트는 `pub use rhwp_contracts::{...}` 방식으로 기존 공개 경로를 유지할 수 있다.

## 변경

- `schema_registry`, `ir_schema`, `provenance`, `ontology`를
  `crates/rhwp-contracts`로 이동했다.
- 기존 `rhwp::schema_registry`, `rhwp::ir_schema`, `rhwp::provenance`,
  `rhwp::ontology` 경로는 재노출하여 호환성을 보존했다.
- 기존 white-box 테스트 15개는 생산 코드와 함께 새 crate로 이동했다.
- Stage 10에서 도입한 workspace 내부 crate 테스트 게이트와 단위 테스트 계층
  인벤토리가 새 crate를 자동으로 포함한다.

## 검증 기준

- `cargo fmt --all -- --check`
- `node --test scripts/tests/rust-unit-test-tiers.test.mjs`
- `node scripts/rust-unit-test-tiers.mjs --check`
- `cargo test -p rhwp-contracts --lib --target-dir target/pr-review`
- `cargo check --workspace --all-targets --all-features --target-dir target/pr-review`

## 장기 구조

공개 API 계약 테스트는 `tests/cases`로 분류하고, private 상태를 검사하는
white-box 테스트는 생산 코드와 함께 소유 내부 crate에 둔다. 이후 경계도
파일별 테스트 개수보다 생산 의존 방향이 닫혀 있는지를 우선해 선택한다.
