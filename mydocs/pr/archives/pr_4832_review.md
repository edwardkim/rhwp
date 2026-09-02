---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4832 검토 - Gym convert/export 손상 입력 DoS 방어

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4832](https://github.com/edwardkim/rhwp/pull/4832) · @kevin9327 |
| 원 head | `161ab859e2ff1df8ad57220f448bb7a7365e1b3e` |
| 누적 순서·적용 SHA | 6/27 · `648826bb7` |
| 통합 기준선 | `upstream/devel@6631e7057` |
| 충돌·의존성 | 충돌 없음 · Gym writer/convert 경계 |

페이지 u32 산술과 WMF 색인 경계를 검사해 convert·export-hwpx·export-markdown의 손상 입력 패닉을 막는다.
전체 nextest와 GitHub code CI가 통과했다. **수용 가능**으로 판정한다.
