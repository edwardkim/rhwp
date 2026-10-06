# Task M100 #1547 Stage 4 완료보고서 — PR 리뷰 운영 문서 보강

- 이슈: #1547
- 브랜치: `local/task1547`
- 작성일: 2026-06-26
- 단계: Stage 4 — 운영 문서 보강

## 1. 변경 요약

`mydocs/manual/pr_review_workflow.md` 의 9장 collaborator-mediated 외부 PR 처리 경로에 review 문서 전용
후속 커밋 fast-pass 규칙을 추가했다.

변경 위치:

- 9.3 PR head push 규칙
- 신규 9.3.1 리뷰 문서 전용 후속 커밋 fast-pass
- 9.4 merge 전 최종 조건

## 2. 문서화한 규칙

### 2.1 PR head push 후 확인 조건

기존 문구는 "문서 커밋 push 후 GitHub Actions 재실행을 기다린다"였다. 이를 다음 의미로 보강했다.

- full CI 재실행 또는 fast-pass 결과를 확인한다.
- 결과는 merge 가능 상태여야 한다.

### 2.2 fast-pass 적용 조건

fast-pass 는 다음 조건을 모두 만족할 때만 적용될 수 있다고 명시했다.

- PR head 의 뒤쪽 후속 커밋들이 `mydocs/pr/**` 또는 `mydocs/orders/*.md` 만 변경한다.
- 해당 후속 커밋들은 single-parent commit 이다.
- 후속 문서 커밋을 제외한 직전 코드 검증 대상 SHA 에 기존 GitHub Actions check-run 이 존재한다.
- 직전 코드 검증 대상 SHA 의 relevant check 가 `success`, `skipped`, `neutral` 중 하나다.

### 2.3 fast-pass 비대상

다음 변경은 반드시 최신 PR head 기준 heavy CI 를 다시 기다리도록 명시했다.

- 코드, 테스트, workflow 파일 변경
- 샘플, baseline, golden, 렌더링 fixture 변경
- `docs/**`, `mydocs/plans/**`, `mydocs/report/**`, `mydocs/working/**` 등 review 기록 범위를 벗어난 문서 변경
- check-run 조회 실패, missing check, failed check, merge commit 형태의 문서 후속 커밋

### 2.4 merge 전 최종 조건

merge 전 최종 조건을 다음처럼 확장했다.

- PR head 최신 커밋 기준 GitHub Actions 통과
- 또는 review 문서 전용 후속 커밋 fast-pass 결과 확인

다만 branch protection 이 pending/failing 상태이면 merge 하지 않는다는 안전 조건도 함께 남겼다.

## 3. 의도

이번 문서 보강은 merge 조건을 완화하기 위한 것이 아니다. 이미 검증된 직전 코드 SHA 의 CI 결과를
review 문서 전용 후속 커밋에서 재사용할 수 있음을 운영 규칙으로 명확히 한 것이다.

따라서 collaborator-mediated 외부 PR 경로는 계속 다음 원칙을 유지한다.

- review 문서는 PR diff 에 포함한다.
- 작업지시자 승인 전에는 merge 판단을 완료하지 않는다.
- 코드나 테스트가 바뀌면 최신 PR head 기준 heavy CI 를 다시 통과해야 한다.

## 4. 검증 결과

| 명령 | 결과 |
|---|---|
| `rg -n "fast-pass|리뷰 문서 전용|collaborator-mediated|GitHub Actions|merge 가능 상태" mydocs/manual/pr_review_workflow.md` | 핵심 문구 위치 확인 |
| `git diff --check -- mydocs/manual/pr_review_workflow.md` | 통과 |

## 5. 남은 작업

Stage 5 에서는 전체 변경 범위를 확인하고, workflow 정적 검증과 최종 보고서를 작성한다. 가능하면 실제 PR 에서
workflow 결과도 확인한다.
