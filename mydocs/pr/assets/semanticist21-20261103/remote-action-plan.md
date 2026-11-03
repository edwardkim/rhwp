# 원격 처리안 — 승인 후 진행 중

작업지시자가 원격 push·Open PR·새 head CI·self-review·merge commit·원 PR 10건 완료·감사 댓글·기록·동기화·정리를 승인했다. 통합 branch를 push해 [PR #8548](https://github.com/edwardkim/rhwp/pull/8548)을 열었다. 원 기여 branch는 변경하지 않았다. 최초 CI에서 확인한 [진단 출력 보안 보정](ci-security-followup.md) 후 같은 PR에 새 head를 제출하며, CI 성공 전에는 merge하지 않는다.

## 승인된 범위

1. 최신 devel과 원 열 head·기존 본인 열린 PR 상태를 다시 조회한다. 현 검증 기준과 다르면 영향 검증을 먼저 보완한다.
2. `review/integration-20261103-semanticist21`을 메인터너 repository에 push하고 devel 대상 Open 통합 PR을 생성한다. 보정 없는 원격 원 head에 approve를 게시하지 않는다.
3. 제출 head의 required CI와 aggregate·mergeable을 확인하고 self-review를 기록한다. 로컬에서 검증한 head와 GitHub head를 구분한다.
4. 최신 base의 merge-tree와 exact-head CI를 재확인한 뒤 merge commit으로 통합한다. 원 열 head의 조상 관계와 기여자 commit을 보존하며 squash하지 않는다.
5. 통합 merge SHA·devel 포함·원 PR 10건의 API 상태를 재조회하고 검증한 기여별 감사 댓글을 게시한다. 자동으로 MERGED가 반영되지 않은 원 PR은 base에 원 head와 보정이 포함된 것을 증명해 통합 수용 완료로 정리하는 범위를 승인 요청한다.
6. 검토 기록·연결 이슈 상태를 대조하고 devel 동기화·검토 branch/산출물 정리를 수행한다. 다른 작업의 WIP는 보존한다.

## 승인 근거

AGENTS.md의 “GitHub comment, remote push, PR 생성은 사용자 승인을 받은 뒤 수행한다”와 `maintainer_general.md` 5–6에 따라 이번 로컬 승인과 원격 승인을 구분한다. 한 번의 승인으로 위 범위를 연속 수행한다. workflow 취소·재실행·설정 변경이 필요한 장애는 실제 원인과 대상을 별도로 제시한다.

## 검토 가능한 제출물

- [PR 본문](pr-body.md)
- [최종 검증](validation.md)
- [원 head·적용 merge](intake.md)
- 원 PR별 아래 댓글 초안. 실제 통합 PR·merge SHA·CI URL은 성공 후 채우고 asset의 devel 포함을 API/Git으로 확인한 뒤 게시한다.

- [#8211 댓글](comments/pr-8211.md) · [개별 검토](../../pr_8211_review.md)
- [#8235 댓글](comments/pr-8235.md) · [개별 검토](../../pr_8235_review.md)
- [#8281 댓글](comments/pr-8281.md) · [개별 검토](../../pr_8281_review.md)
- [#8287 댓글](comments/pr-8287.md) · [개별 검토](../../pr_8287_review.md)
- [#8290 댓글](comments/pr-8290.md) · [개별 검토](../../pr_8290_review.md)
- [#8292 댓글](comments/pr-8292.md) · [개별 검토](../../pr_8292_review.md)
- [#8293 댓글](comments/pr-8293.md) · [개별 검토](../../pr_8293_review.md)
- [#8298 댓글](comments/pr-8298.md) · [개별 검토](../../pr_8298_review.md)
- [#8299 댓글](comments/pr-8299.md) · [개별 검토](../../pr_8299_review.md)
- [#8308 댓글](comments/pr-8308.md) · [개별 검토](../../pr_8308_review.md)
