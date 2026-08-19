---
kind: pr-review
status: merged
pr: 5545
issue: 5518
base: devel
source_head: 37bd853590dddedfd9ca16d3aada41bce6bf755b
integration_commit: 03f16cb371086a3a2084d21336beb2f414be3bfd
integration_pr: 5581
merge_commit: d9ffc47f4f214f0cdadb1561ef5f00c6548a4ece
---

# PR #5545 V-fresh toolVersion 검증기 검토

## 라우팅

```text
base route: collaborator maintainer integration
modifiers: external PR cherry-pick, cumulative validation, individual review record
loaded documents: pr_review_workflow.md, pr_review/README.md,
collaborator_self_merge.md, intake_and_review.md, post_merge.md
```

## 적용 범위

| 항목 | 값 |
| --- | --- |
| Source PR | [#5545](https://github.com/edwardkim/rhwp/pull/5545) |
| 작성자 | `kevin9327` |
| 관련 이슈 | [#5518](https://github.com/edwardkim/rhwp/issues/5518) |
| 원본 head | `37bd853590dddedfd9ca16d3aada41bce6bf755b` |
| 적용 commit | `03f16cb371086a3a2084d21336beb2f414be3bfd` |

- 영수증 `toolVersion`과 검증기 바이너리 버전이 trim 후에도 다르면 `reproduced: true`를 합격 근거로 쓰지 않는다.
- 독립 Rust crate와 고정 코퍼스가 stale-tool 사례, 문자열 동치 규칙, V-replay 축과의 분리를 검증한다.
- CI Lint의 verifier contract 단계에서 이 crate의 테스트를 실행하도록 연결한다.

## 검증 기록

- `CARGO_TARGET_DIR=target/review-kevin9327-tool-version cargo test --manifest-path tools/llm_verifier/tool_version_gate/Cargo.toml`: 33 passed
- `cargo fmt --manifest-path tools/llm_verifier/tool_version_gate/Cargo.toml --all -- --check`: 통과
- `cargo fmt --all -- --check`: 통과
- `node scripts/rust-test-suite-manifest.mjs --check`: 통과
- `node scripts/rust-unit-test-tiers.mjs --check`: 통과
- 누적 통합 검증 `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`: 7,773 passed
- 통합 PR #5581 CI: Lint, Native Skia, archive와 slow·regular shard, CodeQL 세 언어, Proptest, Adapter inter-diff, Build & Test 통과

## 결론

**병합 완료.** 원 저자 commit을 `-x` 계보로 적용한 뒤 통합 PR [#5581](https://github.com/edwardkim/rhwp/pull/5581)에
포함했고, `d9ffc47f4f214f0cdadb1561ef5f00c6548a4ece`로 `devel`에 병합됐다. 원 PR #5545와
관련 이슈 [#5518](https://github.com/edwardkim/rhwp/issues/5518)는 통합 근거 댓글을 남긴 뒤 종료했다.
