# PR #3409 검토 기록 — devel PR stale run 자동 force-cancel

| 항목 | 내용 |
| --- | --- |
| PR | [#3409](https://github.com/edwardkim/rhwp/pull/3409) — `ci: devel PR stale run 자동 force-cancel (#3406)` |
| 작성자·준비자 | `@jangster77` (collaborator self-merge) |
| base / head | `devel` / `task/3406-ci-force-cancel` |
| 관련 이슈 | [#3406](https://github.com/edwardkim/rhwp/issues/3406) |
| 작성 시점 참고 head | `c06a4c398387daf1367291b47fda1abc97aefa2f` |
| 작성 시점 규모 | 7 files, +264 / -7, 4 commits; 이 문서 추가 뒤 최신 head·CI를 다시 확인해야 함 |
| 라우팅 | base: collaborator self-merge; modifiers: intake/review, local validation, post-merge |

## 변경 범위와 판정

새 `Cancel stale PR runs` workflow는 `devel` 대상이며 head가 같은 저장소인 PR의 `synchronize`
event에서 실행된다. 같은 PR 번호의 `pull_request` run 중 현재 live head SHA와 다른 active
(`queued`, `in_progress`, `pending`, `requested`, `waiting`) run만 Actions force-cancel API로 취소한다.

각 취소 직전에 PR live head를 다시 읽는다. 따라서 연속 Update branch에서 먼저 시작한 reaper가 이후
최신 SHA의 run을 취소하지 않는다. 기존 CI·CodeQL·Render Diff의 concurrency/job/required check는 바꾸지
않는다.

workflow에는 PR source checkout, shell step, PR 제공 script 실행이 없다. 권한은 `actions: write`,
`contents: read`, `pull-requests: read`로 한정했다. GitHub가 `pull_request` token을 읽기 전용으로
낮추는 external fork PR은 job을 의도적으로 skip하고, [다수 PR과 update branch 처리](../../manual/pr_review/multi_pr_update_branch.md#25-update-branch-뒤-이전-sha-ci-강제-취소)의
수동 force-cancel 절차를 적용한다.

renderer·layout·Rust·WASM·frontend·fixture·golden/baseline 변경이 없으므로 시각 검증, Cargo, WASM
검증은 대상이 아니다. CI workflow 변경이므로 workflow 구문·권한·조건과 최신 GitHub Actions를 검증
대상으로 한다.

## 로컬·격리 검증

- `actionlint .github/workflows/cancel-stale-pr-runs.yml` 통과
- Ruby YAML parse 통과
- embedded GitHub Script `node --check` 통과
- `python3 scripts/check_markdown_links.py --changed-from upstream/devel` 통과
- `git diff --check` 통과
- 격리 동일 저장소 PR [jangster77/rhwp#9](https://github.com/jangster77/rhwp/pull/9)에서 base를 전진시킨
  뒤 Update branch를 수행했다.
  - 이전 `db82b24`의 CI `30198327109`, CodeQL `30198327121`, Render Diff `30198327120`은 모두
    `completed/cancelled`가 됐다.
  - 새 `4a049c8`의 CI `30198335775`, CodeQL `30198335764`, Render Diff `30198335768`은 자동으로
    다시 시작됐다.
  - [reaper run 30198335771](https://github.com/jangster77/rhwp/actions/runs/30198335771)은 success이며
    세 stale run의 force-cancel 로그를 남겼다. 기존 `pull_request_target` test reaper는
    [run 30198334829](https://github.com/jangster77/rhwp/actions/runs/30198334829)에서 `skipped`여서
    결과에 개입하지 않았다.

## 위험과 후속 처리

- external fork PR은 자동 취소 범위 밖이며, 이 권한 경계를 넓히지 않는다.
- `main`은 메인터너 고유 릴리즈 브랜치이므로 변경하지 않는다.
- merge 뒤 #3406의 close 상태를 확인한다. default branch가 `main`이므로 `Closes #3406`만으로
  auto-close되지 않으면 devel sync 뒤 승인된 수동 close/comment 경로를 적용한다.

## 최종 권고

**merge 권고**. 최종 조건은 이 문서 추가 뒤의 최신 PR head에서 CI, CodeQL, Render Diff가 모두 통과하고,
`MERGEABLE`/비차단 상태를 재확인하며, 작업지시자의 merge 승인을 받는 것이다.
