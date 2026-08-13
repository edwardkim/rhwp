---
kind: pr-review-implementation
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4711 메인터너 보정 구현 기록

## 라우팅

```text
base route: collaborator_external_pr
modifiers: intake_and_review, local_validation, visual_fixture_evidence,
  review_only_fast_pass
```

`maintainerCanModify=true`인 contributor source branch `planet6897/rhwp:local/task2148-3sum-prep`를
직접 보정하는 9.3.1 경로를 사용했다. 원 contributor commit은 재작성하지 않았고, local visibility
branch `review/planet6897-20260813`에서 원 head `1ed3db584e` 위에 보정 commit만 추가했다.

## stage

1. PR head, fork ref, local source parent를 모두 `1ed3db584e`로 고정하고 최신 `upstream/devel`
   `3c7b89356`과의 merge tree를 확인했다.
2. Windows Hancom COM에서 C의 복원 합 차 `−706.89px`가 성공 코드 0으로 끝나는 것을 재현했다.
3. `613e427f5`에서 opt-in `--max-total-diff-px` gate와 COM 비의존 단위 검증을 추가하고,
   C는 종료 3·A는 종료 0을 실제로 확인했다.
4. 변경 파일이 모두 LFS 비대상임을 판독하고 dry-run 뒤 contributor source branch에 push했다.
5. 동일 SHA의 GitHub Full CI와 CodeQL이 녹색이 된 것을 확인했다.

## 경계와 rollback

gate는 기본 비교 결과를 실패로 일반화하지 않는다. 자동화 호출만 의도에 맞는 절대 합계 허용치를
지정한다. 보정을 취소해야 하면 contributor 원 commit을 건드리거나 force-push하지 않고,
`613e427f5`를 되돌리는 새 commit을 같은 source branch에 추가한다.

이 문서와 review 문서는 code candidate 뒤의 review-only trailing commit이다. push 뒤 fast-pass
preflight와 최신 Build & Test aggregate를 확인한 다음에만 merge 판단으로 진행한다.
