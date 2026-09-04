# Task M100 #1547 Stage 3 완료보고서 — CodeQL 및 Render Diff preflight 확장

- 이슈: #1547
- 브랜치: `local/task1547`
- 작성일: 2026-06-26
- 단계: Stage 3 — CodeQL 및 Render Diff workflow 확장

## 1. 변경 요약

Stage 2 에서 `.github/workflows/ci.yml` 에 적용한 review 문서 전용 후속 커밋 fast-pass 구조를
CodeQL 과 Render Diff workflow 로 확장했다.

수정 파일:

- `.github/workflows/codeql.yml`
- `.github/workflows/render-diff.yml`

공통 동작:

- PR 이벤트가 아니면 fast-pass 하지 않는다.
- trailing commit 이 `mydocs/pr/**`, `mydocs/orders/*.md` 만 변경한 single-parent commit 인 경우에만
  docs-only 구간으로 인정한다.
- 허용 경로 밖 변경을 처음 만난 commit 을 candidate SHA 로 삼는다.
- candidate SHA 의 기존 check-run 이 모두 green 일 때만 heavy job 을 skip 한다.
- check-run 이 없거나, API 조회가 실패하거나, 파일 목록이 비정상이면 full workflow 를 실행한다.

## 2. CodeQL 변경

추가 job:

- `preflight`
  - 이름: `CodeQL preflight`
  - 권한: `contents: read`, `checks: read`, `pull-requests: read`
  - 도구: `actions/github-script@v8`

수정 job:

- `analyze`
  - `needs: preflight`
  - `if: ${{ always() && needs.preflight.outputs.fast_pass != 'true' }}`

fast-pass 조건:

candidate SHA 에 다음 3개 check-run 이 모두 있어야 한다.

| check | 요구 상태 |
|---|---|
| `Analyze (javascript-typescript)` | completed + success/skipped/neutral |
| `Analyze (python)` | completed + success/skipped/neutral |
| `Analyze (rust)` | completed + success/skipped/neutral |

`CodeQL` advanced-security check-run 은 현재 PR UI 에 표시되지만, 이 workflow 안에서 직접 생성하는 job 이름이
아니므로 fast-pass 판정의 필수 입력으로 삼지 않았다. required check 설정은 현재 권한에서 확정 조회되지
않으므로, 실제 PR에서 skipped matrix check가 branch protection을 만족하는지 Stage 5에서 확인이 필요하다.

## 3. Render Diff 변경

추가 job:

- `preflight`
  - 이름: `Render Diff preflight`
  - 권한: `contents: read`, `checks: read`, `pull-requests: read`
  - 도구: `actions/github-script@v8`

수정 job:

- `canvas-visual-diff`
  - `needs: preflight`
  - `if: ${{ always() && needs.preflight.outputs.fast_pass != 'true' }}`

fast-pass 조건:

candidate SHA 에 다음 check-run 이 있어야 한다.

| check | 요구 상태 |
|---|---|
| `Canvas visual diff` | completed + success/skipped/neutral |

Render Diff 대상 경로가 아니었던 PR 에서는 `Canvas visual diff` check-run 이 없음을 확인했다. 이 경우
fast-pass 하지 않고 full Render Diff job 으로 떨어진다.

## 4. 실제 PR metadata 확인

PR #1544 head SHA `bff0a603942c300f5a08fb1cd056411c02f2e7cd`:

| check | status | conclusion |
|---|---|---|
| `Analyze (javascript-typescript)` | completed | success |
| `Analyze (python)` | completed | success |
| `Analyze (rust)` | completed | success |
| `Canvas visual diff` | completed | success |

PR #1541 head SHA `287ac2f76b5e401dc95718fc900acacf29905b85`:

| check | status | conclusion |
|---|---|---|
| `Analyze (javascript-typescript)` | completed | success |
| `Analyze (python)` | completed | success |
| `Analyze (rust)` | completed | success |
| `Canvas visual diff` | 없음 | 없음 |

따라서 CodeQL 은 3개 matrix check 를 기준으로 판정할 수 있고, Render Diff 는 `Canvas visual diff` 부재 시
보수적으로 full job 을 실행해야 한다.

## 5. branch protection 조회 결과

추가로 branch protection/ruleset required check 조회를 시도했다.

- `GET /repos/edwardkim/rhwp/branches/devel/protection/required_status_checks`: 404
- GraphQL `branchProtectionRules`: 빈 배열
- REST `repos/edwardkim/rhwp/rulesets`: 빈 배열

`devel` 자체는 protected 로 조회되지만, 현재 토큰에서 required check context 목록은 확인되지 않았다.
따라서 현재 구현은 기존 check 이름을 유지하고 job-level skipped 결론이 통과로 인정된다는 GitHub 동작에
기대는 방식이다. 실제 PR에서 branch protection UI 통과 여부는 Stage 5 검증 항목으로 남긴다.

## 6. 검증 결과

| 명령 | 결과 |
|---|---|
| `ruby -e 'require "yaml"; ...'` | `.github/workflows/ci.yml`, `codeql.yml`, `render-diff.yml` 파싱 통과 |
| `actionlint .github/workflows/ci.yml .github/workflows/codeql.yml .github/workflows/render-diff.yml` | 통과 |
| `git diff --check -- .github/workflows/ci.yml .github/workflows/codeql.yml .github/workflows/render-diff.yml` | 통과 |
| PR #1544 check-runs 조회 | CodeQL 3개 + Canvas visual diff green 확인 |
| PR #1541 check-runs 조회 | CodeQL 3개 green, Canvas visual diff 없음 확인 |

## 7. 주의점

- workflow 별 preflight 스크립트 중복은 의도적으로 허용했다. PR head checkout 없이 base repository
  workflow 파일 안에서만 실행하기 위한 선택이다.
- advanced-security `CodeQL` check-run 이 required 인 저장소 설정이면 CodeQL fast-pass 가 branch protection 을
  만족하지 못할 수 있다. 현재 권한으로 required context 를 확인할 수 없었으므로 Stage 5에서 실제 PR로
  확인하거나 maintainer/admin 이 설정을 확인해야 한다.
- 관련 없는 untracked 파일 `mydocs/pr/pr_1530_review.md` 가 작업트리에 있었지만 이번 Stage 변경에는 포함하지
  않았다.

## 8. 다음 단계

Stage 4 에서는 `mydocs/manual/pr_review_workflow.md` 의 collaborator-mediated 외부 PR 처리 경로에
fast-pass 조건과 한계를 문서화한다.
