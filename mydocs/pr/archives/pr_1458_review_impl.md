# PR #1458 self-merge 실행 계획

- PR: https://github.com/edwardkim/rhwp/pull/1458
- 작성일: 2026-06-21
- 경로: collaborator self-merge 후보
- 작성 시점 head: `ce442a0069ea24db807d25a5341e76022fa342e2`

## 1. 목적

PR #1458은 collaborator가 작성한 본인 PR이다.
작업지시자가 PR 생성, 오늘할일 갱신, review 문서 push, CI 완료 후 merge와 후속 코멘트 처리를 지시했으므로,
`mydocs/manual/pr_review_workflow.md`의 collaborator self-merge 후보 예외 경로에 따라 review 운영 문서를 PR head에 포함하고 최신 GitHub Actions 통과 후 merge한다.

## 2. 커밋 목록

작성 시점 PR 커밋:

| 순서 | SHA | 제목 |
|------|-----|------|
| 1 | `3c74b619` | `task 1452: 그림 삽입과 Shift+Tab 내어쓰기 개선` |
| 2 | `42f50cdf` | `task 1452: PNG 알파 BinData 보존 검증 추가` |
| 3 | `496dca72` | `task 1452: 개체 속성 창 크기 고정` |
| 4 | `f891d8de` | `task 1452: 그림 전체 투명도 구현` |
| 5 | `82f817b1` | `task 1452: 외부 그림 드롭 한컴 동작 정합 보정` |
| 6 | `fd571e9b` | `task 1452: 투명도 그림 렌더링 중복 보정` |
| 7 | `59312f67` | `task 1452: TAC 그림 Enter 줄 유지 보정` |
| 8 | `cdbcb4cf` | `task 1452: 파일 열기 취소 재오픈 방지` |
| 9 | `448853ae` | `task 1452: 저장 커서와 문단부호 표시 정합 개선` |
| 10 | `dddd4a57` | `task 1452: 그림 기준 커서 위치 보정` |
| 11 | `4499c88e` | `task 1452: TAC 그림 문단 분할 보정` |
| 12 | `ee637e33` | `task 1452: 줄 끝 커서 이동 affinity 보정` |
| 13 | `ce442a00` | `task 1452: 커서 회귀와 클립보드 복사 보정` |

본 실행 계획 커밋:

- `mydocs/pr/archives/pr_1458_review.md`
- `mydocs/pr/archives/pr_1458_review_impl.md`

## 3. 진행 단계

### Stage A — review 문서 추가

1. `mydocs/pr/archives/pr_1458_review.md` 작성
2. `mydocs/pr/archives/pr_1458_review_impl.md` 작성
3. 오늘할일 문서에 PR #1458 처리 시작 기록
4. `git diff --check`
5. 문서 커밋 작성
6. `upstream`의 `task_m100_1452`로 push

### Stage B — 최신 GitHub Actions 재확인

push 후 PR head가 바뀌므로 다음 상태를 최신 head 기준으로 다시 확인한다.

- PR 상태: open, draft 아님
- mergeable: `MERGEABLE` 또는 merge 가능한 상태
- Build & Test: pass
- Canvas visual diff: pass
- CodeQL: pass
- Analyze (javascript-typescript): pass
- Analyze (python): pass
- Analyze (rust): pass

로컬 PR 검증은 이미 수행했으므로 이 단계에서 반복하지 않는다.

### Stage C — merge

최신 GitHub Actions 통과와 작업지시자 승인 확인 후 merge한다.

우선 GitHub 일반 merge를 사용한다.

```bash
gh pr merge 1458 --repo edwardkim/rhwp --merge
```

branch protection 또는 권한 문제로 일반 merge가 막히는 경우에만 작업지시자에게 확인 후 admin merge를 검토한다.

## 4. merge 전 체크리스트

- [ ] review 문서 2건이 PR diff에 포함됨
- [ ] 오늘할일 문서가 PR diff에 포함됨
- [ ] 최신 PR head SHA 확인
- [ ] 최신 GitHub Actions 통과 확인
- [ ] merge 가능한 상태 확인
- [ ] 작업지시자 승인 확인
- [ ] `Closes #1452` 자동 close 연결 확인

## 5. merge 후 후속 처리

- PR merge 결과 확인
- `devel` 반영 여부 확인
- 이슈 #1452 close 여부 확인
- 자동 close가 실패하면 PR #1458 머지 결과를 근거로 수동 close와 감사 코멘트를 남긴다.
- PR에 검증 결과와 merge 완료 코멘트를 남긴다.
- `upstream/devel` 기준 로컬 동기화와 작업 브랜치 정리를 진행한다.
