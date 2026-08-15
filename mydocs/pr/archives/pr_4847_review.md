---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4847 검토 - WMF/EMF DIB 치수·좌표 DoS 방어

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4847](https://github.com/edwardkim/rhwp/pull/4847) · @kevin9327 |
| 원 head | `c2b50ed88268f1cf632a8f5a31c08c6dcb49e4b9` |
| 누적 순서·적용 SHA | 13/27 · `f0d308e27` |
| 통합 기준선 | `upstream/devel@6631e7057` |
| 충돌·의존성 | 충돌 없음 · Gym metafile 입력 경계 |

WMF/EMF DIB 치수·좌표 산술과 무한 할당 경로를 제한한다. 전체 nextest와 Native Skia·GitHub code CI가
통과했다. **수용 가능**으로 판정한다.
