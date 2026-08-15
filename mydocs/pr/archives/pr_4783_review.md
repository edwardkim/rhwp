---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-15
---

# PR #4783 검토 - MCP 정적 prompt 표면

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4783](https://github.com/edwardkim/rhwp/pull/4783) · @kevin9327 |
| 원 head | `239090da16add750cc65880a3c95dbdf3a2e6015` |
| 통합 기준선 | `upstream/devel@44bcba400072128bdc4e4d6c05bf822e3ff60996` |
| 누적 적용 | `239090da` → `0900caf62` |
| 원 CI | Lint·Build & Test 성공, Rust CodeQL 성공 |

## 변경과 검토

MCP 초기화 capability에 `prompts`를 선언하고 `prompts/list`, `prompts/get`으로
`triage-document`, `prove-work`의 정적 안내를 제공한다. 변경은 `src/mcp_serve.rs`와
MCP 명세 ledger에 한정된다.

원 변경에는 새 prompt 표면의 응답 계약 검증이 없었다. 통합 branch의 maintainer 보정
`335fcac05`에서 목록의 이름·메타데이터, `triage-document` 본문, 알 수 없는 이름의 `-32602` 응답을
`mcp_server_contract`에 고정했다.

## 검증과 판단

- `cargo test --profile release-test --target-dir target/pr-review --test mcp_server_contract -- --nocapture`:
  25건 통과.
- `cargo fmt --all -- --check`, `git diff --check upstream/devel...HEAD`: 통과.

**통합 후보 수용.** 원격 통합 PR 생성, CI 및 원 PR close/comment는 작업지시자 승인 뒤에만 수행한다.
