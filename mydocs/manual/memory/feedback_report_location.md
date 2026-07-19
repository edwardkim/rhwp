---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: 최종 보고서 위치 규칙
description: 타스크 최종 완료보고서는 mydocs/report/에 저장, 단계별 보고서는 mydocs/working/에 저장
type: feedback
originSessionId: 1f035a49-cf55-4427-a5b6-ba6a493aa832
---
최종 완료보고서는 `mydocs/report/task_{milestone}_{이슈번호}_report.md`에 저장한다.
단계별 완료보고서는 `mydocs/working/task_{milestone}_{이슈번호}_stage{N}.md`에 저장한다.

**Why:** 문서 폴더 규칙(현행 canonical: `mydocs/manual/codex/docs_and_git_workflow.md` Folder Roles — 2026-07-17 CLAUDE.md 부트로더화 이후). `working/`은 단계별 보고서 전용이고 `report/`는 최종 보고서 전용이다. #62r 타스크에서 최종 보고서를 `working/`에 생성하는 실수 발생.

**How to apply:** 최종 완료보고서(`_report.md`) 작성 시 항상 `mydocs/report/`에 저장한다.
