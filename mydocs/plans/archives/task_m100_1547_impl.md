# Task M100 #1547 구현계획서 — PR 리뷰 문서 전용 커밋 heavy CI fast-pass gate

- 수행계획서: `mydocs/plans/task_m100_1547.md`
- 이슈: #1547
- 브랜치: `local/task1547`
- 작성일: 2026-06-26

## 1. 구현 원칙

- **보수적 fast-pass**: 판정에 실패하거나 정보가 부족하면 heavy CI 를 실행한다.
- **기존 required check 이름 보존**: branch protection 설정 변경 없이 적용할 수 있도록 기존 heavy job
  이름(`Build & Test`, `Analyze (...)`, `Canvas visual diff`)을 유지한다.
- **직전 코드 SHA 기준**: PR head 뒤쪽의 문서 전용 커밋을 걷어낸 직전 코드 변경 SHA 가 이미 green 일 때만
  heavy job 을 skip 한다.
- **허용 경로 제한**: 기본 허용 경로는 collaborator-mediated 외부 PR 문서 흐름에 필요한
  `mydocs/pr/**`, `mydocs/orders/*.md` 로 제한한다. `.github/**`, `docs/**`, `samples/**`, 코드,
  테스트, baseline 변경은 fast-pass 대상이 아니다.
- **권한 최소화**: `pull_request_target` 으로 전환하지 않는다. 기존 `pull_request` workflow 안에서
  read-only 조회만 수행한다.

## 2. Stage 1 — GitHub check metadata 조사

### 목표

실제 PR 에서 candidate SHA 의 check run/status 이름과 결론값을 확인하고, workflow 별 green 판정 기준을
확정한다.

### 작업

1. 최근 외부 PR 또는 현재 열려 있는 PR 을 대상으로 GitHub Actions check run 목록을 조회한다.
2. 다음 check 이름이 candidate SHA 에 어떤 형태로 붙는지 확인한다.
   - CI: `Build & Test`
   - CodeQL: `Analyze (javascript-typescript)`, `Analyze (python)`, `Analyze (rust)`
   - Render Diff: `Canvas visual diff`
3. `pull_request` 이벤트에서 check 가 PR head SHA 와 merge SHA 중 어디에 귀속되는지 확인한다.
4. 결과가 불명확하면 구현 범위를 CI workflow 로만 줄이거나 fast-pass 를 비활성화한다.

### 검증

- `gh pr checks` 또는 `gh api /repos/{owner}/{repo}/commits/{sha}/check-runs` 출력 기록.
- 조사 결과를 Stage 1 완료보고서에 기록.

### 산출물

- `mydocs/working/task_m100_1547_stage1.md`

## 3. Stage 2 — CI workflow preflight/gate 적용

### 목표

`.github/workflows/ci.yml` 의 `Build & Test` job 에 문서 전용 후속 커밋 fast-pass 조건을 적용한다.

### 설계

`preflight` job 을 추가한다.

- PR 이벤트가 아니면 `fast_pass=false`.
- PR commit 목록을 GitHub API 로 가져온다.
- 뒤에서부터 commit 을 검사한다.
  - 변경 파일이 전부 `mydocs/pr/**` 또는 `mydocs/orders/*.md` 이면 trailing docs-only commit 으로 인정한다.
  - single-parent commit 이 아니면 fast-pass 를 중단한다.
  - 허용 경로 밖 변경을 만나면 그 commit 을 candidate SHA 로 삼는다.
- trailing docs-only commit 이 0개이면 `fast_pass=false`.
- candidate SHA 가 없으면 `fast_pass=false`.
- candidate SHA 의 `Build & Test` check conclusion 이 `success`, `skipped`, `neutral` 중 하나이면
  `fast_pass=true`.
- check run 이 없거나 conclusion 이 불명확하면 `fast_pass=false`.

`build-and-test` job 은 다음 조건을 추가한다.

```yaml
needs: preflight
if: needs.preflight.outputs.fast_pass != 'true'
```

### 검증

- YAML 문법 확인.
- synthetic PR commit 목록 또는 실제 PR commit 목록으로 preflight 스크립트 dry-run.
- `git diff --check`.

### 산출물

- `.github/workflows/ci.yml`
- `mydocs/working/task_m100_1547_stage2.md`

## 4. Stage 3 — CodeQL 및 Render Diff workflow 확장

### 목표

CI workflow 와 같은 판정 원칙을 CodeQL, Render Diff 에 적용한다.

### 작업

1. `.github/workflows/codeql.yml`
   - `preflight` job 추가.
   - candidate SHA 에서 CodeQL matrix check 3개가 모두 green 일 때만 fast-pass.
   - `analyze` matrix job 은 `needs: preflight` 와 job-level `if` 로 skip.
2. `.github/workflows/render-diff.yml`
   - `preflight` job 추가.
   - candidate SHA 에서 `Canvas visual diff` 가 green 일 때만 fast-pass.
   - `canvas-visual-diff` job 은 `needs: preflight` 와 job-level `if` 로 skip.
3. workflow 별 preflight 스크립트 중복은 우선 허용한다.
   - 로컬 composite action 은 PR head 코드 실행/변조 리스크가 있어 이번 단계에서는 만들지 않는다.
   - 중복이 과하면 후속 이슈에서 base-branch 고정 reusable workflow 를 검토한다.

### 검증

- YAML 문법 확인.
- 각 workflow 의 expected check 이름이 Stage 1 조사 결과와 일치하는지 확인.
- `git diff --check`.

### 산출물

- `.github/workflows/codeql.yml`
- `.github/workflows/render-diff.yml`
- `mydocs/working/task_m100_1547_stage3.md`

## 5. Stage 4 — 운영 문서 보강

### 목표

collaborator-mediated 외부 PR 처리 경로에서 fast-pass 조건과 한계를 명확히 기록한다.

### 작업

`mydocs/manual/pr_review_workflow.md` 9장에 다음 내용을 보강한다.

- 리뷰 문서 전용 후속 커밋은 heavy CI fast-pass 대상이 될 수 있다.
- fast-pass 는 이전 코드 검증 SHA 가 green 일 때만 적용된다.
- 코드/테스트/workflow/샘플/baseline 변경이 있으면 반드시 heavy CI 를 다시 기다린다.
- fast-pass 가 적용되지 않는 경우 기존처럼 최신 PR head 기준 GitHub Actions 통과를 기다린다.

### 검증

- `rg -n "fast-pass|리뷰 문서 전용|collaborator-mediated|GitHub Actions" mydocs/manual/pr_review_workflow.md`
- `git diff --check`

### 산출물

- `mydocs/manual/pr_review_workflow.md`
- `mydocs/working/task_m100_1547_stage4.md`

## 6. Stage 5 — 최종 검증과 보고

### 목표

문서 전용 fast-pass 조건과 일반 CI 실행 조건을 모두 검증하고 최종 보고서를 작성한다.

### 검증 항목

1. 변경 범위 확인
   - `.github/workflows/ci.yml`
   - `.github/workflows/codeql.yml`
   - `.github/workflows/render-diff.yml`
   - `mydocs/manual/pr_review_workflow.md`
   - 작업 문서
2. 정적 확인
   - `git diff --check`
   - YAML 구조 확인
3. 로컬 dry-run
   - trailing docs-only commit range 판정 false/true 케이스 확인
   - candidate SHA check run 미존재 시 false 확인
4. GitHub Actions 확인
   - 가능하면 PR 을 열어 code-change update 에서 heavy job 이 실행되는지 확인
   - 이후 문서 전용 후속 커밋에서 heavy job 이 skipped 또는 fast-pass 되는지 확인

### 산출물

- `mydocs/working/task_m100_1547_stage5.md`
- `mydocs/report/task_m100_1547_report.md`
- `mydocs/orders/20260626.md` 상태 갱신

## 7. 승인 게이트

각 Stage 완료 후 다음을 수행한다.

1. 단계별 완료보고서 작성.
2. 해당 단계 변경과 보고서를 타스크 브랜치에서 커밋.
3. 작업지시자 승인 요청.
4. 승인 후 다음 Stage 진행.

소스 또는 workflow 파일 수정은 Stage 2부터 발생하므로, Stage 2 착수 전 별도 승인을 받은 뒤 진행한다.
