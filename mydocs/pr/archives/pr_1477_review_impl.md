# PR #1477 self-merge 실행 계획

- PR: https://github.com/edwardkim/rhwp/pull/1477
- 작성일: 2026-06-22
- 경로: collaborator self-merge 후보
- 문서 작성 시점 참고 head: `0031c92f09afb0b04e20dc62b25526d2e6a50258`

## 1. 목적

PR #1477은 collaborator가 작성한 본인 PR이다.
작업지시자가 PR review 문서와 오늘할일 문서 준비 및 remote push를 지시했으므로,
`mydocs/manual/pr_review_workflow.md`의 collaborator self-merge 후보 예외 경로에 따라 review 운영 문서를 PR head에 포함한다.

이번 PR은 `rhwp-studio` 프론트와 문서 변경에 한정된다. 작업지시자 지시에 따라 PR 준비 검증은 프론트 회귀 테스트로 제한했다.

## 2. 커밋 목록

문서 작성 시점 PR 커밋:

| 순서 | SHA | 제목 |
|------|-----|------|
| 1 | `6ea30f42` | `task 1476: 플랫폼별 메뉴 단축키 표시 보정` |
| 2 | `0031c92f` | `task 1476: PR 준비 문서 추가` |

본 실행 계획 커밋:

- `mydocs/pr/archives/pr_1477_review.md`
- `mydocs/pr/archives/pr_1477_review_impl.md`
- `mydocs/orders/20260622.md`

## 3. 진행 단계

### Stage A - review 문서 추가

1. `mydocs/pr/archives/pr_1477_review.md` 작성
2. `mydocs/pr/archives/pr_1477_review_impl.md` 작성
3. 오늘할일 문서에 PR #1477 처리 시작 기록
4. `git diff --check`
5. 문서 커밋 작성
6. `origin`의 `task_m100_1476`로 push

### Stage B - 최신 GitHub Actions 재확인

push 후 PR head가 바뀌므로 다음 상태를 최신 head 기준으로 다시 확인한다.

- PR 상태: open, draft 아님
- merge 상태: merge 가능한 상태
- Build & Test: pass
- Canvas visual diff: pass
- Analyze (javascript-typescript): pass
- Analyze (python): pass
- Analyze (rust): pass
- CodeQL: neutral 또는 pass
- WASM Build: skipped 또는 pass

로컬 프론트 회귀 검증은 이미 수행했으므로 이 단계에서 cargo/clippy 전체 검증은 반복하지 않는다.

### Stage C - merge

최신 GitHub Actions 통과와 작업지시자 승인 확인 후 merge한다.

우선 GitHub 일반 merge를 사용한다.

```bash
gh pr merge 1477 --repo edwardkim/rhwp --merge
```

branch protection 또는 권한 문제로 일반 merge가 막히는 경우에만 작업지시자에게 확인 후 admin merge를 검토한다.

## 4. merge 전 체크리스트

- [ ] review 문서 2건이 PR diff에 포함됨
- [ ] 오늘할일 문서가 PR diff에 포함됨
- [ ] 최신 PR head SHA 확인
- [ ] 최신 GitHub Actions 통과 확인
- [ ] merge 가능한 상태 확인
- [ ] 작업지시자 승인 확인
- [ ] `Closes #1476` 자동 close 연결 확인

## 5. merge 후 후속 처리

- PR merge 결과 확인
- `devel` 반영 여부 확인
- 이슈 #1476 close 여부 확인
- 자동 close가 실패하면 PR #1477 머지 결과를 근거로 수동 close와 감사 코멘트를 남긴다.
- PR에 검증 결과와 merge 완료 코멘트를 남긴다.
- `upstream/devel` 기준 로컬 동기화와 작업 브랜치 정리를 진행한다.
