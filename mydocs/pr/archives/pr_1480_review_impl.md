# PR #1480 self-merge 실행 계획

- PR: https://github.com/edwardkim/rhwp/pull/1480
- 작성일: 2026-06-22
- 경로: collaborator self-merge 후보
- 문서 작성 시점 참고 head: `e358b8fd602cd63f77f41107ab2e99d27977e73b`

## 1. 목적

PR #1480은 collaborator가 #1471을 해결하기 위해 준비한 PR이다.
작업지시자가 collaborator 직접 merge 케이스에 맞는 review 문서 포함 여부를 확인 요청했으므로,
`mydocs/manual/pr_review_workflow.md`의 collaborator self-merge 후보 예외 경로에 따라 review 운영 문서를 PR head에 포함한다.

이번 PR은 `rhwp-chrome` download interceptor와 관련 테스트, 수행 문서 변경에 한정된다.

## 2. 커밋 목록

문서 작성 시점 PR 커밋:

| 순서 | SHA | 제목 |
|------|-----|------|
| 1 | `2c22dbe9` | `Task #1471: Add execution and implementation plans` |
| 2 | `2febc42b` | `Task #1471 Stage 1: Replace Chrome filename interceptor` |
| 3 | `9da8535e` | `Task #1471 Stage 2: Add Chrome download observer tests` |
| 4 | `f4844a25` | `Task #1471 Stage 3: Verify Chrome download path fix` |
| 5 | `d35a1c2d` | `Task #1471: Final report and daily order update` |
| 6 | `e358b8fd` | `Merge remote-tracking branch 'upstream/devel' into local/task1471` |

본 실행 계획 커밋:

- `mydocs/pr/archives/pr_1480_review.md`
- `mydocs/pr/archives/pr_1480_review_impl.md`
- `mydocs/report/task_m100_1471_report.md`
- `mydocs/orders/20260622.md`

## 3. 진행 단계

### Stage A - review 문서 추가

1. `mydocs/pr/archives/pr_1480_review.md` 작성
2. `mydocs/pr/archives/pr_1480_review_impl.md` 작성
3. 최종 보고서와 오늘할일의 PR 준비 상태 보정
4. `git diff --check`
5. 문서 커밋 작성
6. PR head 브랜치 `task_m100_1471`로 push

### Stage B - 최신 GitHub Actions 재확인

push 후 PR head가 바뀌므로 다음 상태를 최신 head 기준으로 다시 확인한다.

- PR 상태: open, draft 아님
- merge 상태: merge 가능한 상태
- Build & Test: pass
- Analyze (javascript-typescript): pass
- Analyze (python): pass
- Analyze (rust): pass
- CodeQL: neutral 또는 pass
- WASM Build: skipped 또는 pass

로컬 검증과 사용자 수동 검증은 이미 완료했으므로, 이 단계에서는 GitHub Actions 최신 head 통과 여부를 확인한다.

### Stage C - merge

최신 GitHub Actions 통과와 작업지시자 승인 확인 후 merge한다.

우선 GitHub 일반 merge를 사용한다.

```bash
gh pr merge 1480 --repo edwardkim/rhwp --merge
```

branch protection 또는 권한 문제로 일반 merge가 막히는 경우에만 작업지시자에게 확인 후 admin merge를 검토한다.

## 4. merge 전 체크리스트

- [ ] review 문서 2건이 PR diff에 포함됨
- [ ] 수행/구현 계획서와 stage별 작업 보고서가 PR diff에 포함됨
- [ ] 최신 PR head SHA 확인
- [ ] 최신 GitHub Actions 통과 확인
- [ ] merge 가능한 상태 확인
- [ ] 작업지시자 승인 확인
- [ ] `Refs #1471`이므로 merge 후 이슈 close는 작업지시자 승인 후 별도로 처리

## 5. merge 후 후속 처리

- PR merge 결과 확인
- `devel` 반영 여부 확인
- 이슈 #1471에는 merge 결과와 검증 내용을 코멘트한다.
- 이슈 #1471 close는 작업지시자 승인 후에만 수행한다.
- `upstream/devel` 기준 로컬 동기화와 작업 브랜치 정리를 진행한다.
