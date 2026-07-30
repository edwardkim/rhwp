---
kind: pr_review_plan
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-31
---

# PR #3597 및 kevin9327 적층 PR 처리 계획

## 입력과 commit 경계

| 구분 | SHA / 범위 | 처리 원칙 |
| --- | --- | --- |
| #3597 contributor 원 변경 | `c930cac3c` | rewrite·amend·force-push 금지 |
| #3597 base update | `f9b9ea9f3` | 최신 `upstream/devel`을 source head에 병합 |
| #3597 보정 | `7d6c4a5bf`, `71c798465` | 계약 테스트 및 `src/main.rs` rustfmt를 각각 별도 commit |
| review 묶음 | 이 문서·`pr_3597_review.md`·`20260731.md` | code 보정과 분리한 docs commit |

적층 관계는 `#3597 → #3607`, `#3599 → #3602`이며, #3610은 독립 문서 PR이다. 최신 `devel`에서
cherry-pick 누적 검토를 먼저 완료했지만, 이 통합 branch는 검증용일 뿐 contributor branch에 push하지 않는다.

## 실행 순서

1. #3597의 원격 head, `git ls-remote`, local `review/pr3597-maintainer` SHA를 다시 대조한다.
   contributor 원 head `c930cac3c`가 바뀌지 않았을 때만 base-update·보정·docs commit을 source branch에
   추가한다.
2. source 원 head와 local HEAD 사이 파일을 `git check-attr filter` 및 `git lfs status`로 먼저 판독한다.
   LFS 대상이 없으면 `GIT_LFS_SKIP_PUSH=1` dry-run과 실제 push를 사용한다. LFS 대상이면 정상 pre-push
   hook을 포함한다.
3. 보정 commit이 있으므로 review-only fast-pass를 쓰지 않는다. push한 최신 #3597 head의 full CI, 정확한
   head SHA, `MERGEABLE` 상태를 확인한 뒤 approve·admin merge한다.
4. merge 뒤 #3596 auto-close 상태를 재조회하고, 실제 LF를 가진 contributor comment를 게시한 뒤 local
   `devel`을 동기화한다.
5. #3599를 새 `devel`로 update하여 current code candidate의 full CI를 확인한다. review·오늘할일 trailing
   docs commit은 A 경로 fast-pass 조건(후보 Build & Test와 최신 head preflight·aggregate)으로 판정한다.
6. #3599 merge 뒤 같은 방식으로 #3602를 update·검토·fast-pass·merge한다. #3601 close는 merge 뒤 확인한다.
7. #3607은 #3597 merge 뒤 최신 `devel`로 update한다. 원 PR 크기(+1,000 lines)를 별도 범위 검토하고,
   current code head full CI 뒤 review-only fast-pass를 확인해 merge한다. #3600 close는 merge 뒤 확인한다.
8. #3610은 마지막으로 update한다. 전체 diff가 review-only 허용 경로면 B 경로 fast-pass의 최신 preflight와
   aggregate를 확인해 merge한다. 각 PR의 review 문서는 해당 원 PR head에 따로 넣고, 오늘할일은 최초 원 PR
   review 묶음에만 갱신한다.

## 중단·rollback 기준

- source SHA, remote ref, local ref가 다르거나 contributor 새 commit이 있으면 push하지 않고 fetch부터 다시
  시작한다.
- 최신 full CI 실패, aggregate pending/failure, mergeability 변화, LFS object/lock 필요은 해당 PR에서 멈추고
  원인을 분리한다. 이전에 merge한 PR을 되돌리거나 contributor history를 rewrite하지 않는다.
- collaborator 보정에 문제가 발견되면 `7d6c4a5bf` 또는 `71c798465`만 별도 revert할 수 있어 contributor
  원 변경과 review 문서가 독립적으로 보존된다.
- 모든 PR의 post-merge 확인이 끝난 뒤에만 전용 Cargo target과 검토 branch를 정리한다.
