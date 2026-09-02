---
kind: pr-review
status: merged
pr: 5581
base: devel
head: review/kevin9327-open-20260819
merge_commit: d9ffc47f4f214f0cdadb1561ef5f00c6548a4ece
source_prs: [5545, 5546, 5550]
issues: [5518, 5508]
---

# PR #5581 kevin9327 verifier와 rhwp-agent CLI 통합 검토

## 라우팅

```text
base route: collaborator maintainer integration
modifiers: external PR cherry-pick, cumulative validation, maintainer correction,
post-merge source PR and issue closure, option B docs-only record
loaded documents: pr_review_workflow.md, pr_review/README.md,
collaborator_self_merge.md, intake_and_review.md, post_merge.md,
local_validation.md
```

## 적용 범위

| 원 PR | 적용 내용 | 원본 head |
| --- | --- | --- |
| [#5545](https://github.com/edwardkim/rhwp/pull/5545) | V-fresh toolVersion 검증기와 고정 코퍼스 | `37bd853590dddedfd9ca16d3aada41bce6bf755b` |
| [#5546](https://github.com/edwardkim/rhwp/pull/5546) | V-abstain 봉투 모순 기권 검증기와 코퍼스 | `b3a4987765995bc0de9436d6ca72c04caa5649e9` |
| [#5550](https://github.com/edwardkim/rhwp/pull/5550) | 읽기 전용 `rhwp-agent` 조회·비교·검색 명령 | `d53b4e355b1c1c59238f637ea8e1dd650e1bacce` |

- 원 PR commit은 `-x` 계보로 누적 적용했다.
- `tests/agent_cli_pack_contract.rs`는 `tests/cases/agent_cli_pack_contract.rs`로 옮겨 파생 harness에서 실제 실행되도록 보정했다.
- CI가 발견한 `capabilities --json`의 고정 명령 계약 누락은 `0c7dc94b2`에서 80개 공개 명령으로 보정했다. 구현 단일 출처 `caps::COMMANDS`와 별도로 외부 공개 표면의 추가·삭제·개명을 감지한다.

## 검증 기록

- `node scripts/rust-test-suite-manifest.mjs --prepare`와 `--check`: 통과
- `node scripts/rust-unit-test-tiers.mjs --check`: 통과
- `cargo fmt --all -- --check`: 통과
- `CARGO_TARGET_DIR=target/pr-review cargo clippy --all-targets -- -D warnings`: 통과
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`: 7,773 passed
- GitHub CI: Lint, Native Skia, archive builder, slow shard, regular shard 1~3, CodeQL 세 언어, Proptest, Adapter inter-diff, Frontend package gate, Build & Test aggregate 통과

## 결론 및 후속 처리

**병합 완료.** 통합 PR은 `d9ffc47f4f214f0cdadb1561ef5f00c6548a4ece`로 `devel`에 병합됐다.
원 PR #5545, #5546, #5550에는 통합 근거 댓글을 남기고 종료했고, 관련 이슈 #5518과 #5508도 종료했다.
이 문서는 소스 변경을 다시 실행시키지 않기 위해 옵션 B의 별도 docs-only PR로 보존한다.
