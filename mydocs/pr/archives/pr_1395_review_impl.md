# PR #1395 리뷰 처리 구현 계획서

## PR 커밋 3개

| # | SHA | 내용 |
|---|---|---|
| 1 | `40a33271` | task 1394: 미주 덤프 검증 인프라 분리 |
| 2 | `47c25529` | Merge branch 'devel' into task_m100_1394 |
| 3 | `0ead213a` | task 1394: clippy counter loop 수정 |

## 현재 상태

| 항목 | 값 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1395 |
| 이슈 | #1394 |
| base | `devel` |
| head | `jangster77:task_m100_1394` |
| mergeable | `MERGEABLE` |
| CI | Build & Test, CodeQL 모두 통과 |
| assignee | PR/이슈 모두 `jangster77` 지정 완료 |

## Stage 구성

### Stage 1 - 리뷰 문서 작성 - 완료

- `mydocs/pr/pr_1395_review.md` 작성
- `mydocs/pr/pr_1395_review_impl.md` 작성
- PR 메타, 변경 범위, CI, merge 시뮬레이션 결과 기록

### Stage 2 - 작업지시자 승인 대기 - 완료

머지는 별도 명시 승인이 필요하다. 2026-06-12 작업지시자가 "진행"으로 승인했다.

확인 요청 포맷:

```text
PR #1395 검토 결과 merge 준비 완료.

- base: devel
- mergeable: MERGEABLE
- CI: Build & Test / CodeQL 통과
- 충돌 시뮬레이션: 0건
- 리뷰 문서: mydocs/pr/pr_1395_review.md

merge 진행할까요?
```

### Stage 3 - 승인 시 merge - 완료

작업지시자가 명시적으로 merge를 승인한 경우에만 진행한다.

```bash
gh pr merge 1395 --repo edwardkim/rhwp --merge
```

주의:

- `gh pr merge --admin`, auto-merge 활성화, GitHub UI merge는 작업지시자의 명시 지시 없이는 수행하지 않는다.
- 현재 세션은 PR 작성자 계정 `jangster77`로 로그인되어 있으므로, merge 지시는 반드시 별도로 확인한다.

처리 결과:

- `gh pr merge 1395 --repo edwardkim/rhwp --merge`
- merge commit: `ef14397a1800aa897521f31fb62c04b2350ea21b`

### Stage 4 - 이슈 close 확인 - 완료

PR 본문에 `Closes #1394`가 포함되어 있으므로 auto-close가 기대된다.
merge 후 반드시 확인한다.

```bash
gh issue view 1394 --repo edwardkim/rhwp --json state,closedAt
```

열려 있으면 작업지시자 승인 후 수동 close한다.

처리 결과:

- #1394는 merge 직후에도 OPEN
- `gh issue close 1394 --repo edwardkim/rhwp --comment ...`로 수동 close

### Stage 5 - devel sync - 완료

merge 후 로컬 `local/devel`을 원본 `devel`에 맞춘다.

```bash
git fetch upstream
git checkout local/devel
git rebase upstream/devel
```

처리 결과:

- `local/devel`을 `upstream/devel` merge commit까지 rebase 완료

### Stage 6 - 리뷰 문서 archives 이동 - 완료

merge 완료 후 처리 기록을 archives로 이동한다.

```bash
mv mydocs/pr/pr_1395_review.md mydocs/pr/archives/
mv mydocs/pr/pr_1395_review_impl.md mydocs/pr/archives/
```

### Stage 7 - 후속 작업

- Task 1293 본 구현 PR에서 #1395의 샘플/덤프 CLI/sweep 지표를 기준으로 미주 공통 로직을 재분석한다.
- #1366 또는 #1359가 먼저 병합되면 `src/main.rs` CLI 영역 rebase 충돌 여부를 재확인한다.

## 작업지시자 확인 필요 사항

| 항목 | 제안 |
|---|---|
| merge 여부 | 승인 후 진행 |
| merge 방식 | 일반 merge |
| 후속 sync | merge 후 `local/devel` rebase |
| 이슈 #1394 | auto-close 확인, 실패 시 수동 close 승인 요청 |
| 리뷰 문서 | merge 후 archives 이동 |

## 위험 요소 요약

| 위험 | 평가 |
|---|---|
| 코드 충돌 | 낮음. 현재 merge-tree 충돌 0 |
| CI 회귀 | 낮음. GitHub Actions 통과 |
| 렌더링 회귀 | 낮음. 렌더링 본체 변경 없음 |
| 샘플 용량 증가 | 의도된 검증 fixture 추가 |
| 실제 미주 문제 미해결 | 의도된 제외 범위. 후속 Task 1293 구현에서 처리 |
