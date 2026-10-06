# Stage 13 - 파생 suite 산출물 untrack 전환

## 목표

`tests/suites/suite-policy.json`만 추적 입력으로 유지하고, 자동 배정 결과인
`tests/suites/manifest.json`과 `tests/generated/**`는 PR review·CI의 `--prepare`가 만드는 ignored
산출물로 전환한다.

## 계약

- source PR은 `tests/cases/**` 원본과 필요할 때 `suite-policy.json`의 정책만 변경한다.
- `manifest.json`과 harness는 Git index에 존재하지 않으며, `--prepare` 뒤에만 작업 트리에 나타난다.
- `run-rust-test.mjs`와 `--check`는 같은 policy에서 in-memory 배정을 재구성한다.
- Cargo generated target block은 `--prepare`가 검증 시에만 갱신하며, #5177 base-diff gate가 PR 커밋을 거부한다.
- 이번 전환의 삭제는 허용하되, base 이후 새로 생긴 파생 파일과 `tests/cases/**` 밖의 신규 integration source는 CI가 계속 거부한다.
# 검증 결과

- `node --test scripts/tests/rust-test-suite-manifest.test.mjs`: 15 passed, 0 failed.
- `node scripts/rust-test-suite-manifest.mjs --prepare` 및 `node scripts/rust-test-suite-manifest.mjs --check --base-ref upstream/devel`: 통과.
- `CARGO_INCREMENTAL=0 cargo nextest run --cargo-profile release-test --target-dir target/pr-review-kevin9327-unincluded-5175-20260817 --tests --test-threads 8 --no-fail-fast`: 6,643 passed, 38 skipped.
