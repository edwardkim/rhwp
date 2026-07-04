# PR #1871 사전 처리 판단 보고서

## 개요

| 항목 | 내용 |
|---|---|
| PR | #1871 |
| 제목 | `task 1870: devel merge 이슈 자동 close 보정` |
| 작성자 | `jangster77` |
| 관련 이슈 | #1870 |
| 문서 작성일 | 2026-07-04 |
| 문서 작성 시점 head | `8413e28a6eceaefd6f159c321eb74373677baef8` |
| route decision | 원 PR merge 후보 |
| 권고 | merge 수용 |

이 문서는 merge 전 사전 판단 보고서다. 아직 merge 완료, merge commit, 이슈 close 완료를 확정 사실로
기록하지 않는다.

## 변경 요약

PR #1871은 `devel` merge 후 GitHub native closing keyword가 issue를 자동 close하지 못하는 문제를 보정하기
위해 `.github/workflows/close-issues-on-devel-push.yml`을 추가한다.

workflow는 `push` to `devel` 이벤트에서 실행되며, pushed commit과 associated PR의 title/body/commit message를
조회해 같은 저장소 issue에 대한 closing keyword reference를 수집한다. open issue가 있으면 comment를 남긴 뒤
`state_reason: completed`로 close한다.

## 원인과 해결 방향

현재 저장소 default branch는 `main`이고 운영 merge 기준은 `devel`이다. GitHub native closing keyword는
default branch 기준으로 동작하므로, `devel` 대상 PR이 merge되어도 PR description의 `Closes #N`이 issue
auto-close로 연결되지 않는 사례가 있었다.

PR #1871은 GitHub native auto-close를 대체하는 것이 아니라, `devel` push 이후 같은 저장소 issue에 한정해
closing keyword를 보정 처리하는 workflow를 추가한다.

## 리뷰 중 발견된 문제와 수정 결과

1차 head `3ed3ed5f51ed00e117ede6daad4a754d83a1a8c5`에서는 closing keyword 뒤 같은 줄에 있는 모든 issue
reference를 수집하는 문제가 있었다. 이 경우 `Fixes #1, related follow-up #2 remains open` 같은 문장에서
`#2`까지 자동 close될 수 있어 Request changes로 재작업을 요청했다.

최신 head `8413e28a6eceaefd6f159c321eb74373677baef8`에서는 `closingIssueRef` 패턴으로 closing keyword와
직접 연결된 issue reference만 수집한다. 로컬 재현 결과 최초 지적한 과잉 close 사례는 해소됐다.

## 검증 결과

로컬 검증:

- `git diff --check upstream/devel...HEAD`: 통과
- Ruby YAML parse: 통과
- `upstream/devel` merge 시뮬레이션: 충돌 없음 (`Already up to date`)
- Node 기반 closing keyword 경계 사례 재현: 통과

GitHub Actions:

- `Build & Test`: success
- `CI preflight`: success
- `CodeQL preflight`: success
- `Analyze (javascript-typescript)`: success
- `Analyze (python)`: success
- `Analyze (rust)`: success
- `CodeQL`: success
- `WASM Build`: skipped

리뷰 문서 push 후에는 새 docs-only head가 생기므로, 해당 head의 check 상태와 approval 상태를 다시 확인해야 한다.

## PR-head 문서 push 계획

작업지시자 승인에 따라 다음 문서를 PR #1871 head 브랜치에 별도 docs-only 커밋으로 추가한다.

- `mydocs/pr/archives/pr_1871_review.md`
- `mydocs/pr/archives/pr_1871_report.md`

push 대상은 `upstream` remote의 `task/m100-1870-devel-auto-close` 브랜치다. contributor 코드 커밋은 rewrite하지
않고, review 문서만 별도 커밋으로 추가한다.

문서 push 후 확인할 항목:

- PR head SHA가 문서 커밋으로 갱신됐는지 확인
- PR diff에 review/report 문서 2건이 포함됐는지 확인
- 새 커밋의 변경 범위가 `mydocs/pr/archives/` 문서 2건으로 제한됐는지 확인
- 최신 PR head 기준 GitHub Actions 또는 fast-pass 상태 확인
- 기존 approval이 유지되는지, stale 처리되는지 확인

## contributor credit

원 contributor 커밋:

- `3ed3ed5f51ed00e117ede6daad4a754d83a1a8c5` - Taesup Jang `<tsjang@gmail.com>`
- `8413e28a6eceaefd6f159c321eb74373677baef8` - Taesup Jang `<tsjang@gmail.com>`

문서 커밋은 collaborator review 기록 커밋으로 분리한다. 원 contributor author identity와 PR credit은 유지한다.

## merge 전 조건

merge 전에는 다음을 다시 확인해야 한다.

- latest head SHA
- `mergeable` / `mergeStateStatus`
- latest head relevant checks
- PR diff에 review/report 문서 포함 여부
- latest head 기준 approval 상태
- 작업지시자 merge 승인

## merge 후 확인 계획

PR #1871 merge 후에는 다음을 확인한다.

1. PR metadata에서 merge commit SHA와 merged timestamp를 확인한다.
2. Issue #1870 state를 확인한다.
3. 새 workflow가 `devel` push에서 Issue #1870을 자동 close했는지 확인한다.
4. 자동 close가 실패해 Issue #1870이 여전히 open이면, workflow run 로그를 확인하고 작업지시자 승인 후 수동
   close/comment 여부를 결정한다.
5. PR에 contributor 감사 코멘트와 검증 요약을 남긴다.

Issue #1870 close는 작업지시자 승인 또는 직접 지시 없이는 수동 수행하지 않는다.
