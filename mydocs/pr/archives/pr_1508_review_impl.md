# PR #1508 처리 계획 — HWPX 분할 표와 경고 보정

- 작성일: 2026-06-24
- PR: https://github.com/edwardkim/rhwp/pull/1508
- 관련 이슈: #1486
- 처리 경로: collaborator self-merge 후보 예외 경로
- 문서 위치: `mydocs/pr/archives/pr_1508_review.md`, `mydocs/pr/archives/pr_1508_review_impl.md`

## 1. 커밋 목록

문서 작성 직전 PR head 기준 커밋:

| SHA | 제목 |
|-----|------|
| `9dbd32cc` | `task 1486: HWPX 분할 표 TAC 배치 1차 보정` |
| `f27d39cb` | `task 1486: RowBreak 잔여 조각 페이지 보정` |
| `ee17cb39` | `task 1486: RowBreak 병합행 분할 순서 보정` |
| `ab6c9f3c` | `task 1486: 13쪽 footer와 19쪽 그림 잘림 보정` |
| `a2a7d848` | `task 1486: 마지막 쪽 TAC 그림과 RowBreak 회귀 보정` |
| `515da5d1` | `task 1486: HWPX 경고 보정 상태 초기화` |
| `cb2596af` | `task 1486: RowBreak rowspan hard-break 컷 보정` |

이 문서와 오늘할일 기록을 추가한 뒤 PR head SHA는 다시 바뀐다. merge 전 최신 SHA와 GitHub Actions 상태를
재확인한다.

## 2. Stage 구성

### Stage A — PR 문서 동반 커밋

- `mydocs/pr/archives/pr_1508_review.md` 작성
- `mydocs/pr/archives/pr_1508_review_impl.md` 작성
- `mydocs/orders/20260624.md`에 #1508 처리 대기 항목 추가
- 문서 전용 변경이므로 `git diff --check`와 변경 범위 확인
- `docs: PR #1508 검토 기록` 커밋 후 `upstream/task_m100_1486`에 push

### Stage B — CI 완료 대기

- `gh pr checks 1508 --repo edwardkim/rhwp --watch` 또는 주기적 확인
- 최종 조건:
  - `Build & Test` 성공
  - `Canvas visual diff` 성공
  - CodeQL Analyze 계열 성공
  - skip이 의도된 `WASM Build`는 skipped 허용
- 실패 시 로그 확인 후 별도 stage 문서 작성, 수정, 재검증

### Stage C — Merge

merge 직전 확인:

```bash
gh pr view 1508 --repo edwardkim/rhwp --json mergeable,mergeStateStatus,headRefOid,statusCheckRollup
```

조건 충족 시:

```bash
gh pr merge 1508 --repo edwardkim/rhwp --merge --admin
```

## 3. Merge 후 후속 처리

### 3.1 Issue close 확인

```bash
gh issue view 1486 --repo edwardkim/rhwp --json state,closedAt
```

자동 close가 실패하면 수동 close:

```bash
gh issue close 1486 --repo edwardkim/rhwp --comment "PR #1508 머지로 해결했습니다."
```

### 3.2 devel sync

```bash
git fetch upstream
git checkout local/devel
git rebase upstream/devel
```

### 3.3 렌더 영향 후속 확인

```bash
cargo test --test svg_snapshot
```

실패 시 의도된 렌더 변경인지 확인하고, 필요할 때만 golden 재생성 절차를 따른다.

### 3.4 작업 브랜치 정리

```bash
git push upstream --delete task_m100_1486
git branch -D local/task_m100_1486
git fetch upstream --prune
```

### 3.5 오늘할일 갱신

`mydocs/orders/20260624.md`에 다음을 반영한다.

- merge SHA
- #1486 close 여부
- CI 최종 상태
- 후속 확인 결과

## 4. 작업지시자 확인 사항

- PR #1508은 Open PR로 생성되어 있고 Draft가 아니다.
- 작업지시자가 PR review 문서와 오늘할일 문서 추가 및 remote push를 승인했다.
- 작업지시자가 CI 완료 대기 후 merge와 후처리를 지시했다.
- merge는 GitHub Actions 최신 통과 상태 확인 뒤 수행한다.
