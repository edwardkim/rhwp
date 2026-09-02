---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4866 검토 - 수식 파서 깊이·괄호 복잡도 DoS 방어

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4866](https://github.com/edwardkim/rhwp/pull/4866) · @kevin9327 |
| 원 head | `908b315a69be996fa68f5bddb243901279e9a1f3` |
| 누적 순서·적용 SHA | 21/27 · `c39304d49` |
| 통합 기준선 | `upstream/devel@6631e7057` |
| 충돌·의존성 | 충돌 없음 · eqedit/equation parser 경계 |

수식 중첩 깊이와 괄호 탐색의 O(n²) 경로를 제한한다. 전체 nextest·Native Skia·GitHub CI가 통과했다.
**수용 가능**으로 판정한다.
