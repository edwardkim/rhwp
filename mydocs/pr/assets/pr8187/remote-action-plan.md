# #8187 원격 처리안 — 승인 전

이 문서는 제안이다. 현재 승인 범위는 로컬 보정·필수 검증이며 아래 원격 변경은 실행하지 않았다.
최종 로컬 gate가 모두 통과하고 검토 기록·제출 head를 확정한 뒤 이 계획의 승인을 요청한다.

## 대상과 사전 조건

- 원 PR: https://github.com/edwardkim/rhwp/pull/8187
- 원 contributor head: `semanticist21/rhwp:codex/platy-official-engine-contracts`,
  `32df7b71a6e731f767e4945e3e1d79ca0cac3e58`.
- 로컬 후보: `review/pr-8187-logical-editing`. 원 13커밋과 최신 devel의 merge를 보존한다.
- 기준 devel: `4e2cc6bb36576b95d58d4c76cd833281ea652554`.
- 2026-11-03 재조회: OPEN/non-draft, `maintainer_can_modify=true`, reviewer edwardkim,
  원격 head와 기준 devel은 위 값과 같았다. 아직 원 PR은 원 head의 충돌 상태다.

실행 직전 head/devel과 권한을 다시 조회한다. head가 바뀌었으면 기여자 변경을 덮어쓰지 않고 새
내용을 검토·통합한다. base가 바뀌면 해당 기능 보존 절차를 이행한다. 이 후보는 원 head의 후손이므로
원 이력을 삭제하는 force-push를 사용하지 않는다.

## 승인 요청 범위

1. 제출 head와 로컬 gate·TSV·대표 PNG를 확인한 뒤 원 contributor branch로 fast-forward push한다.
2. 기존 원 PR 본문을 보존하며 [메인터너 검증 보충](maintainer-pr-body-addendum.md)을 추가한다.
   게시 직전 본문을 다시 읽어 기여자의 새 설명을 덮어쓰지 않는다. 새 대체 PR은 만들지 않는다.
3. 새 exact head의 필수 GitHub CI와 충돌 상태를 확인하고 self-review를 기록한다.
   옛 head의 성공을 재사용하지 않는다. CI 취소·재실행이나 workflow 명령 변경이 필요하면 정본의
   별도 승인 규칙을 따른다. 실패한 필수 검사를 우회하지 않는다.
4. fresh base merge-tree가 clean이고 최신 head의 필수 CI가 모두 통과하면, 승인된 범위에서 원 PR을
   merge commit 방식으로 통합한다. 직전 상태를 다시 조회하고 `--match-head-commit`으로 head를 고정한다.
5. 실제 merge SHA·CI·검토 기록 URL을 확정한 [감사 댓글](merge-comment-draft.md)을 원 PR에 게시한다.
   #7444 전체를 닫거나 이 PR 밖의 해결을 주장하지 않는다.
6. merge·후속 CI를 확인하고 기록을 archive로 반영한다. devel을 동기화한 뒤 이번 review branch와
   base worktree를 정리한다. 실제 사용 종료 후 정본 7.7에 따라 root output 전체와
   target/pr-review/debug를 정리하고 완료를 검증한다. 다른 작업의 source/WIP/worktree는 보존한다.

GitHub 변경은 승인 뒤 로컬 gh/REST를 사용한다. 원격 게시·권한 변경·CI 운영의 상세 규칙은
[GitHub 운영](../../../manual/github_operations.md),
[메인터너 절차](../../../manual/pr_review/maintainer_general.md),
[후속 정리](../../../manual/pr_review/post_merge.md)를 따른다.
