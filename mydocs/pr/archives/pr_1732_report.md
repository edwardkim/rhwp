# PR #1732 사전 판단 보고서

## 개요

- PR: #1732 `Task #1664: 최종 measurement 기록을 mydocs에 반영`
- 작성자: @postmelee
- 경로: collaborator self-merge 후보
- base: `devel`
- head: `postmelee:task1664-docs-final-measurement`
- 검토 대상 원 변경 커밋: `70c1e8b42406e7c792b5873ec35cc8392261f7cf`
- 관련 이슈: #1664, #1668

이 문서는 merge 완료 후 사후 보고서가 아니라, merge 전 수용 판단과 후속 조건을 기록하는 사전 판단
보고서다. 실제 merge commit, merge 시각, issue close 결과는 merge 후 GitHub metadata를 원천 기록으로
확인한다.

## 권고

수용 권고.

근거:

- 변경 범위가 `mydocs/**` 문서로만 제한된다.
- #1702 merge 후 cleanup, trusted branch save, 후속 exact-hit 관측값이 #1664/#1668 이슈 코멘트와 일치한다.
- `.github/workflows/ci.yml`, `Cargo.toml`, `tests/`, fixture, golden 변경이 없다.
- `git diff --check`가 통과했다.
- 문서 전용 PR fast-pass 조건에 맞는 형태다.

## 변경 의도와 파일별 역할

근본 목적은 #1702 merge 후 최종 measurement가 GitHub issue comment에만 남아 있는 상태를 해소하고,
장기 보관 문서에 같은 기준으로 보존하는 것이다.

변경 파일:

- `mydocs/report/task_m100_1664_measurement.md`
  - cache cleanup, trusted branch save success, 후속 exact-hit restore/save skipped 기록
- `mydocs/report/task_m100_1668_ci_pipeline_tracking.md`
  - #1664 완료 기준선과 #1666/#1667 후속 판단 연결
- `mydocs/report/task_m100_1664_report.md`
  - "merge 후 확인 필요"였던 항목을 실제 관측 결과로 보정
- `mydocs/plans/task_m100_1664_v2.md`
  - 최종 measurement 반영 문서 PR 수행 계획 기록
- `mydocs/orders/20260701.md`
  - 당일 후속 문서 PR 작업 기록

## 검증 요약

로컬 및 원격 확인:

- `gh pr view 1732 --repo edwardkim/rhwp --json ...`로 최신 metadata 확인
- `git fetch upstream pull/1732/head:local/pr1732`
- `git diff --stat upstream/devel...local/pr1732`
- `git diff --name-status upstream/devel...local/pr1732`
- `git diff --check upstream/devel...local/pr1732`
- `git rev-list --parents -n 1 local/pr1732`
- #1664 issue comment `4853263686` 원문 대조
- #1668 issue comment `4853268643` 원문 대조
- `gh run view 28505355210 --attempt 2` 대조
- `gh run view 28507949075` 대조

결과:

- PR head 원 변경 커밋은 single-parent commit이다.
- 변경 파일은 `mydocs/**` 5개뿐이다.
- whitespace 문제는 없다.
- run head, job 시간, save 상태, cache size가 문서 기록과 일치한다.
- 시각/렌더 검증은 필요하지 않다.

## PR head 문서 커밋 계획

리뷰 문서 2건을 별도 문서 커밋으로 PR head에 추가한다.

대상 파일:

- `mydocs/pr/archives/pr_1732_review.md`
- `mydocs/pr/archives/pr_1732_report.md`

push 대상:

- repository: `postmelee/rhwp`
- branch: `task1664-docs-final-measurement`

push 후 확인:

- PR head SHA가 문서 커밋으로 변경됐는지 확인
- PR diff에 위 review/report 문서 2건이 포함됐는지 확인
- 추가 변경 파일이 `mydocs/pr/archives/` 문서 2건으로 제한됐는지 확인
- GitHub Actions가 문서 전용 fast-pass 또는 required checks 통과 상태인지 확인

## route 결정

Route: collaborator self-merge 후보.

- 외부 contributor PR이 아니므로 collaborator-mediated 외부 PR section 9 경로가 아니다.
- stacked/conflicting PR이 아니므로 cherry-pick integration PR 경로도 아니다.
- 원 PR #1732를 그대로 merge 후보로 유지한다.

## source commit / credit

원 PR source commit:

- `70c1e8b42406e7c792b5873ec35cc8392261f7cf` — `docs: record #1664 final cache measurements`

작성자:

- original author: @postmelee
- co-author trailer: 없음

cherry-pick integration이 아니므로 source commit to integration commit mapping은 해당 없다.

## merge 전 조건

merge 전 다음을 다시 확인한다.

- 최신 PR head SHA
- `mergeable` / `mergeStateStatus`
- 최신 GitHub Actions 상태
- PR diff에 review/report 문서 포함 여부
- 변경 범위가 `mydocs/**`로만 제한되는지
- 작업지시자 merge 승인

## issue close 계획

PR #1732는 PR body에서 이슈 close를 하지 않는다고 명시한 문서 전용 후속 PR이다.

- #1668: tracking/RFC 이슈이므로 이 PR merge만으로 close하지 않는다.
- #1664: close 여부는 #1732 merge 후에도 작업지시자 승인으로 별도 판단한다.
- auto-close 기대 없음: PR description은 `Refs #1664, #1668`이고 `Closes` 문구가 없다.
