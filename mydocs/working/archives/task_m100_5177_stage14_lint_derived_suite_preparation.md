# Stage 14 - Lint 이전 파생 suite 준비

## 배경

Stage 13에서 `tests/generated/**`와 `tests/suites/manifest.json`을 추적 대상에서
제거했다. 로컬 전체 회귀는 `--prepare` 뒤에 실행되어 통과했지만, CI lint job은
`cargo fmt --all -- --check`를 먼저 실행했다. Cargo가 `Cargo.toml`의 integration
target 경로를 해석하는 시점에 파생 harness가 없어 CI가 실패했다.

## 보정

- lint job의 첫 Cargo 명령 전에 `rust-test-suite-manifest.mjs --prepare`를 실행한다.
- 기존 suite 정책 검증 단계에서는 중복 생성하지 않고 검증만 수행한다.
- workflow 계약 테스트가 준비 단계의 존재·순서·단일 실행을 검사한다.
- CI가 함께 보고한 `src/parser/body_text/tests.rs`의 rustfmt 불일치를 정리한다.

## 검증 계획

- suite manifest Node 테스트
- CI workflow Python 계약 테스트
- `--prepare` 후 `cargo fmt --all -- --check`

## 검증 결과

- `node --test scripts/tests/rust-test-suite-manifest.test.mjs`: 15 passed.
- `python3 -m unittest scripts/tests/test_ci_impact_workflow.py`: 28 passed.
- `node scripts/rust-test-suite-manifest.mjs --prepare && cargo fmt --all -- --check`: 통과.
