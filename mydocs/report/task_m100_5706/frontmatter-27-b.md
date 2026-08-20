# Frontmatter re-scan B — `.claude/skills/*/SKILL.md` (27)

PASS — 27/27. FAIL — 0/27. Offenders: none.

재측정 B: 2026-08-20. `.claude/skills/*/SKILL.md` 27개 전수 재독(동일 파서 3회, SHA-256 일치).
신규 `rhwp-skill-author` 포함. SKILL.md 수정 없음. 커밋 없음. push 없음.
기준은 `tests/skills_contract.rs` 의
`skills_have_valid_frontmatter_and_are_executable` 와 동일한 파싱이다
(`tools/skill_router/gate_new_skill.py` 도 같은 규칙).

디스크에서 바이트를 세 번 다시 읽었다. 스캔 1·2·3 의 계약 레코드·TSV·파일 SHA-256 연결값이 같다.

## 규칙

1. YAML frontmatter `name` 이 폴더명과 같다 (`name:` 줄, trim).
2. `description` 이 20자 이상 (유니코드 스칼라, Rust `.chars().count()` / Python `len(str)`).
3. 본문 전체(frontmatter 포함)에 실행 가능한 `rhwp <명령>` 이 하나 이상.
   명령 토큰은 ASCII 소문자로 시작하는 `[a-z0-9-]+`.
   `rhwp <명령>` 플레이스홀더, `rhwp CLI`, `rhwp 를` 는 참조가 아니다.

부가: 첫 줄 `---`, 닫는 `---`, SKILL.md 존재, UTF-8 BOM 없음.

스킬 폴더 27개 전수. 폴더에 SKILL.md 가 없거나 SKILL.md 만 있는 고아는 없다.
frontmatter 블록은 전 파일 `name` + `description` 두 줄뿐(기타 YAML 키 0).

## 3회 재스캔

| pass | n | PASS | FAIL | TSV SHA-256 | record SHA-256 | concat file SHA-256 |
|---|---:|---:|---:|---|---|---|
| 1 | 27 | 27 | 0 | `e3c57210198df5d77632ee0d5225c55434b14b86085607da2edbe0baf8943bf3` | `dad14bd49b5c53793abcc777dde1f9360c8c2664cca99a0d1cec909d5cd0e1b9` | `d7bd97b446bfba17c41d37f360f180cd85ba258450ec499ca4f83cbdc060c3a8` |
| 2 | 27 | 27 | 0 | `e3c57210198df5d77632ee0d5225c55434b14b86085607da2edbe0baf8943bf3` | `dad14bd49b5c53793abcc777dde1f9360c8c2664cca99a0d1cec909d5cd0e1b9` | `d7bd97b446bfba17c41d37f360f180cd85ba258450ec499ca4f83cbdc060c3a8` |
| 3 | 27 | 27 | 0 | `e3c57210198df5d77632ee0d5225c55434b14b86085607da2edbe0baf8943bf3` | `dad14bd49b5c53793abcc777dde1f9360c8c2664cca99a0d1cec909d5cd0e1b9` | `d7bd97b446bfba17c41d37f360f180cd85ba258450ec499ca4f83cbdc060c3a8` |

scan1 == scan2 == scan3.

## Offenders

없음.

## 전수 표

| folder | name==folder | desc_len | rhwp refs | unique tokens (sample) | result |
|---|---|---:|---:|---|---|
| rhwp-agent-surface | yes | 596 | 12 | capabilities, export-provenance-map | PASS |
| rhwp-bug-hunter | yes | 191 | 1 | info | PASS |
| rhwp-bulk-pipeline | yes | 413 | 5 | batch, capabilities | PASS |
| rhwp-chief | yes | 260 | 12 | convert, edit, export-hwpx, export-pdf, export-tables, export-text, table-to-csv | PASS |
| rhwp-cli | yes | 388 | 9 | dump, dump-pages, export-render-tree, export-svg, hwp5-anchor-trace, hwp5-inventory-diff, hwp5-table-probe, ir-diff | PASS |
| rhwp-codex | yes | 307 | 3 | capabilities | PASS |
| rhwp-contributor | yes | 301 | 6 | audit, lineage, replay | PASS |
| rhwp-doc-triage | yes | 351 | 1 | info | PASS |
| rhwp-exam-ingest | yes | 256 | 9 | build-from-ingest, dump, export-text | PASS |
| rhwp-explore | yes | 385 | 20 | capabilities, chart-to-csv, digest, explain, explore, export-structure, export-tables, fields, inspect | PASS |
| rhwp-fde | yes | 407 | 1 | capabilities | PASS |
| rhwp-fidelity-compare | yes | 360 | 6 | export-svg, render-diff | PASS |
| rhwp-form-fill | yes | 330 | 1 | edit | PASS |
| rhwp-handoff | yes | 402 | 4 | edit, replay | PASS |
| rhwp-knowledge-map | yes | 330 | 7 | capabilities, mcp-serve | PASS |
| rhwp-mcp-session | yes | 446 | 6 | capabilities, mcp-serve | PASS |
| rhwp-onboarding | yes | 409 | 8 | digest, explain, export-text, info, inspect, mcp-serve | PASS |
| rhwp-provenance | yes | 465 | 8 | armor, export-provenance-map, inspect | PASS |
| rhwp-recipes | yes | 332 | 9 | batch, edit, export-tables, fields, info, inspect, render-diff | PASS |
| rhwp-safe-edit | yes | 372 | 11 | edit, export-plan-schema, export-tables, run, search | PASS |
| rhwp-security-sweep | yes | 354 | 17 | digest, edit, fields, info, inspect | PASS |
| rhwp-skill-author | yes | 204 | 3 | capabilities, export-svg, info | PASS |
| rhwp-skill-router | yes | 263 | 5 | capabilities, edit, export-svg, fields, info | PASS |
| rhwp-strategist | yes | 251 | 1 | capabilities | PASS |
| rhwp-table-exchange | yes | 326 | 8 | batch, csv-to-table, export-tables, table-to-csv | PASS |
| rhwp-visual-regression | yes | 300 | 3 | ir-diff, render-diff | PASS |
| rhwp-work-receipt | yes | 331 | 13 | audit, lineage, replay, run | PASS |

## 요약

- scanned: 27
- PASS: 27
- FAIL: 0
- name match: 27/27
- description ≥ 20: 27/27 (shortest: `rhwp-bug-hunter` 191; new `rhwp-skill-author` 204)
- ≥1 `rhwp [a-z]` token: 27/27 (single unique-token skills: bug-hunter `info`, codex `capabilities`, doc-triage `info`, fde `capabilities`, form-fill `edit`, strategist `capabilities`)
- UTF-8 BOM: 0/27
- CRLF: 0/27
- closing `---`: 27/27 (전부 줄 3, `name`+`description` 뒤)
- extra YAML keys: 0/27
- offenders: 0
- 3회 재스캔 SHA-256: `e3c57210198df5d77632ee0d5225c55434b14b86085607da2edbe0baf8943bf3` (일치; 전수 표 TSV UTF-8)
- 3회 재스캔 레코드 SHA-256: `dad14bd49b5c53793abcc777dde1f9360c8c2664cca99a0d1cec909d5cd0e1b9` (일치)
- 3회 재스캔 파일 SHA-256 연결: `d7bd97b446bfba17c41d37f360f180cd85ba258450ec499ca4f83cbdc060c3a8` (일치)

소스 수정 없음. 커밋 없음.
