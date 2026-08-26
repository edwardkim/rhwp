---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4821 검토 - HWPX variant paragraph 위치 오버플로

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4821](https://github.com/edwardkim/rhwp/pull/4821) · @kevin9327 |
| 원 head | `fafa62b39d7cddf3c30989ae5c6702e401e50e69` |
| 누적 순서·적용 SHA | 2/27 · `116e46c01` |
| 통합 기준선 | `upstream/devel@6631e7057` |
| 충돌·의존성 | 충돌 없음 · parser 입력 경계 보정 |

`normalize_variant_paragraph_vpos`의 i32 산술을 포화 처리해 퍼징 입력이 패닉으로 이어지지 않게 한다.
로컬 전체 nextest, Native Skia, OVR 및 GitHub code CI가 통과했다. **수용 가능**으로 판정한다.
