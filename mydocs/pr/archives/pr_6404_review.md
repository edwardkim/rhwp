---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6404
issue: 3884
author: kevin9327
---

# PR #6404 review - bench 실패 stdout 계약

## Metadata

| 항목 | 확인값 |
| --- | --- |
| 원 PR head / 통합 적용 | `29561396e913b6794871e8a9e5f81c391bfc79e6` / `3fdadea` |
| 규모 | 2 files, `+152/-12`, 1 commit |
| 작성 시점 원 PR 상태 | Open, non-draft, check `SUCCESS`, `mergeStateStatus=UNKNOWN` |

## 검토와 판단

- `bench` 실패 경로에서 stdout을 비워 JSON 소비자가 오류 문자열을 정상 결과로 읽지 않게 하고, `issue_3884_bench_json_failure_stdout` 실 CLI contract가 이를 잠근다.
- CLI failure envelope 변경이며 HWP/HWPX visual fixture와 무관하다. 통합 후보의 full nextest와 clippy는 사전 통과했다.
- 원 PR comment는 자동 quota 안내뿐이다.

**수용.** 실패 채널을 stderr로 한정하는 계약과 test가 일치한다. merge 전 최신 CI를 재확인한다.
