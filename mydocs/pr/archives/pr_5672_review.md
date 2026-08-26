---
kind: pr-review
status: merged
pr: 5672
issue: 5671
merged_at: 2026-08-19T17:54:04Z
---

# PR #5672 검토 기록 - rhwp-q-pack 조회 CLI 50개와 Control 필드 조회 코드

## 결론

**머지 완료.** 원 기여분과 메인터너 보정은 `f9616a95fdffb917e9a5a74d4cdf4f4ad774b32e`로
`devel`에 squash merge되었다.

## 검토 범위

- 원 PR: [#5672](https://github.com/edwardkim/rhwp/pull/5672)
- 관련 이슈: [#5671](https://github.com/edwardkim/rhwp/issues/5671)
- 기여자: `kevin9327`
- 원 source head: `d81d69b`
- 최종 검토 head: `47b2fab503c3ce6d74528c09dbfefd0710c00afe`
- 메인터너 보정: `47b2fab`의 `caption-tables`, `ctrl-kinds`, `page-starts-on` 계약 보정

## 검토 결과

- `caption-tables`는 실제 caption이 있는 표만 반환하고, 행·열·cell 수와 caption 문단 수를 공개한다.
- `ctrl-kinds`는 문서 안의 Control variant를 빠짐없이 안정된 kind/count 집계로 반환한다.
- `page-starts-on`은 저장된 section 시작 면의 `BOTH`, `EVEN`, `ODD` 값을 보존한다.
- 50개 명령의 help/JSON/volume-probe 계약은 `tests/cases/agent_q_pack_contract.rs`로 회귀 범위를 고정했다.
- 신규 source-side `#[cfg(test)]` 또는 source PR에 파생 suite/manifest를 추가하지 않았다.

## 검증 근거

- GitHub Actions 최신 head CI: Build & Test, Lint, Rust CodeQL, adapter inter-diff, proptest 성공.
  archive build는 12분 37초, Lint는 17분 31초, Rust CodeQL은 20분 57초였다.
- Focused `agent_q_pack_contract`: 4건 통과.
- 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --no-fail-fast`:
  7,995 passed, 38 skipped.
- `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings` 통과.
- suite manifest prepare/check, unit-test tier 정책 검사, `cargo fmt --all -- --check`, `git diff --check` 통과.
- CLI 조회 변경만 포함하므로 renderer/layout 시각 검증은 적용 대상이 아니다.

## 후속 처리

- #5671은 PR 본문의 close 연결로 자동 종료됐고, 해결 범위와 검증 결과를 이슈에 안내했다.
- 기여자 PR에는 merge SHA, 보정 내역, CI 및 로컬 검증 결과를 코멘트로 안내했다.
- 원 기여자 fork branch는 외부 소유이므로 삭제하지 않는다.

