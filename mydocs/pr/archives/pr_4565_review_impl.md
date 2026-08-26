---
kind: pr-review-implementation
status: pending-full-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4565 메인터너 source 보정 계획

## 기준과 커밋

| 단계 | 커밋 / 기준 | 상태 |
| --- | --- | --- |
| contributor 기능 1 | `ff4ba1e53` | 완료 |
| contributor 보완 2 | `eaa7aac9f` | 완료 |
| 최신 기준선 merge | `efbdd88db` (`upstream/devel@296d1fb3f`) | 완료 |
| TypeScript 호환 보정 | `4c511dd85` | 완료 |
| review·오늘할일 | 이 문서와 `pr_4565_review.md`, `orders/20260812.md` | 진행 |

## 처리 순서

1. #4565 source `eaa7aac9f`와 `upstream/devel@296d1fb3f`를 고정하고, source를 첫 부모로 하는
   `efbdd88db` merge commit을 같은 가시성 브랜치에 만들었다.
2. 최신 기준선에서만 드러난 `TS18046`을 `4c511dd85`로 보정하고 Studio 검증을 완료했다.
3. 원 PR별 archive review와 오늘할일을 하나의 trailing 문서 commit으로 고정한다.
4. 동일한 #4565 source branch `issue-4564-embed-chrome-profile`에 fast-forward push한다.
5. 최신 #4565 head의 CI·CodeQL·Render Diff·mergeable을 확인한다. 모두 성공해도 작업지시자 승인 전에는
   merge하지 않는다.
6. merge 후 #4564 종료 상태, local branch와 검토 target 정리를 merge 후속 처리 절차에서 확인한다.

## 롤백 경계

원 contributor branch와 commit은 rewrite·amend·force-push하지 않는다. source 보정 candidate가 실패하면
`4c511dd85`만 되돌리거나 별도 원인 보정을 추가하며, 원 contributor commit은 유지한다.
