# task_m100_4947 stage 12: OOXML 차트 crate 경계

## 목표

루트 `rhwp` lib 테스트 바이너리에서 큰 white-box 테스트 묶음을 실제로
분리한다. 생산 코드 의존 방향이 닫힌 `ooxml_chart` 계층을 내부 workspace
crate로 옮겨 로컬과 CI가 독립 컴파일·실행할 수 있게 한다.

## 경계 선정 근거

- `ooxml_chart`에는 source-side 테스트 165개가 있다.
- `data`, `parser`, `patch`, `renderer`와 공개 모델을 담은 모듈 루트로 닫혀 있다.
- 다른 루트 생산 모듈에 의존하지 않는다.
- 외부 crate 의존성은 `quick-xml` 하나뿐이다.
- 루트가 crate를 `ooxml_chart`라는 이름으로 재노출하면 기존 공개 경로와
  내부 호출부를 그대로 유지할 수 있다.

## 변경

- `src/ooxml_chart`를 `crates/rhwp-ooxml-chart`로 이동했다.
- 루트에서 `pub use rhwp_ooxml_chart as ooxml_chart`로 재노출했다.
- 기존 `rhwp::ooxml_chart::*` 공개 경로를 보존했다.
- 생산 코드와 private 상태를 함께 검사하는 165개 테스트를 새 crate의 lib
  테스트 바이너리로 분리했다.
- 단위 테스트 계층 인벤토리는 경로 이동만 반영하고 전체 테스트 수와 tier
  분류를 유지한다.

## 검증 기준

- `cargo fmt --all -- --check`
- `node --test scripts/tests/rust-unit-test-tiers.test.mjs`
- `node scripts/rust-unit-test-tiers.mjs --check`
- `cargo test -p rhwp-ooxml-chart --lib --target-dir target/pr-review`
- `cargo check --workspace --all-targets --all-features --target-dir target/pr-review`

## 장기 구조

루트 소스의 `#[cfg(test)]`를 금지하거나 private 검사를 외부로 강제하지 않는다.
대신 생산 의존성이 닫힌 기능 계층을 코드와 테스트가 함께 소유하는 내부
crate로 분리한다. 이 방식은 새 테스트의 자동 분류 규칙을 유지하면서 Cargo의
crate별 lib 테스트 바이너리 병렬성을 활용한다.
