# Task M100 #1547 최종 보고서 — PR 리뷰 문서 전용 커밋 heavy CI fast-pass gate

- 이슈: #1547
- 브랜치: `local/task1547`
- 작성일: 2026-06-26

## 1. 개요

외부 contributor PR 을 collaborator 가 검토할 때, 코드 변경 CI 가 이미 통과한 뒤 review 문서만 PR head 에
추가해도 heavy CI 가 다시 실행되는 대기 문제를 줄이기 위해 GitHub Actions fast-pass preflight 를 도입했다.

이 fast-pass 는 merge 조건 완화가 아니라, trailing review 문서 전용 커밋을 제외한 직전 코드 검증 SHA 의
성공한 check-run 을 재사용하는 좁은 최적화다. 판정이 불명확하면 항상 기존 heavy CI 를 실행한다.

## 2. 변경 내용

### 2.1 CI workflow

`.github/workflows/ci.yml`:

- `CI preflight` job 추가
- PR commit 목록에서 trailing docs-only 구간 판정
- candidate SHA 의 `Build & Test` check-run 이 green 일 때만 `Build & Test` job skip

### 2.2 CodeQL workflow

`.github/workflows/codeql.yml`:

- `CodeQL preflight` job 추가
- candidate SHA 의 `Analyze (javascript-typescript)`, `Analyze (python)`, `Analyze (rust)` 가 모두 green 일
  때만 matrix `analyze` job skip

### 2.3 Render Diff workflow

`.github/workflows/render-diff.yml`:

- `Render Diff preflight` job 추가
- candidate SHA 의 `Canvas visual diff` 가 green 일 때만 `canvas-visual-diff` job skip

### 2.4 운영 문서

`mydocs/manual/pr_review_workflow.md`:

- collaborator-mediated 외부 PR 처리 경로에 `9.3.1 리뷰 문서 전용 후속 커밋 fast-pass` 절 추가
- fast-pass 적용 조건과 비대상 조건 명시
- branch protection 이 pending/failing 이면 merge 하지 않는 안전 조건 명시

## 3. fast-pass 조건

다음 조건을 모두 만족해야 fast-pass 될 수 있다.

- PR head 뒤쪽 후속 커밋이 `mydocs/pr/**` 또는 `mydocs/orders/*.md` 만 변경한다.
- 해당 후속 커밋은 single-parent commit 이다.
- 직전 코드 검증 대상 SHA 에 필요한 GitHub Actions check-run 이 존재한다.
- 필요한 check-run 이 `completed` 이고 conclusion 이 `success`, `skipped`, `neutral` 중 하나다.

다음 경우에는 fast-pass 하지 않는다.

- 코드, 테스트, workflow, 샘플, baseline, golden 변경
- `mydocs/plans/**`, `mydocs/report/**`, `mydocs/working/**` 등 review 기록 범위 밖 문서 변경
- check-run 누락, API 오류, failed check, merge commit 형태의 문서 후속 커밋

## 4. 검증 결과

| 항목 | 결과 |
|---|---|
| Workflow YAML parse | 통과 |
| `actionlint` | 통과 |
| `git diff --check upstream/devel..HEAD` | 통과 |
| PR #1544 check-run metadata 확인 | 통과 |
| PR #1541 Render Diff 부재 케이스 확인 | 통과 |

Live GitHub Actions 검증은 아직 수행하지 않았다. 브랜치 push/PR 생성 이후 확인해야 한다.

## 5. 남은 위험

- `devel` 은 protected 로 조회되지만 required check context 상세는 현재 토큰에서 확인되지 않았다.
- advanced-security `CodeQL` 체크가 branch protection required check 이면 CodeQL matrix job skip 만으로는
  PR UI 통과가 부족할 수 있다.
- 따라서 첫 PR 적용 시 GitHub Actions UI 와 branch protection 상태를 반드시 확인해야 한다.

## 6. 커밋

| 커밋 | 내용 |
|---|---|
| `fb755cd5` | 조사 계획 및 GitHub check metadata 확인 |
| `72a2537f` | CI 리뷰 문서 fast-pass preflight 추가 |
| `b248e97f` | CodeQL과 Render Diff fast-pass 확장 |
| `96aed706` | PR 리뷰 fast-pass 운영 규칙 문서화 |
| 본 커밋 | 최종 검증, 최종 보고서, 오늘할일 갱신 |

## 7. 결론

Stage 1~5 계획 범위의 구현과 정적 검증을 완료했다. 최종 merge 전에는 PR 생성 후 실제 GitHub Actions 결과와
branch protection 통과 여부를 확인해야 한다.
