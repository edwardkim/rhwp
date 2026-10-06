# #3406 단계 2 완료 — 동일 저장소 `pull_request` 실제 Update branch 검증

- 이슈: [#3406](https://github.com/edwardkim/rhwp/issues/3406)
- 구현 브랜치: `task/3406-ci-force-cancel`
- 격리 검증 PR: [jangster77/rhwp#9](https://github.com/jangster77/rhwp/pull/9)

## 검증 방법

`jangster77/rhwp` 안의 서로 다른 test base/head branch로 PR을 만들었다. production workflow와 같은
`pull_request` API reaper를 source에 두고, 기존 CI·CodeQL·Render Diff가 실제로 시작된 상태에서 base를
한 commit 전진시킨 뒤 GitHub **Update branch**를 실행했다.

기존 `pull_request_target` 검증 workflow는 이 test base와 다른 branch 조건이라 job이 `skipped`되었다.
따라서 결과는 이번 `pull_request` reaper만의 결과다.

## 결과

| 구분 | SHA / run | 결과 |
| --- | --- | --- |
| 이전 head | `db82b24` | CI `30198327109`, CodeQL `30198327121`, Render Diff `30198327120` 모두 `completed/cancelled` |
| Update branch head | `4a049c8` | CI `30198335775`, CodeQL `30198335764`, Render Diff `30198335768` 자동 재시작 |
| reaper | [run 30198335771](https://github.com/jangster77/rhwp/actions/runs/30198335771) | success; 이전 run 3개 force-cancel 로그 확인 |
| 비개입 확인 | [run 30198334829](https://github.com/jangster77/rhwp/actions/runs/30198334829) | 기존 `pull_request_target` reaper job `skipped` |

reaper 로그는 `Current head 4a049c8…; force-cancelled 3 stale run(s).`을 기록했다. 이로써 `devel` 대상
동일 저장소 PR에서 필요한 `actions: write` 권한과 live head 재확인·force-cancel 경로가 실제로 동작함을
확인했다.

## 경계 확인

- production `main`은 변경하지 않았다.
- external fork PR은 GitHub의 read-only token 제약을 우회하지 않는다. workflow job이 skip되고
  `multi_pr_update_branch.md`의 수동 force-cancel 절차가 fallback이다.
- PR source checkout, shell step, PR 제공 script 실행은 없다.
