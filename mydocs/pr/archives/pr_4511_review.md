---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4511 리뷰 - 캡슐 Ed25519 서명

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4511](https://github.com/edwardkim/rhwp/pull/4511) · @kevin9327 |
| base / 최신 head | `devel` / `f213d4239f176e61fb7246c3e41d232cac7aa16a` |
| 규모 | 1,712 추가 / 27 삭제, 18 파일, 3 commit |
| 작성 시점 CI | GitHub required `Build & Test` 통과. merge 전 최신 상태 재확인 필요. |

## 범위와 검토

캡슐 발급·검증·계보 귀속의 Ed25519 서명 경로와 계약 테스트를 추가한다. 후속 harness·anchor·gate의
전제 기능이므로 누적 순서상 먼저 적용했다. `signing_contract`를 포함한 전체 nextest 회귀에서 통과했다.

**권고: 최신 head와 required check를 merge 직전에 재확인하고 통합 수용.**
