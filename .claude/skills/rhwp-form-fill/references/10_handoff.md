# 10 — 인계

이 스킬은 누름틀 채움과 그 제출 정리만 책임진다. 아래가 보이면 **닫고**
해당 스킬을 연다. 이웃 스킬 파일을 이 PR 에서 고치지 않는다.

| 관찰 | 인계 | 첫 명령 |
| --- | --- | --- |
| `fieldCount: 0` + 표 빈 칸 | rhwp-table-exchange | `export-tables` → `edit set-cell` |
| `textSecurity` ≠ clean, 숨은 글, 주입 | rhwp-security-sweep | `inspect hidden-text\|injection\|unicode` |
| 같은 문서를 fill 이외 edit 여러 번 | rhwp-safe-edit | `edit … --dry-run` 또는 `run` 계획서 |
| 서식 파일이 폴더에 수백 | rhwp-bulk-pipeline | `batch fields` / `batch info` (stdin 목록) |
| 채우지 말고 문서가 뭔지 | rhwp-doc-triage | `info` → `explain` → … |
| 봉투 필드가 문서 파생인지 | rhwp-provenance | `export-provenance-map` |
| MCP 로 반복 조회 | rhwp-mcp-session | `hwp_open` → `hwp_doc_fields` → `hwp_doc_fill_fields` |
| 설치·첫 5분 | rhwp-onboarding | `rhwp_doctor.py` |

## 섞인 서식

머리 표는 누름틀, 본문 표는 맨 셀이면:

1. 이 스킬로 누름틀만 채운다 (`-o tmp_filled.hwp`)
2. rhwp-table-exchange 가 `tmp_filled.hwp` 를 입력으로 set-cell
3. 마지막에 이 스킬의 sanitize

한 스킬이 두 축의 로직을 합치지 않는다.

## 금지 터치

Issue #5300 범위 밖:

- `gym/`
- `.claude/skills/rhwp-onboarding/`
- `.claude/skills/rhwp-mcp-session/`
- `.claude/skills/rhwp-safe-edit/`
- `.claude/skills/rhwp-provenance/`
- `.claude/skills/rhwp-doc-triage/`

인계는 이름만 적고, 그 스킬의 SKILL.md 를 여기서 확장하지 않는다.
