# task_m100_4947 stage 10: 내부 production crate 테스트 경계

## 목표

소스의 `#[cfg(test)]`를 억지로 integration test로 바꾸지 않고 production 모듈과 white-box 테스트를
함께 workspace crate로 분리해 단일 root lib test binary의 장기적 컴파일 병목을 줄인다.

## 설계

- 공개 API 회귀는 계속 `tests/cases/`로 이동한다.
- private 구현 불변식은 production 코드와 같은 crate의 `#[cfg(test)]`로 유지한다.
- 외부 의존성만 사용하는 leaf 모듈부터 `crates/*`로 추출한다.
- root는 기존 공개 module 경로를 re-export해 소비자 호환성을 유지한다.
- workspace `default-members`가 root와 내부 crate를 함께 선택하므로 로컬 cargo/nextest에서 누락되지 않는다.
- CI root nextest archive는 기존 integration target을 유지하고 내부 crate lib test는 required lint job에서
  별도 실행한다.

## 첫 경계

`password_crypto`는 parser·serializer가 공개 함수만 소비하고 다른 rhwp 내부 모듈에 의존하지 않는다.
따라서 구현과 private 테스트를 `rhwp-password-crypto` crate로 함께 이동하고
`rhwp::password_crypto` 경로를 re-export한다.

## 자동화

- `rust-unit-test-tiers.mjs`가 root `src/`뿐 아니라 `crates/*/src/`도 전수 수집한다.
- 내부 crate를 추가해도 unit tier 기준선과 로컬 default-member 실행에서 빠지지 않는다.
- CI workflow 계약 테스트가 내부 crate 실행 gate의 제거를 막는다.

## 검증

```bash
node --test scripts/tests/rust-unit-test-tiers.test.mjs
node scripts/rust-unit-test-tiers.mjs --check
python3 -m unittest scripts.tests.test_nextest_archive_workflow
cargo check --workspace --all-targets
cargo nextest list --cargo-profile release-test --target-dir target/pr-review --tests
```

## 결과

- workspace default-members 목록에서 root `rhwp` 6,549개와 `rhwp-password-crypto` 2개가 함께
  발견되어 총 6,551개 기준선을 유지했다.
- 내부 crate 테스트 2개는 CI와 같은 workspace exclude 명령으로 통과했다.
- 전체 nextest는 45개 binary에서 runnable 6,513개 전부 통과했고 38개가 skip됐다.
- release-test 구조 변경 후 최초 목록 컴파일은 4분 23초, warm 전체 실행은 170.146초였다.
- leaf crate 하나만 분리한 이번 단계에서는 즉시 wall time이 줄지 않았다. 성능 효과를 주장하지 않고,
  이후 테스트 집중도가 높은 leaf 경계를 같은 방식으로 분리할 수 있게 된 것을 성과로 한정한다.
