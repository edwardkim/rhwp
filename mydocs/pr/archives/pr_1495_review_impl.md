# PR #1495 처리 계획

## 커밋 목록

문서 작성 시점 PR 원 커밋:

- `fb60fba9` `fix: keep reset rewind endnote off full column`

collaborator-mediated 처리 커밋:

- 최신 `upstream/devel` merge commit: PR #1494까지 반영하여 `BEHIND` 상태 해소
- review 문서 커밋:
  - `mydocs/pr/archives/pr_1495_review.md`
  - `mydocs/pr/archives/pr_1495_review_impl.md`

## Stage 구성

### Stage A: PR head 최신화

1. `upstream/devel`을 최신화한다.
2. PR #1495 head를 별도 worktree에서 checkout한다.
3. 원 contributor 커밋을 rewrite하지 않고 `upstream/devel`을 merge commit으로 반영한다.
4. merge 충돌 여부를 확인한다.

### Stage B: review 문서 포함

1. `mydocs/pr/archives/pr_1495_review.md` 작성
2. `mydocs/pr/archives/pr_1495_review_impl.md` 작성
3. `git diff --check`
4. 문서 커밋 작성
5. contributor fork의 `snvtac/1375-endnote-rewind-column-overflow` 브랜치로 push

### Stage C: 최신 GitHub Actions 재확인

push 후 PR head가 바뀌므로 다음 상태를 최신 head 기준으로 다시 확인한다.

- PR 상태: open, draft 아님
- merge 상태: merge 가능한 상태
- Build & Test: pass
- Canvas visual diff: pass
- CodeQL: pass
- Analyze (javascript-typescript): pass
- Analyze (python): pass
- Analyze (rust): pass
- WASM Build: skipped 또는 pass

### Stage D: merge 및 후속 처리

최신 GitHub Actions 통과와 작업지시자 승인 확인 후 merge한다.

우선 일반 merge를 사용한다.

```bash
gh pr merge 1495 --repo edwardkim/rhwp --merge
```

branch protection 또는 권한 문제로 일반 merge가 막히는 경우에만 작업지시자에게 확인 후 admin merge를 검토한다.

merge 후에는 `pr_review_workflow.md` 7장 후속 처리를 따른다.

- PR merge 결과 확인
- #1375 auto-close 여부 확인
- 자동 close가 실패하면 작업지시자 승인 후 수동 close
- PR comment 또는 GitHub review에 검증 결과와 처리 사실을 과장 없이 기록
- 로컬 `devel`을 `upstream/devel`과 동기화

## merge 전 체크리스트

- [ ] review 문서 2건이 PR diff에 포함됨
- [ ] 최신 PR head SHA 확인
- [ ] 최신 GitHub Actions 통과 확인
- [ ] merge 가능한 상태 확인
- [ ] PR comment 또는 GitHub review로 검토 결과 공유
- [ ] 작업지시자 승인 확인
- [ ] `closes #1375` 자동 close 연결 확인

## 작업지시자 확인 필요 사항

- 현재 로컬 병합본 검증 결과로는 blocking 문제가 없다.
- 최종 merge 판단은 PR head 최신 커밋 기준 GitHub Actions 통과 후 수행한다.
- PR comment는 사실 중심으로 작성한다.
