# Task M100 #1574 Stage 3 — Render Diff 순수 문서 PR fast-pass 보정

- 이슈: #1574
- 브랜치: `task_m100_1574_render_doc_fast_pass`
- 작성일: 2026-06-26

## 1. 배경

PR #1575를 머지한 뒤 PR #1572에서 review 문서 전용 브랜치 업데이트를 검증했다.
GitHub의 Update branch 버튼이 만든 merge commit은 마지막 커밋이 문서 전용이 아니어서 fast-pass 조건을
깨는 것이 확인됐다. 이를 rebase + force-with-lease 방식으로 바로잡자 CodeQL은 fast-pass skip으로 동작했다.

하지만 Render Diff는 PR 전체가 `mydocs/pr/**`, `mydocs/orders/*.md`만 변경하는 경우에도 base push에는
`Canvas visual diff` check가 없기 때문에 full Canvas job으로 fallback했다.

## 2. 수정

- `.github/workflows/render-diff.yml`
  - PR 전체가 허용된 review 문서 경로만 변경한 경우 `pr.base.sha`를 candidate로 기록하되,
    `Canvas visual diff` 기존 check를 요구하지 않고 `all-review-docs-no-render-impact`로 fast-pass한다.
  - trailing 문서 커밋 뒤에 코드/렌더 변경 candidate가 있는 경우는 기존처럼 candidate SHA의
    `Canvas visual diff` green check를 요구한다.

## 3. 검증 계획

- `actionlint .github/workflows/render-diff.yml`
- workflow YAML parse
- `git diff --check`
- 후속 PR CI 통과 후 #1572를 rebase 문서-only head로 다시 갱신해 Render Diff skip 여부 확인
