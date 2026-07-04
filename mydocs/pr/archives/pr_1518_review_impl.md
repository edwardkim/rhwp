# PR #1518 처리 계획 — co-anchored float 표 배치와 PDF 정합 보정

- 작성일: 2026-06-25
- PR: <https://github.com/edwardkim/rhwp/pull/1518>
- 관련 이슈: #1510
- 처리 경로: collaborator self-merge 후보 예외 경로
- 문서 위치: `mydocs/pr/archives/pr_1518_review.md`, `mydocs/pr/archives/pr_1518_review_impl.md`

## 1. 커밋 목록

문서 작성 직전 PR head 기준 커밋:

| SHA | 제목 |
|-----|------|
| `fd6ca17a` | `task 1510: co-anchored float 표 순서 보정` |
| `6b61d683` | `task 1510: visible float 표 시각 정합 보정` |
| `06c45988` | `task 1510: HWPX visible float 페이지네이션 보정` |
| `91341137` | `task 1510: PDF 단위와 본문 피치 보정` |

이 문서와 오늘할일 기록을 추가한 뒤 PR head SHA는 다시 바뀐다. merge 전 최신 SHA와 GitHub Actions 상태를
재확인한다.

## 2. Stage 구성

### Stage A — PR 문서 동반 커밋

- `mydocs/pr/archives/pr_1518_review.md` 작성
- `mydocs/pr/archives/pr_1518_review_impl.md` 작성
- `mydocs/orders/20260625.md`에 #1518 처리 대기 항목 추가
- 문서 전용 변경이므로 `git diff --check`와 변경 범위 확인
- `docs: PR #1518 검토 기록` 커밋 후 `upstream/task_m100_1510`에 push

상태: 완료. 이 문서와 오늘할일을 포함한 docs 커밋 push 후 최신 head 기준 CI를 다시 확인한다.

### Stage B — CI 완료 대기

- `gh pr checks 1518 --repo edwardkim/rhwp --watch` 또는 주기적 확인
- 최종 조건:
  - `Build & Test` 성공
  - `Canvas visual diff` 성공
  - CodeQL Analyze 계열 성공
  - skip이 의도된 `WASM Build`는 skipped 허용
- 실패 시 로그 확인 후 별도 stage 문서 작성, 수정, 재검증

상태: 대기.

### Stage C — Merge

merge 직전 확인:

```bash
gh pr view 1518 --repo edwardkim/rhwp --json mergeable,mergeStateStatus,headRefOid,statusCheckRollup,closingIssuesReferences
```

조건 충족 시:

```bash
gh pr merge 1518 --repo edwardkim/rhwp --merge --admin
```

실행 조건:

- PR head 최신 커밋 기준 GitHub Actions 통과.
- review 문서와 오늘할일 기록이 PR diff에 포함됨.
- `mergeStateStatus`가 `CLEAN` 또는 작업지시자가 허용한 상태.
- 작업지시자 명시 승인.

상태: 대기.

## 3. Merge 후 후속 처리

### 3.1 Issue close 확인

PR 본문에는 `Closes #1510`이 포함되어 있지만, 문서 작성 시점 `closingIssuesReferences`는 비어 있다.
merge 후 반드시 issue 상태를 확인한다.

```bash
gh issue view 1510 --repo edwardkim/rhwp --json state,closedAt
```

자동 close가 실패하면 수동 close:

```bash
gh issue close 1510 --repo edwardkim/rhwp --comment-file /tmp/issue1510_close_comment.md
```

close 코멘트 초안:

```text
PR #1518 머지로 #1510 재현 조건을 기준으로 보정했습니다.

이번 검증은 원본 내부 양식이 없어, 이슈 본문 조건에 맞춘 임시 합성 HWP/HWPX 파일을 만들고 한컴 2024에서 PDF로 저장한 기준 파일과 rhwp export-pdf 결과를 비교하는 방식으로 진행했습니다. 해당 샘플과 PDF 기준 자료는 PR에 포함했습니다.

아직 실제 문서에서 부족한 부분이 남아 있으면, 가능하다면 재현 가능한 HWP/HWPX 샘플 파일을 zip으로 묶어 첨부해 주세요. 내부 양식 공유가 어렵다면 민감한 내용을 제거한 최소 재현 파일도 괜찮습니다.
```

### 3.2 devel sync

```bash
git fetch upstream
git checkout local/devel
git rebase upstream/devel
```

### 3.3 렌더 영향 후속 확인

이 PR은 렌더링과 PDF export에 영향을 준다. merge 후 다음을 확인한다.

```bash
cargo test --test svg_snapshot
```

실패 시 의도된 렌더 변경인지 확인하고, 필요할 때만 golden 재생성 절차를 따른다.

### 3.4 작업 브랜치 정리

```bash
git push upstream --delete task_m100_1510
git branch -D task_m100_1510
git fetch upstream --prune
```

삭제 후 확인:

```bash
git branch --list 'task_m100_1510'
git branch -r | rg 'task_m100_1510' || true
git ls-remote --heads upstream task_m100_1510
```

### 3.5 오늘할일 갱신

`mydocs/orders/20260625.md`에 다음을 반영한다.

- merge SHA
- #1510 close 여부
- CI 최종 상태
- `svg_snapshot` 후속 확인 결과
- HWPX 표 셀 2행 y 잔차 후속 분리 여부

## 4. 작업지시자 확인 사항

- PR #1518은 Open PR로 생성되어 있고 Draft가 아니다.
- 작업지시자가 PR review 문서와 오늘할일 문서 추가 및 remote push를 지시했다.
- merge는 GitHub Actions 최신 통과 상태와 작업지시자 승인 뒤 수행한다.
- HWPX 1쪽 표 셀 y 잔차는 현재 PR의 merge 차단 사유가 아니라 후속 내부 row/baseline 보정 대상으로 기록한다.
