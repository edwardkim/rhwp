---
kind: pr-review-implementation
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4271 메인테이너 보정 계획 — pending 서식 삽입 undo 복원

## 검토 경로

```text
base route: maintainer_general.md
modifiers: collaborator_external_pr.md, intake_and_review.md, local_validation.md,
           rework_and_exceptions.md, multi_pr_update_branch.md, post_merge.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  maintainer_general.md, collaborator_external_pr.md,
                  intake_and_review.md,
                  local_validation.md, rework_and_exceptions.md,
                  multi_pr_update_branch.md, post_merge.md
contributor head: 2a3ab45e1e533b0670fa9d7e5fcbe74fe635bff6
maintainer correction: 034df2ec0
integrated local head: edc1869ea
integrated devel: f94fe5e4f834867d830cb7dacfd1d0043d9383d5
final PR head: 06d3ec7e5167e03b16561c28d509f282c709a745
merge commit: d4834d7375c5fa7cff850c378e8c939de6a6e90e
```

## 목표

contributor가 보정한 pending 수명주기 네 경로는 보존한다. 메인테이너 fresh Node WASM
검증에서 새로 확인한 `InsertTextCommand` undo 뒤 원문 글자모양 오염을 수정하고, 최신
`devel`의 #4272 변경과 충돌 없이 같은 PR head에 반영한다.

## 원인

pending 서식 삽입은 텍스트를 먼저 넣고 삽입 범위에 새 `char_shape_id`를 적용한다. undo의
텍스트 삭제가 이 범위의 시작·끝 `CharShapeRef`를 삭제 시작 위치로 모으면, 현재
`Paragraph::delete_text_at()`은 같은 `start_pos`의 첫 ref를 남긴다. 삽입 범위 뒤 원문이
생존하는 경우에는 마지막 ref가 오른쪽 원문의 글자모양이므로 첫 ref를 남기면 적용한
pending 서식이 원문 끝까지 퍼진다.

## 단계

1. `Paragraph::delete_text_at()`에 오른쪽 생존 텍스트가 있을 때 삭제 시작 위치로 모인
   중복 `CharShapeRef` 중 마지막, 즉 오른쪽 원문 글자모양을 보존하는 최소 정규화를 넣는다.
   문단 끝까지 삭제하는 경우에는 기존 첫 ref 보존 동작을 유지한다.
2. model unit test에 서식이 다른 가운데 범위를 삭제할 때 오른쪽 글자모양이 보존되는
   RED→GREEN을 추가한다. 문단 끝 삭제의 기존 동작도 함께 고정한다.
3. `pending-char-shape.runner.mjs`의 시나리오 2에서 undo 뒤 텍스트뿐 아니라 주변 원문
   `bold`·`textColor`가 baseline으로 돌아왔는지 직접 단언한다. 시나리오 간 공유 문서가
   오염되면 즉시 앞 단계에서 실패하도록 만든다.
4. focused Rust test, 표준 Docker web·Node WASM, TypeScript, focused Studio test와 Studio
   전체 테스트를 순차 실행한다. Rust 변경이 생기므로 GitHub full CI 전에 필요한 로컬
   Rust gate 범위도 `local_validation.md`에 따라 실행한다.
5. 기능 보정 commit을 먼저 고정한 뒤 최신 `devel`을 review branch에 병합한다.
   `input-handler.ts` import는 #4272의 `cellAxisPath`·`cellParaIndexOf`와 #4271의
   `applyCharShapeModsToRange`를 모두 보존하고, 오늘할일은 양쪽 PR 기록을 모두 남긴다.
6. merge tree 검증과 review 문서 갱신 뒤 로컬 커밋을 준비한다. contributor fork push,
   GitHub review/comment와 최종 merge는 각각 작업지시자 별도 승인 뒤 수행한다.

## 실행 결과

- [x] 오른쪽 원문이 남는 삭제 경계에서는 마지막 `CharShapeRef`를 보존하고, 끝 삭제는
  기존 첫 ref 계약을 유지하도록 `Paragraph::delete_text_at()`을 보정했다.
- [x] Rust model unit test 2건과 Studio undo baseline 단언을 추가했다.
- [x] 기능 보정 commit `034df2ec0`을 contributor 커밋 뒤에 별도로 기록했다.
- [x] 최신 `upstream/devel` `f94fe5e4f`을 merge commit `edc1869ea`로 통합했다.
- [x] 두 충돌에서 #4271·#4272 helper와 #4271·#4272/#4276 오늘할일 기록을 모두 보존했다.
- [x] 통합 head에서 fmt, clippy, nextest 5,486건, Docker web·Node WASM, TypeScript,
  focused Studio 5건, Studio 전체 823건, diff 검사를 통과했다.
- [x] contributor fork의 같은 source branch에 `GIT_LFS_SKIP_PUSH=1` non-force push했다.
- [x] 최종 head `06d3ec7e5`의 Full CI·CodeQL·Render Diff 통과와 `MERGEABLE / CLEAN`을 확인했다.
- [x] [승인 review](https://github.com/edwardkim/rhwp/pull/4271#pullrequestreview-4890430796)를 게시했다.
- [x] 작업지시자 승인 뒤 merge commit `d4834d737`로 `devel`에 반영했다.

## rollback 경계

- 기능 보정 commit은 Rust 삭제 경계 정규화, 해당 unit test, Studio undo 단언만 포함한다.
- 최신 `devel` 반영은 별도 merge commit으로 유지해 contributor 원 커밋과 메인테이너
  보정의 저자·범위를 구분한다.
- 기존 contributor 커밋은 amend·rebase·force-push하지 않는다.
