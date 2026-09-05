# Catalog route coverage

PASS — 27/27 catalog skills win at least one `route.py --json` request.
Skills that failed to win: **none**.

측정: 2026-08-20 09:14:51. `PYTHONIOENCODING=utf-8`, `PYTHONUTF8=1`.
CWD: `C:\Users\swsz9\rhwp-skill-router`.
`intents.py` was not edited. Unique remaining catalog triggers covered the two first-trigger collisions.

Test: `tools/skill_router/test_catalog_routes.py` (stdlib unittest).
Each skill's first `catalog.json` trigger is sent to `python tools/skill_router/route.py "<request>" --json` (timeout 20s). Envelope keys asserted: `schemaVersion`, `request`, `intent`, `requiredCapabilities`, `skillSelection`, `executionGraph`. `requiredCapabilities` non-empty. `executionGraph` is a dict with `nodes`+`edges` (non-empty) or a list of nodes. Coverage test: winning skill ids == catalog skill ids.

## First-trigger collisions

| Skill | First trigger | Loses to | Dedicated probe used |
|-------|---------------|----------|----------------------|
| rhwp-exam-ingest | HWPX로 만들어줘 | rhwp-codex (no exam-ingest pattern) | 한글 시험지로 변환 |
| rhwp-safe-edit | 누름틀 채워 | rhwp-form-fill (specificity 95 > 70) | 문구 일괄 치환 |

The other 25 skills win on their first catalog trigger.

## Probe set (27)

| Skill | Request | Source |
|-------|---------|--------|
| rhwp-agent-surface | 표면 추가/새 MCP 도구/새 --json 명령 | first trigger |
| rhwp-bug-hunter | 버그 찾아줘(실사용 기준) | first trigger |
| rhwp-bulk-pipeline | 폴더 전체를 텍스트로/한꺼번에 변환 | first trigger |
| rhwp-chief | 요청 큐 돌려/처리해 | first trigger |
| rhwp-cli | .hwp/.hwpx 파일을 SVG/PNG/PDF/텍스트로 내보내 | first trigger |
| rhwp-codex | rhwp 사용법/전체 명령/뭘 쓸지 모르겠다 | first trigger |
| rhwp-contributor | rhwp에 기여 | first trigger |
| rhwp-doc-triage | 이 hwp 뭔 문서야? | first trigger |
| rhwp-exam-ingest | 한글 시험지로 변환 | dedicated |
| rhwp-explore | 이 문서로 뭘 할 수 있어? | first trigger |
| rhwp-fde | 고객이 이 문서가 안 열린대 | first trigger |
| rhwp-fidelity-compare | 한컴 PDF와 비교 | first trigger |
| rhwp-form-fill | 이 서식/신청서/양식 채워줘 | first trigger |
| rhwp-handoff | 세션 핸드오프 | first trigger |
| rhwp-knowledge-map | 지식 지도 | first trigger |
| rhwp-mcp-session | rhwp 를 MCP로 붙여/등록해 | first trigger |
| rhwp-onboarding | rhwp 처음/설치/시작/온보딩 | first trigger |
| rhwp-provenance | 이 값이 문서에서 온 건가 | first trigger |
| rhwp-recipes | 어떤 레시피로 가? | first trigger |
| rhwp-safe-edit | 문구 일괄 치환 | dedicated |
| rhwp-security-sweep | 이 문서 보내도 돼/배포 전 점검 | first trigger |
| rhwp-skill-author | 새 스킬 | first trigger |
| rhwp-skill-router | 라우터 | first trigger |
| rhwp-strategist | 이 문서들로 전략 보고서/제안서 | first trigger |
| rhwp-table-exchange | 표를 CSV/엑셀로 뽑아줘 | first trigger |
| rhwp-visual-regression | 편집 전후 화면 비교 | first trigger |
| rhwp-work-receipt | 이 작업 증명해/영수증 남겨 | first trigger |

## 1. `python -m unittest tools.skill_router.test_catalog_routes`

3 tests: catalog present, every skill wins one CLI route, winning set == catalog ids.

| Repeat | Exit | Summary |
|--------|------|---------|
| 1 | 0 | Ran 3 tests in 3.464s — OK |
| 2 | 0 | Ran 3 tests in 3.439s — OK |
| 3 | 0 | Ran 3 tests in 3.588s — OK |

Counts: 3/3 pass. 27/27 skills covered. 0 failed to win.

## 2. `python tools/skill_router/gate_new_skill.py`

| Repeat | Exit | Summary |
|--------|------|---------|
| 1 | 0 | OK: 27 skills x 3 scans, catalog=26, route_probes=78, rhwp=207 |
| 2 | 0 | OK: 27 skills x 3 scans, catalog=26, route_probes=78, rhwp=207 |
| 3 | 0 | OK: 27 skills x 3 scans, catalog=27, route_probes=81, rhwp=207 |

Counts: 3/3 pass. Repeats 1–2 ran while catalog.json still had 26 skills (`rhwp-skill-author` existed under `.agents/skills/` and was scanned). Repeat 3 ran after `rhwp-skill-author` was in catalog.json (27 ids, 81 route probes). `rhwp.exe` command check: 207 known, 189 refs.

Live rhwp: `C:\Users\swsz9\rhwp-skill-router\target\release\rhwp.exe`.
