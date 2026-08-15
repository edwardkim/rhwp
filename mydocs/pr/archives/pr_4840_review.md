---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4840 검토 - HWPX HwpUnitChar 2배 스케일 오버플로

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4840](https://github.com/edwardkim/rhwp/pull/4840) · @kevin9327 |
| 원 head | `368494dd07ad3da3c79a3ea0e0612a521e12f187` |
| 누적 순서·적용 SHA | 9/27 · `b2d0bfa6b` |
| 통합 기준선 | `upstream/devel@6631e7057` |
| 충돌·의존성 | 충돌 없음 · HWPX 숫자 입력 경계 |

손상 HWPX의 HwpUnitChar 2배 변환에서 정수 오버플로가 패닉이 되지 않도록 처리한다. 전체 nextest와
GitHub code CI가 통과했다. **수용 가능**으로 판정한다.
