---
kind: pr-review
status: active
pr: 5683
issue: 5682
---

# PR #5683 검토 - 파생 suite 준비의 Cargo 파일 격리

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#5683](https://github.com/edwardkim/rhwp/pull/5683) |
| 작성자 | `jangster77` (collaborator self-review) |
| base / head | `devel` / `codex/5682-derived-suite-cargo-isolation` |
| 후보 head | `97c3534ea7464edb6f17b69dd6dc65d96b5bac3a` |
| 관련 이슈 | [#5682](https://github.com/edwardkim/rhwp/issues/5682) |
| 변경 범위 | manifest 준비, Rust test wrapper, suite policy, 계약 테스트, 개발·검토 문서 |

## 변경 판정

- 일반 `rust-test-suite-manifest --prepare`는 ignored generated harness와 manifest만 만든다.
- 루트 Cargo test-target registry 갱신은 marker block만 허용하는 명시적
  `--sync-cargo-targets` 경로로 분리한다.
- test wrapper는 `--locked`로 `Cargo.lock` 갱신을 막는다.
- review/CI 산출물과 기여자 source PR의 제출 경계를 문서화했다.

## 완료한 로컬 검증

- `node --test scripts/tests/rust-test-suite-manifest.test.mjs scripts/tests/rust-test-suite-cargo-isolation.test.mjs scripts/tests/run-rust-test-locking.test.mjs`: 22 passed
- `node scripts/rust-test-suite-manifest.mjs --prepare` 및 `--check`: 통과
- prepare 전후 SHA-256 비교: `Cargo.toml`, `Cargo.lock` 모두 불변
- `cargo fmt --all -- --check`, `git diff --check`: 통과

## 위험과 범위 밖

- 명시적 `--sync-cargo-targets`는 maintainer registry 동기화 전용이며 일반 PR·review 준비에는 사용하지 않는다.
- renderer, fixture, CI workflow 변경이 없으므로 시각 검증과 workflow 변경 등급 검증은 적용하지 않는다.

## 결론

**CI 검증 대기.** 최신 PR head의 required check 통과, mergeability 재확인, 작업지시자 승인 뒤 merge 후보로 판단한다.
