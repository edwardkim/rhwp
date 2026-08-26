---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4911 검토 - COLR paint graph 캐시 키 보정

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4911](https://github.com/edwardkim/rhwp/pull/4911) · @kevin9327 |
| 원 head | `fc5aa3bac40dad9a56294468d347e743fade1e48` |
| 누적 적용 | source commit 15/17 · `ec711d3a1` |
| 통합 기준선 | `upstream/devel@441254611` |

COLR paint graph 캐시 키가 실제 paint 내용을 식별하도록 보정한다. renderer 경계 변경이므로 전체 nextest
6,383건, Native Skia 58건, Canvas visual diff와 #4918 Full CI를 확인했다. **수용 가능**이다.
