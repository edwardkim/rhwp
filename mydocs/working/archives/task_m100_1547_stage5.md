# Task M100 #1547 Stage 5 완료보고서 — 최종 검증

- 이슈: #1547
- 브랜치: `local/task1547`
- 작성일: 2026-06-26
- 단계: Stage 5 — 최종 검증과 보고

## 1. 검증 범위

최종 검증은 `upstream/devel..HEAD` 변경 범위를 기준으로 수행했다.

변경 파일:

- `.github/workflows/ci.yml`
- `.github/workflows/codeql.yml`
- `.github/workflows/render-diff.yml`
- `mydocs/manual/pr_review_workflow.md`
- `mydocs/orders/20260626.md`
- `mydocs/plans/task_m100_1547.md`
- `mydocs/plans/task_m100_1547_impl.md`
- `mydocs/working/task_m100_1547_stage1.md`
- `mydocs/working/task_m100_1547_stage2.md`
- `mydocs/working/task_m100_1547_stage3.md`
- `mydocs/working/task_m100_1547_stage4.md`

Stage 5 에서 추가 작성한 본 보고서와 최종 보고서는 별도 커밋에 포함한다.

## 2. 정적 검증

| 명령 | 결과 |
|---|---|
| `actionlint .github/workflows/ci.yml .github/workflows/codeql.yml .github/workflows/render-diff.yml` | 통과 |
| `ruby -e 'require "yaml"; ...'` | 3개 workflow YAML 파싱 통과 |
| `git diff --check upstream/devel..HEAD` | 통과 |
| `git diff --stat upstream/devel..HEAD` | 변경 범위 확인 |

## 3. API 기반 검증

Stage 1~3 에서 실제 PR metadata 를 조회해 다음을 확인했다.

- GitHub Actions check-run 은 PR test-merge SHA 가 아니라 PR head SHA 에 붙는다.
- classic commit status endpoint 는 현재 저장소의 GitHub Actions 판정 기준으로 부적합하다.
- PR #1544 head SHA 에서 `Build & Test`, CodeQL matrix 3개, `Canvas visual diff` 가 모두 green 으로 조회됐다.
- PR #1541 head SHA 에서는 Render Diff 대상이 아니어서 `Canvas visual diff` check-run 이 없었다.

이 결과를 반영해 missing check 또는 불명확한 API 결과는 모두 fast-pass 하지 않고 heavy job 실행으로 떨어지게
구성했다.

## 4. Live GitHub Actions 검증 상태

이번 Stage 에서는 브랜치를 push 하거나 PR 을 생성하지 않았다. 따라서 실제 PR UI 에서 heavy job 이
`skipped` 로 표시되고 branch protection 을 통과하는지는 아직 확인하지 않았다.

남은 확인 사항:

- PR 생성 후 code-change update 에서는 기존 heavy job 이 실행되는지 확인.
- review 문서 전용 후속 커밋에서는 heavy job 이 skipped 되거나 fast-pass 되는지 확인.
- branch protection 이 required check 로 advanced-security `CodeQL` 체크를 요구하는지 maintainer/admin 이
  확인.

## 5. 작업트리 주의 사항

관련 없는 untracked 파일 `mydocs/pr/pr_1530_review.md` 가 작업트리에 존재한다. 이번 task 커밋에는 포함하지
않았다.

## 6. 판정

정적 검증과 API 기반 설계 검증 기준으로는 Stage 1~5 완료 조건을 충족한다.

다만 live GitHub Actions 검증은 PR 생성 후 수행해야 하므로, 최종 merge 전 확인 항목으로 남긴다.
