---
kind: pr-review
status: review-ready
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4302 검토 - MCP opt-in 통계

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4302](https://github.com/edwardkim/rhwp/pull/4302) / `kevin9327` |
| 관련 roadmap | #3907 R80 |
| base / source head | `devel` / `e6be401d40c5f8af88265482095a791a223278e6` |
| 누적 적용 / 보정 | `2c25762cc`, `6048d3d7a` |
| 접수 참고 상태 | MERGEABLE / BLOCKED, 2 files, +189 / -2 |

## 판정

**수용 권고, 메인터너 보정 포함.** `--stats`는 기본 비활성이며 stdout JSON-RPC를 오염시키지
않고 종료 시 stderr에 call/error count만 쓴다. 다만 caller가 보낸 임의 tool name을 map key로
저장하면 unique key로 메모리를 늘릴 수 있어, 보정은 선언 도구명만 허용하고 그 밖은 고정 unknown
bucket으로 보낸다. 32KiB unknown name contract와 전체 MCP contract가 통과했다.

GitHub `BLOCKED`는 접수 시점 참고값이며, merge 전 최신 checks를 재확인한다. 공통 검증은
[통합 검토 계획](pr_4282_review_impl.md)에 있다.
