---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4836 검토 - inspect injection O(n^2) DoS 방어

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4836](https://github.com/edwardkim/rhwp/pull/4836) · @kevin9327 |
| 원 head | `c5cf1eadc98b886782247f5d34ce127f36d12217` |
| 누적 순서·적용 SHA | 7/27 · `1b79fb2e4` |
| 통합 기준선 | `upstream/devel@6631e7057` |
| 충돌·의존성 | 충돌 없음 · inspect scanner 단일 수집 경계 |

주입 신호 스캐너가 발췌를 반복 수집하거나 서술어 앞 문맥을 반복 탐색하지 않도록 선형 경로로 보정한다.
전체 로컬 nextest와 GitHub code CI가 통과했다. **수용 가능**으로 판정한다.
