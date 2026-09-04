# Task M100 #1601 구현 계획서

## Stage 1

`mydocs/**` 전용 변경 fast-pass 기준을 workflow에 반영한다.

- `ci.yml`
  - preflight step 이름을 `mydocs-only` 기준으로 정리한다.
  - allowlist 함수를 `mydocs/**` 기준으로 확장한다.
  - PR 전체가 `mydocs/**`만 변경하면 `Build & Test`를 skip한다.

- `codeql.yml`
  - `ci.yml`과 같은 allowlist 기준을 적용한다.
  - PR 전체가 `mydocs/**`만 변경하면 CodeQL matrix를 skip한다.

- `render-diff.yml`
  - PR trigger path에 `mydocs/**`를 포함한다.
  - preflight allowlist를 `mydocs/**` 기준으로 확장한다.
  - 문서 전용 PR에서 `Canvas visual diff`를 skip한다.

## Stage 2

운영 문서를 갱신한다.

- `pr_review_workflow.md`
  - fast-pass 설명을 `mydocs/**` 기준으로 정정한다.
  - 영어 요청 contributor 응답 시 한글 문단과 영어 문단을 분리해 병기하는 규칙을 추가한다.

## Stage 3

검증과 PR 생성.

- workflow 문법을 `actionlint`로 확인한다.
- `git diff --check`로 whitespace 문제를 확인한다.
- PR 본문에는 workflow 변경으로 인해 full CI가 한 번 실행되는 점과, merge 후 `mydocs/**` 전용 PR에서 기대되는
  preflight/skip 상태를 기록한다.
