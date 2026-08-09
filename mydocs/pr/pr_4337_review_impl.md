---
kind: pr-review-implementation
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4337 메인터너 보정·통합 계획

## Commit 소유권과 순서

Contributor history는 그대로 보존한다.

1. `85fb44bf63e753af6e4c03055f9d96c7b23ffaba` — contributor의 PyPI/npm 릴리스 파이프라인
2. `13a41d380e3ad117e680872c3d5148df142cfaf6` — maintainer code/test/workflow 보정
3. 이 문서를 포함하는 별도 trailing review-doc commit — `mydocs/pr/` 두 파일만 변경

보정 commit의 유일한 parent는 contributor head다. contributor commit을 amend, rebase,
merge 또는 force-push하지 않는다. 검토 근거는 [PR #4337 검토 기록](pr_4337_review.md)에 있다.

## 단계

1. **완료 - 접수 고정:** contributor head, 규모, merge 상태와 기존 check를 기록했다.
2. **완료 - 로컬 보정:** dispatch source 고정과 계약 테스트를 하나의 maintainer commit으로 추가했다.
3. **완료 - 로컬 검증:** focused unittest, workflow test 배선, commit parent와 diff check를 확인했다.
4. **대기 - push 승인:** 작업지시자 승인 뒤 code correction과 trailing review-doc commit만 PR source에
   fast-forward push하고 원격 head SHA를 다시 확인한다.
5. **대기 - Full CI:** code/test/workflow가 바뀌었으므로 fast-pass가 아니라 최신 head의 Full CI,
   CodeQL 및 branch protection required aggregate를 기다린다. 실제 package matrix도 이 단계에서 확인한다.
6. **대기 - 최종 판단:** 최신 mergeability와 check가 모두 녹색이어도 별도 작업지시자 merge 승인 전에는
   GitHub review 게시나 merge를 수행하지 않는다.
7. **merge 후:** merge SHA를 확인하고 review 기록을 archive로 이동할지 판정한 뒤 branch/worktree를 정리한다.

## Rollback

- push 전에는 이 visibility branch와 worktree만 정리하면 원 PR에는 영향이 없다.
- push 뒤 merge 전에는 trailing review-doc commit을 먼저 revert하고, 필요하면
  `13a41d380e3ad117e680872c3d5148df142cfaf6`을 revert해 contributor head로 되돌린다.
- merge 뒤에도 동일한 역순 revert를 사용한다. contributor commit을 rewrite하거나 force-push하지 않는다.

현재 단계에는 push, GitHub review/comment 또는 merge 승인이 없다.
