# Task M100 #2072 Stage 21 - 활성 manual 현행성 감사

## 목표

부트스트랩 정리 뒤에도 활성 manual에 남은 개인 환경, 종료 브랜치, 직접 `devel` push 절차를 제거한다.
현재 운영 가이드와 당시 경험을 기록한 historical 문서를 구분한다.

## 감사 대상

- `dev_environment_guide.md`
- `onboarding_guide.md`
- `publish_guide.md`
- `hyper_waterfall_docs_guide.md`
- `ai_pair_programming_guide.md`
- `export_png_command.md`
- `verification/svg_regression_diff.md`

## 변경 원칙

- 실사용 가이드는 최신 `upstream/devel` 기반 작업 브랜치와 PR 통합 절차만 안내한다.
- 개인 PC, 사설 IP, 개인 폰트 경로, 종료된 `local/devel` 흐름은 제거한다.
- 문서·Git 절차의 세부 규칙은 canonical workflow에 위임하고 중복하지 않는다.
- 당시 수치와 사례를 보존하는 AI pair programming 문서는 historical reference로 명시한다.

## 검증 계획

- 활성 manual에서 개인 절대경로와 구형 직접 push 절차 재검색
- 기본 링크와 전체 변경 Markdown 링크 검사
- 메타데이터, Python 구문, `actionlint`, `git diff --check`

## 수행 결과

- 활성 환경·온보딩·배포 가이드에서 개인 PC, 사설 IP, 개인 폰트 절대경로와 종료된
  `local/devel`·`local/task*` 절차를 제거했다.
- 현재 명령은 `upstream/devel` 기준 작업 브랜치, `devel` 대상 PR, `wasm-pack` 빌드로 통일했다.
- 작성 당시 수치와 구형 브랜치 사례를 보존하는 AI pair programming 문서는 `historical`로 분류하고
  현재 canonical workflow 링크를 명시했다.
- 검색에 남은 구형 경로는 `manual/memory/`와 `manual/codex/archive/`의 historical 기록뿐이다.
  `pr_review_workflow.md`의 `git push upstream devel`은 승인된 maintainer 직접 통합 옵션이므로 유지했다.

## 검증 결과

- 기본 Markdown 링크 검사: `374개`, 이상 없음
- `upstream/devel` 이후 전체 변경 Markdown 링크 검사: `324개`, 이상 없음
- 문서 메타데이터 검사: `209개`, 이상 없음
- `python3 -m py_compile scripts/check_document_metadata.py scripts/check_markdown_links.py`: 통과
- `actionlint .github/workflows/docs-link-check.yml`: 통과
- `git diff --check`: 통과
