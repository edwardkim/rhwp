---
kind: memory
status: historical
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-07-17
---

PR #2331(#2072, jangster77)로 저장소 문서 거버넌스가 바뀌었다 (2026-07-17):

- **CLAUDE.md = 부트로더**(38줄). 절차·명령을 CLAUDE.md에 중복 기록하지 않는다.
- 로딩 순서: `AGENTS.md` → `mydocs/README.md`(manifest) → `manual/README.md`·`tech/README.md` 지도 → canonical 문서. 충돌 시 canonical 우선.
- 주요 canonical: 문서·Git 절차 = `mydocs/manual/codex/docs_and_git_workflow.md` (Folder Roles 포함 — orders/archives·plans/archives·feedback 행은 메인테이너 보완 25fac21e), PR 처리 = `mydocs/manual/pr_review_workflow.md`, 빌드/검증 = `dev_environment_guide.md`, CLI = `cli_commands.md`, 시각 검증 = `manual/verification/`.
- `mydocs/manual/memory/` = 이 라이브 메모리의 historical dump (status: historical, 충돌 시 canonical 우선).
- 문서 이동/링크 검사: `scripts/check_markdown_links.py` (일반 문서 변경엔 CI 미실행 — `markdown_link_check_guide.md`).

**How to apply**: 새 세션에서 절차 근거를 찾을 때 CLAUDE.md가 아니라 canonical 문서를 연다. 규칙 변경 시 CLAUDE.md가 아니라 해당 canonical 문서를 갱신하고, 필요하면 이 라이브 메모리와 dump(`mydocs/manual/memory/`)도 함께 갱신한다. 관련: [[feedback-process-must-follow]], [[hyper-waterfall-workflow]].
