# PR #1871 Review - devel merge 이슈 자동 close 보정

## 메타

| 항목 | 내용 |
|---|---|
| PR | #1871 |
| 제목 | `task 1870: devel merge 이슈 자동 close 보정` |
| 작성자 | `jangster77` |
| base | `devel` |
| head | `task/m100-1870-devel-auto-close` |
| 관련 이슈 | #1870 |
| 문서 작성일 | 2026-07-04 |
| 문서 작성 시점 head | `8413e28a6eceaefd6f159c321eb74373677baef8` |
| 문서 작성 시점 mergeable 참고값 | `MERGEABLE` / `CLEAN` |
| 문서 작성 시점 review decision | `APPROVED` |
| maintainerCanModify | `false` |
| PR head 문서 push 경로 | `upstream`의 `task/m100-1870-devel-auto-close` 브랜치 |

`draft`, `mergeable`, `head SHA`, `CI 상태`는 volatile 값이다. merge 전에는 최신 PR head 기준으로
GitHub Actions, merge state, review 문서 포함 여부, approval 상태를 다시 확인해야 한다.

## 관련 이슈 요약

Issue #1870은 저장소 default branch가 `main`이고 운영 merge 기준이 `devel`인 구조에서, GitHub native
closing keyword가 `devel` 대상 PR merge 후 이슈를 자동 close하지 못하는 문제를 다룬다.

확인된 사례는 PR #1863이다. PR 본문에 `closes #1836`이 있었지만 `closingIssuesReferences`가 비어 있었고,
`devel` merge 후 issue auto-close가 동작하지 않았다. contributor는 `jangster77/rhwp` fork에서
`push` to `devel` 기반 workflow가 테스트 issue를 자동 close하는 것을 검증했다고 PR 본문에 기록했다.

문서 작성 시점 기준 Issue #1870은 `OPEN` 상태이며 milestone은 없다.

## 변경 범위

- `.github/workflows/close-issues-on-devel-push.yml`
  - `devel` push 이벤트에서 동작하는 workflow를 추가한다.
  - merge commit 및 associated PR을 조회한다.
  - PR title/body/commit message에 포함된 `Closes #N`, `Fixes #N`, `Resolves #N` 계열 closing keyword를
    파싱한다.
  - 같은 저장소의 open issue에 comment를 남긴 뒤 `state_reason: completed`로 close한다.

Rust 소스, 테스트 코드, 렌더링 경로, 샘플 파일 변경은 없다.

## 커밋 목록과 contributor credit

| SHA | 작성자 | 내용 |
|---|---|---|
| `3ed3ed5f51ed00e117ede6daad4a754d83a1a8c5` | Taesup Jang `<tsjang@gmail.com>` | `task 1870: devel 이슈 자동 close workflow 추가` |
| `8413e28a6eceaefd6f159c321eb74373677baef8` | Taesup Jang `<tsjang@gmail.com>` | `task 1870: closing keyword 직접 참조만 수집` |

원 contributor 커밋은 rewrite하지 않았다. Co-authored-by trailer는 없다.

## 리뷰 이력

1차 검토에서 `collectIssueNumbers`가 closing keyword 뒤 같은 줄에 나오는 모든 issue reference를 수집하는
문제를 확인했다. 예를 들어 `Fixes #1, related follow-up #2 remains open`이 `#1,#2`를 반환해 follow-up
issue까지 자동 close 대상이 될 수 있었다. 이 workflow는 `issues: write` 권한으로 실제 issue를 닫기 때문에
2026-07-03에 Request changes review를 제출했다.

contributor는 두 번째 커밋에서 `closingIssueRef` 단일 패턴으로 수정했다. 수정 후에는 closing keyword와
issue reference가 직접 연결된 경우만 수집한다. 로컬 재현에서 이전 문제 문장은 `#1`만 반환했다.

## 로컬 검증

검증은 `/private/tmp/rhwp-pr1871-review` worktree에서 최신 PR head 기준으로 수행했다.

| 항목 | 결과 |
|---|---|
| `git diff --check upstream/devel...HEAD` | 통과 |
| `ruby -e "require 'yaml'; YAML.load_file('.github/workflows/close-issues-on-devel-push.yml')"` | 통과 |
| `git merge --no-commit --no-ff upstream/devel` | `Already up to date` |
| closing keyword 경계 사례 Node 재현 | 통과 |

경계 사례 재현 결과:

| 입력 | 수집 결과 |
|---|---|
| `Closes #1870` | `1870` |
| `Fixes #1, related follow-up #2 remains open` | `1` |
| `Resolves #3; follow-up issue #4 tracks remaining work` | `3` |
| `Related #5 only` | 없음 |
| `Fixes edwardkim/rhwp#6 and external/repo#7` | `6` |
| `Resolves #10, resolves #123, resolves octo-org/octo-repo#100` | `10`, `123` |
| `Closes: https://github.com/edwardkim/rhwp/issues/11` | `11` |

`octo-org/octo-repo#100`은 다른 저장소 reference라 close 대상에서 제외되는 것이 맞다.

## GitHub Actions

문서 작성 시점 기준 최신 head `8413e28a6eceaefd6f159c321eb74373677baef8`에서 확인한 check 상태:

| Check | 결과 |
|---|---|
| CI preflight | success |
| Build & Test | success |
| WASM Build | skipped |
| CodeQL preflight | success |
| Analyze (javascript-typescript) | success |
| Analyze (python) | success |
| Analyze (rust) | success |
| CodeQL | success |

리뷰 문서 push 후에는 head SHA가 바뀌므로, latest head 기준 check 상태를 다시 확인해야 한다.

## 위험 요소와 판단

핵심 위험은 자동 close 대상의 과잉 수집이었다. contributor의 두 번째 커밋으로 closing keyword와 직접 연결된
issue reference만 수집하도록 좁혀져, 최초 지적한 과잉 close 문제는 해소됐다.

이 workflow는 `devel` push에서만 실행되고 `contents: read`, `issues: write`, `pull-requests: read` 권한만
요청한다. 같은 저장소 issue만 닫도록 owner/repo 비교도 수행한다.

남은 주의점은 GitHub native auto-close와 달리 이 workflow가 merge 후 comment를 남기고 issue를 닫는 별도
자동화라는 점이다. PR #1871이 merge된 뒤 Issue #1870이 자동 close되는지 확인해야 하며, 동작하지 않으면
workflow run 로그와 issue state를 확인한 뒤 수동 close 여부를 작업지시자에게 다시 승인받아야 한다.

## 최종 권고

PR #1871은 merge 후보로 판단한다.

merge 전 조건:

- 최신 PR head 기준 GitHub Actions relevant checks 통과 또는 문서-only 후속 커밋 fast-pass 조건 확인
- PR diff에 `mydocs/pr/archives/pr_1871_review.md`와 `mydocs/pr/archives/pr_1871_report.md` 포함 확인
- 최신 `mergeable` / `mergeStateStatus` 재확인
- latest head 기준 approval 상태 확인
- 작업지시자 merge 승인
