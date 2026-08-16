---
kind: pr-review-implementation
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4935 메인터너 보정 기록

| 순서 | commit | 역할 |
| --- | --- | --- |
| 1 | `59b587c02` | 원 PR의 locate·diagnose·repair·re-verify 루프 적용 |
| 2 | `9c139b118` | OS별 가짜 `rhwp` 래퍼로 Python 회귀 시험을 Linux/macOS에서도 실행 가능하게 보정 |

원 시험은 `.bat`만 만들었기 때문에 Linux에서 가짜 바이너리를 spawn하지 못했다. 보정은 Windows의
배치 래퍼를 유지하고 POSIX에서는 `exec python ... "$@"` shell 래퍼를 만든다. 루프 알고리즘과
실제 `rhwp` 호출 계약은 바꾸지 않는다. 관련 Python 시험 33건이 통과했다.
