# PR #3305 메인터너 보정·반영 계획

## 기준과 commit 분리

- 검토 기준 branch: `review/kevin9327-parser-safety-20260726`
- 기준 devel: `0324cd96deb31c8a722c99360921e2e9ce55bfbe`
- contributor 원 commit / 현재 head: `1b02247ff4d0a43c45a61c0c4f07760c8116c7ff` /
  `3009a461ce3944d534aa2061c40f25bde80acb16` (`devel` update merge 포함)
- 검토용 cherry-pick: `d307baccb`
- maintainer code/report 보정: `e7a34e0bd`

`e7a34e0`은 검토 branch에서만 만든 별도 commit이다. 실제 contributor source branch에는 원 commit을
rewrite하지 않고, source SHA를 재확인한 뒤 이 보정 commit과 review 기록 commit만 순서대로 cherry-pick한다.

## 반영 단계

1. push 직전에 PR API, fork remote, local fetch branch의 source SHA가 모두 `3009a46`인지 확인한다.
2. `local/pr3305-latest`에서 `review/pr3305-maintainer`를 만들고 `e7a34e0`을 cherry-pick한다.
3. `pr_3305_review.md`, 이 계획서, 오늘할일을 별도 docs commit으로 cherry-pick한다.
4. LFS 추적물 유무와 정상 pre-push hook을 포함한 dry-run을 수행한다.
5. 작업지시자 push 승인 뒤에만 contributor fork의 `pr/task-static-bugs-bundle`로 normal push한다.
6. source head에는 이미 `devel` update merge가 포함되어 있다. rebase·force-push는 하지 않으며, 보정 뒤
   새 head의 full CI를 확인한다.
7. CI 성공과 merge 승인 뒤에만 merge한다. merge 뒤 #3301 close 상태, 원 PR comment, branch와 검토 전용
   target 정리는 merge 후속 절차를 따른다.

## rollback

source branch에는 보정과 review 기록이 별도 commit으로 추가되므로, merge 전 문제 발견 시 해당 보정 commit만
되돌리고 검증을 다시 수행할 수 있다. contributor의 `1b02247` commit은 수정하지 않는다.
