# PR #3399 collaborator 보정 실행 계획

## source와 commit 경계

| 구분 | SHA·내용 |
| --- | --- |
| contributor source head | `2b8531da40efa2d406d214f57488ffa3d19a6dc9` — 보정 시작 시점 참고값 |
| collaborator code·test 보정 | `fef78b9e` — fidelity 비교 하네스 검증 계약 보강 |
| review 기록 | 이 문서, `pr_3399_review.md`, `mydocs/orders/20260729.md`를 별도 commit으로 추가 |

contributor 원 commit은 rebase·amend·reset하지 않는다. 현재 source branch
`kevin9327/rhwp:pr/task-3398-bug-hunter-playbook`의 remote SHA와 PR `headRefOid`가 보정 시작 SHA와
다르면 push하지 않고 새 head에서 다시 검토한다.

## Stage

1. **보정·로컬·교차 호스트 검증** — 완료. 메인터너 제안 5건을 code·test·playbook에 반영하고
   Python 회귀, Markdown 링크, macOS 실제 1쪽·10쪽 smoke를 통과했다. Linux에는 Chromium과 Python
   의존성을 설치하고, Linux·Windows 모두 격리 worktree·전용 Cargo target에서 `release-test`를
   빌드한 뒤 플랫폼 기본 탐색을 사용하는 실제 1쪽 end-to-end 비교를 통과했다.
2. **review 기록** — 이 문서, `pr_3399_review.md`, 오늘할일을 같은 별도 commit으로 만들고 working
   tree가 clean인지 확인한다.
3. **push 승인 게이트** — source/remote/PR SHA 일치와 LFS 변경 없음을 확인했다. 정상 dry-run은
   contributor fork의 LFS lock 확인 권한만 실패했고 `GIT_LFS_SKIP_PUSH=1` dry-run은 fast-forward
   성공했다. 작업지시자는 두 local commit의 실제 push와 최신 head CI 모니터링을 승인했다.
4. **source push** — 승인 뒤에만 contributor head로 fast-forward push한다. push 직전 remote SHA가
   `2b8531da...`인지 다시 확인하고, push 뒤 remote ref·PR `headRefOid`·local HEAD 일치를 확인한다.
5. **최신 CI** — code·test 보정이므로 review-only fast-pass를 사용하지 않는다. 최신 head의 full CI,
   CodeQL, Render Diff와 mergeable 상태를 확인한다.
6. **merge·후속 처리** — 작업지시자 별도 승인 뒤 merge하고 관련 이슈 상태, contributor comment,
   devel sync, branch·worktree 정리를 `post_merge.md` 순서로 수행한다.

## push 명령 초안

```bash
git ls-remote --heads https://github.com/kevin9327/rhwp.git \
  refs/heads/pr/task-3398-bug-hunter-playbook
git push --dry-run https://github.com/kevin9327/rhwp.git \
  HEAD:pr/task-3398-bug-hunter-playbook

# 작업지시자 승인 뒤에만
GIT_LFS_SKIP_PUSH=1 git push https://github.com/kevin9327/rhwp.git \
  HEAD:pr/task-3398-bug-hunter-playbook
```

보정 범위에 LFS 추적 파일이나 신규 LFS object는 없다. 정상 pre-push hook dry-run에서 LFS
`locks:verify` 인증만 실패할 경우에만 가이드가 허용한 `GIT_LFS_SKIP_PUSH=1` dry-run으로 Git ref
write 가능성을 분리 확인한다.

## rollback

- 기능 보정만 되돌릴 때는 review 기록을 유지하고 `fef78b9e`를 별도 revert 검토한다.
- 원격 반영 전에 중단하면 contributor branch에는 영향이 없고 local review worktree만 정리한다.
- 최신 contributor commit이 추가되면 force-push하지 않고 새 source head 위에 보정 commit을 다시 준비한다.
