# PR #1460 self-merge 실행 계획

- PR: https://github.com/edwardkim/rhwp/pull/1460
- 작성일: 2026-06-22
- 경로: collaborator self-merge 후보
- 작성 시점 head: `e80e9f534b0d1c179eeff1bb5447b359e0a3c50a`

## 1. 목적

PR #1460은 collaborator가 작성한 본인 PR이다.
작업지시자가 PR review 문서와 오늘할일 문서 준비를 지시했으므로,
`mydocs/manual/pr_review_workflow.md`의 collaborator self-merge 후보 예외 경로에 따라 review 운영 문서를 PR head에 포함하고 최신 GitHub Actions 통과 후 merge한다.

## 2. 커밋 목록

작성 시점 PR 커밋:

| 순서 | SHA | 제목 |
|------|-----|------|
| 1 | `ecaef99a` | `task 1459: 자리차지 그림 혼합 문단 렌더 보정` |
| 2 | `caa96c89` | `task 1459: TopAndBottom TAC 간격 중복 보정` |
| 3 | `672b0f8a` | `task 1459: 비TAC 그림 커서 진입 제외` |
| 4 | `e85c2589` | `task 1459: TAC 해제 그림 재흐름 보정` |
| 5 | `413b299e` | `task 1459: PR 준비 문서 정리` |
| 6 | `74b678a5` | `task 1459: 클립보드 분할 offset 보정` |
| 7 | `e80e9f53` | `task 1459: PR 검증 절차 문서 보완` |

본 실행 계획 커밋:

- `mydocs/pr/archives/pr_1460_review.md`
- `mydocs/pr/archives/pr_1460_review_impl.md`
- `mydocs/orders/20260622.md`

## 3. 진행 단계

### Stage A - review 문서 추가

1. `mydocs/pr/archives/pr_1460_review.md` 작성
2. `mydocs/pr/archives/pr_1460_review_impl.md` 작성
3. 오늘할일 문서에 PR #1460 처리 시작 기록
4. `git diff --check`
5. 문서 커밋 작성
6. `upstream`의 `task_m100_1459`로 push

### Stage B - 최신 GitHub Actions 재확인

push 후 PR head가 바뀌므로 다음 상태를 최신 head 기준으로 다시 확인한다.

- PR 상태: open, draft 아님
- merge 상태: merge 가능한 상태
- Build & Test: pass
- Canvas visual diff: pass
- CodeQL: pass
- Analyze (javascript-typescript): pass
- Analyze (python): pass
- Analyze (rust): pass

로컬 PR 검증은 이미 수행했으므로 이 단계에서 반복하지 않는다.

### Stage C - merge

최신 GitHub Actions 통과와 작업지시자 승인 확인 후 merge한다.

우선 GitHub 일반 merge를 사용한다.

```bash
gh pr merge 1460 --repo edwardkim/rhwp --merge
```

branch protection 또는 권한 문제로 일반 merge가 막히는 경우에만 작업지시자에게 확인 후 admin merge를 검토한다.

## 4. merge 전 체크리스트

- [ ] review 문서 2건이 PR diff에 포함됨
- [ ] 오늘할일 문서가 PR diff에 포함됨
- [ ] 최신 PR head SHA 확인
- [ ] 최신 GitHub Actions 통과 확인
- [ ] merge 가능한 상태 확인
- [ ] 작업지시자 승인 확인
- [ ] `Closes #1459` 자동 close 연결 확인

## 5. merge 후 후속 처리

- PR merge 결과 확인
- `devel` 반영 여부 확인
- 이슈 #1459 close 여부 확인
- 자동 close가 실패하면 PR #1460 머지 결과를 근거로 수동 close와 감사 코멘트를 남긴다.
- PR에 검증 결과와 merge 완료 코멘트를 남긴다.
- `upstream/devel` 기준 로컬 동기화와 작업 브랜치 정리를 진행한다.
