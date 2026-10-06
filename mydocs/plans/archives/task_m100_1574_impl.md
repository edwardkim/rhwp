# Task M100 #1574 구현계획서 — review 문서 전용 PR fast-pass trigger 보정

- 수행계획서: `mydocs/plans/task_m100_1574.md`
- 이슈: #1574
- 브랜치: `task_m100_1574_ci_fast_pass_trigger`
- 작성일: 2026-06-26

## Stage 1 — trigger와 candidate 판정 보정

### 작업

- `.github/workflows/ci.yml`
  - `pull_request` trigger가 `mydocs/pr/**`, `mydocs/orders/*.md` 변경에서 제외되지 않도록 조정한다.
  - preflight에서 PR 전체가 허용 문서 경로만 변경한 경우 base SHA를 candidate로 삼는다.
- `.github/workflows/codeql.yml`
  - CI와 같은 trigger/candidate 원칙을 적용한다.
- `.github/workflows/render-diff.yml`
  - review 문서 전용 경로에서도 preflight가 실행되도록 `pull_request.paths`를 보강한다.
  - 순수 문서 PR의 base SHA candidate 판정을 적용한다.

### 검증

- workflow YAML parse.
- preflight JS 문법 확인.
- `git diff --check`.

### 산출물

- `mydocs/working/task_m100_1574_stage1.md`

## Stage 2 — PR 검증과 보고

### 작업

- 후속 PR을 생성해 #1574 수정 자체의 GitHub Actions 결과를 확인한다.
- #1572에 허용 경로 후속 문서 커밋을 추가해 check가 생성되는지 확인한다.
- 결과를 최종 보고서에 기록한다.

### 검증

- `gh pr checks <후속 PR>` 결과 확인.
- `gh pr view 1572 --json statusCheckRollup,mergeStateStatus` 확인.

### 산출물

- `mydocs/working/task_m100_1574_stage2.md`
- `mydocs/report/task_m100_1574_report.md`
