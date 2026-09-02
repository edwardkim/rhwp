# 20 — 인계

이 스킬은 폴더 N건의 batch 표면만 닫는다. 옆 스킬 본문을 여기 복사하지 않는다.

| 상황 | 인계 |
| --- | --- |
| 문서 1건을 컨텍스트 아끼며 읽기 | `rhwp-doc-triage` |
| 누름틀 1건 채움·순번·sanitize | `rhwp-form-fill` |
| 표 CSV 왕복 | `rhwp-table-exchange` |
| 배포 전 은닉/주입/유니코드 | `rhwp-security-sweep` |
| 원본을 계획서로 여러 번 편집 | `rhwp-safe-edit` |
| MCP 세션·도구 선택 | `rhwp-mcp-session` (`hwp_batch` 에 convert 없음) |
| 작업 영수증·감사 | `rhwp-work-receipt` |
| 출처 표지 소비 | `rhwp-provenance` |
| 첫 설치·doctor | `rhwp-onboarding` |

금지 트리: `gym/`. 금지 재작성: onboarding, mcp-session, safe-edit,
provenance, doc-triage, form-fill SKILL 본문.

인계할 때 넘기는 것: 목록 경로, NDJSON 경로, 실패 행, 게이트 숫자.
인계하지 않는 것: 지어낸 서브커맨드, 비밀번호가 붙은 batch 명령줄.

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `20_handoff.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
