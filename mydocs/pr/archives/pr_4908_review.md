---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4908 검토 - DAR·DAP/1.0·DATP/1.0

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4908](https://github.com/edwardkim/rhwp/pull/4908) · @kevin9327 |
| 원 head | `c038cba6cf13308d0026afedfb2a6bdd8639a5c3` |
| 누적 적용 | source commit 12-13/17 · `876d77893`, `6baee1ac3` |
| 통합 기준선 | `upstream/devel@441254611` |

문서 에이전트 런타임과 DAP/DATP 프로토콜을 추가한다. 메인터너 보정 `51e9daa96`은 입력과 같은 commit
출력을 거부하고, 제안·영수증의 op/params 검증을 replayer 접근 전에 수행하며, replay·verify 경로를
conformance에 포함했다. `dar/conformance.py --self-check`, Python 계약 13건, #4918 Full CI·CodeQL로
**수용 가능**을 확인했다.
