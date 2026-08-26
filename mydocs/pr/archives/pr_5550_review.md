---
kind: pr-review
status: merged
pr: 5550
base: devel
source_head: d53b4e355b1c1c59238f637ea8e1dd650e1bacce
integration_commits:
  - 60ae15737763d7dfd038a1bf57e91e3347b01b7b
  - 68f39978ff70300baa2d5bccbe96fed2b9f3a2c1
  - 202e9d7dfe10c5f4d012eee96ed812f96bfd6b17
  - 1e71faf5c4deca6b5c65314eeb4015999f9eb9f5
  - 7ec61beea563b06eb16419794d59e7f537d9f649
  - 5c6aff9e0aa72202a3e16476b3f0925b2b7d1a51
  - 523d76f317117b3d0e35770d39600d7b4abb91e0
maintainer_correction: 27e733d143c3d2939540bd53c43386ac237f41dc
integration_pr: 5581
merge_commit: d9ffc47f4f214f0cdadb1561ef5f00c6548a4ece
post_validation_correction: 0c7dc94b2a0397c7bd0784b79b5e35c64bf5986f
---

# PR #5550 rhwp-agent 조회 CLI 묶음 검토

## 라우팅

```text
base route: collaborator maintainer integration
modifiers: external PR cherry-pick, cumulative validation, maintainer correction,
individual review record
loaded documents: pr_review_workflow.md, pr_review/README.md,
collaborator_self_merge.md, intake_and_review.md, post_merge.md
```

## 적용 범위

| 항목 | 값 |
| --- | --- |
| Source PR | [#5550](https://github.com/edwardkim/rhwp/pull/5550) |
| 작성자 | `kevin9327` |
| 원본 head | `d53b4e355b1c1c59238f637ea8e1dd650e1bacce` |
| 적용 commits | 7개, 아래 계보 참조 |
| 메인터너 보정 | `27e733d143c3d2939540bd53c43386ac237f41dc` |

- `rhwp-agent`에 문서 정보, 검색, 필드, 표, 페이지, 차트, 보안 조회 명령을 추가한다.
- 새 편집 동작을 만들지 않고 기존 `DocumentCore` 조회 API를 통해 읽기 전용 결과를 제공한다.
- JSON 경로는 schema version 및 untrusted 표지를 유지하고, 미지 플래그는 usage exit code 2로 고정한다.

## 메인터너 보정

원 PR의 `tests/agent_cli_pack_contract.rs`는 현재 정책상 새 integration source로 등록되지 않아
`cargo test --test agent_cli_pack_contract` 대상이 아니었다. 이를
`tests/cases/agent_cli_pack_contract.rs`로 이동했다. review/CI의
`rust-test-suite-manifest.mjs --prepare`가 `regression_suite_004` harness에 포함하므로 테스트가
파생 산출물을 커밋하지 않고도 실제 CI worker에서 실행된다.

## 검증 기록

- `node scripts/rust-test-suite-manifest.mjs --prepare`: `agent_cli_pack_contract -> regression_suite_004::agent_cli_pack_contract`
- `CARGO_TARGET_DIR=target/review-kevin9327-agent cargo nextest run --test regression_suite_004 -E 'test(/(^|::)agent_cli_pack_contract::/)'`: 44 passed
- `CARGO_TARGET_DIR=target/review-kevin9327-agent cargo clippy --bin rhwp-agent -- -D warnings`: 통과
- `cargo fmt --all -- --check`: 통과
- `node scripts/rust-test-suite-manifest.mjs --check`: 통과
- `node scripts/rust-unit-test-tiers.mjs --check`: 통과
- `0c7dc94b2`가 `capabilities --json`의 외부 명령 목록을 새 80개 공개 명령으로 갱신해 등재·실행 왕복 계약을 복구했다.
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`: 7,773 passed
- `CARGO_TARGET_DIR=target/pr-review cargo clippy --all-targets -- -D warnings`: 통과
- 통합 PR #5581 CI: Lint, Native Skia, archive와 slow·regular shard, CodeQL 세 언어, Proptest, Adapter inter-diff, Build & Test 통과

## 적용 계보

`60ae15737`, `68f39978f`, `202e9d7df`, `1e71faf5c`, `7ec61beea`, `5c6aff9e0`,
`523d76f31`은 각각 원 PR commit `0559c57e1`, `f00c2fb97`, `308f20f56`, `fba4b5617`,
`d2505a64e`, `583eaab89`, `d53b4e355`에서 `-x`로 적용했다.

## 결론

**병합 완료.** 누적 통합 PR [#5581](https://github.com/edwardkim/rhwp/pull/5581)이
`d9ffc47f4f214f0cdadb1561ef5f00c6548a4ece`로 `devel`에 병합됐다. 원 PR #5550에는 적용 계보와
메인터너 보정 근거를 남긴 뒤 종료했다.
