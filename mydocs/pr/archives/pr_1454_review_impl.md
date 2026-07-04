# PR #1454 self-merge 실행 계획

- PR: https://github.com/edwardkim/rhwp/pull/1454
- 작성일: 2026-06-21
- 경로: collaborator self-merge 후보
- 작성 시점 head: `00f462dbf3dbe8da5dfb225267519616315b42b9`

## 1. 목적

PR #1454는 collaborator가 작성한 본인 PR이다.
작업지시자가 직접 merge 진행을 요청했으므로, #1425에서 정리한 collaborator self-merge 후보 규칙에 따라
review 운영 문서를 PR head에 포함하고 최신 GitHub Actions 통과 후 merge한다.

## 2. 커밋 목록

작성 시점 PR 커밋:

| 순서 | SHA | 제목 |
|------|-----|------|
| 1 | `5209dab8` | `Task #1328: Add local font consent plans` |
| 2 | `3f9595c0` | `Task #1328: Stage 1 local font state model` |
| 3 | `bbf55cce` | `Task #1328: Stage 2 local font consent modal` |
| 4 | `ef828502` | `Task #1328: Stage 3 gate display font chains` |
| 5 | `27687f84` | `Task #1328: Stage 4 refresh local font UI` |
| 6 | `b63428ac` | `Task #1328: Stage 5 finalize local font settings` |
| 7 | `00f462db` | `Task #1328: Add sample fixture and final report` |

본 실행 계획 커밋:

- `mydocs/pr/archives/pr_1454_review.md`
- `mydocs/pr/archives/pr_1454_review_impl.md`

## 3. 진행 단계

### Stage A — review 문서 추가

1. `mydocs/pr/archives/pr_1454_review.md` 작성
2. `mydocs/pr/archives/pr_1454_review_impl.md` 작성
3. `git diff --check`
4. 문서 커밋 작성
5. `postmelee:1328-local-font-consent`로 push

### Stage B — 최신 GitHub Actions 재확인

push 후 PR head가 바뀌므로 다음 상태를 최신 head 기준으로 다시 확인한다.

- PR 상태: open, draft 아님
- mergeable: `MERGEABLE` / `CLEAN`
- Build & Test: pass
- Canvas visual diff: pass
- CodeQL: pass
- Analyze (javascript-typescript): pass
- Analyze (python): pass
- Analyze (rust): pass

### Stage C — merge

최신 GitHub Actions 통과와 작업지시자 승인 확인 후 merge한다.

우선 GitHub 일반 merge를 사용한다.

```bash
gh pr merge 1454 --repo edwardkim/rhwp --merge
```

branch protection 또는 권한 문제로 일반 merge가 막히는 경우에만 작업지시자에게 확인 후 admin merge를 검토한다.

## 4. merge 전 체크리스트

- [ ] review 문서 2건이 PR diff에 포함됨
- [ ] 최신 PR head SHA 확인
- [ ] 최신 GitHub Actions 통과 확인
- [ ] `mergeStateStatus == CLEAN`
- [ ] 작업지시자 승인 확인
- [ ] 이슈 #1328 자동 close가 걸려 있지 않음을 확인

## 5. merge 후 후속 처리

- PR merge 결과 확인
- `devel` 반영 여부 확인
- 이슈 #1328은 `Refs`만 연결되어 있으므로 자동 close되지 않는다.
- 이슈 close는 별도 작업지시자 승인 후 수동 처리한다.
