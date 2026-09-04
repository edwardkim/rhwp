# task_m100_4947 stage 16: 느린 아카이브 대상 독립 계약 복원

## 배경

PR #4976의 CI에서 세 개 `Build test archive` 작업이 모두 컴파일 전에 실패했다.
아카이브 계획기는 `overflow_cell_baseline`을 별도의 Cargo 통합 테스트 대상으로 찾아 느린 전용
아카이브에 배치하지만, 자동 샤딩 과정에서 이 파일이 `regression_suite_027`의 모듈로 흡수되어
독립 대상이 사라졌다.

## 계약

- `overflow_cell_baseline`은 느린 테스트 전용 아카이브의 단일 Cargo 통합 테스트 대상으로 유지한다.
- 자동 생성 manifest에서는 `manual_isolation` 예외로 관리한다.
- nextest 우선순위 100은 그대로 유지한다.
- 일반 회귀 테스트는 기존 자동 샤딩 규칙을 계속 따른다.

## 변경

- `tests/overflow_cell_baseline.rs`를 `regression_suite_027`에서 제외하고 수동 격리 예외로 등록한다.
- Cargo 테스트 대상과 생성 harness를 manifest에서 다시 생성한다.
- 느린 대상이 독립 target 및 nextest 우선순위를 함께 유지하는 회귀 계약을 추가한다.

## 검증

- `node scripts/rust-test-suite-manifest.mjs --check`
- `node scripts/rust-test-suite-manifest.mjs --check --base-ref upstream/devel`
- `node --test scripts/tests/rust-test-suite-manifest.test.mjs`
- GitHub Actions 아카이브 계획기를 로컬 Cargo metadata에 적용
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --test overflow_cell_baseline`
- 전체 nextest 회귀 테스트
- `cargo fmt --all -- --check`
- `git diff --check`

검증 결과:

- manifest 현재 상태 및 `upstream/devel` 기준 검사 통과
- manifest 단위 계약 9건 통과
- CI 아카이브 계획기에서 `slow.args`가 `--test overflow_cell_baseline` 하나만 포함함을 확인
- 독립 느린 테스트 1건 통과(13.67초)
- 전체 nextest: 6,518건 통과, 38건 제외, 실패 0건(3분 7.54초)
- rustfmt 및 diff whitespace 검사 통과
