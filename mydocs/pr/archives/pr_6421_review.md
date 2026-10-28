---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6421
issue: 3884
author: kevin9327
---

# PR #6421 review - inspect·edit capability 자기서술

## Metadata

| 항목 | 확인값 |
| --- | --- |
| 원 PR head / 통합 적용 | `2a3301f9d6f782a49bcbc8113a2648bb5ebc487b` / `429636f` |
| 규모 | 2 files, `+138/-2`, 1 commit |
| 작성 시점 원 PR 상태 | Open, non-draft, check `SUCCESS`, `mergeStateStatus=UNKNOWN` |

## 검토와 판단

- inspect와 edit 하위 명령을 capability 자기서술에 잠그는 `issue_3884_g4_inspect_edit_subcommands` contract와 troubleshooting 문서를 추가한다.
- HWP render path를 바꾸지 않아 visual sweep 대상이 아니다. full nextest가 사전 통과했고 comment는 자동 quota 안내뿐이다.

**수용.** capability 표면의 누락을 실 CLI contract로 막는다.
