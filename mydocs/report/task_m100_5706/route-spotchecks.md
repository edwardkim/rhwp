# Route spot-checks

PASS — 9/9 requests × 1, selected skill matches expected.

측정: 2026-08-20. `PYTHONIOENCODING=utf-8`, `PYTHONUTF8=1`.
`python tools/skill_router/route.py "<요청>" --json` stdout 을 UTF-8 **without BOM**
임시 파일에 쓴 뒤 `json.loads` 로 파싱했다. 선택 스킬은
`skillSelection[0].id`.

임시 봉투: `C:\Users\swsz9\AppData\Local\Temp\rhwp-route-spotchecks\envelopes\01.json` … `09.json`.

| # | Request | Expected | Selected | Intent | Exit | Parse | Result |
|---|---------|----------|----------|--------|------|-------|--------|
| 1 | 이 서식 채워줘 | rhwp-form-fill | rhwp-form-fill | fill-form | 0 | OK | PASS |
| 2 | PR 올려 | rhwp-contributor | rhwp-contributor | contribute | 0 | OK | PASS |
| 3 | 어떤 스킬을 쓰지 | rhwp-skill-router | rhwp-skill-router | route | 0 | OK | PASS |
| 4 | 표를 CSV로 뽑아줘 | rhwp-table-exchange | rhwp-table-exchange | table-csv | 0 | OK | PASS |
| 5 | 이 문서 보내도 돼? | rhwp-security-sweep | rhwp-security-sweep | security | 0 | OK | PASS |
| 6 | 버그 찾아줘 | rhwp-bug-hunter | rhwp-bug-hunter | hunt-bug | 0 | OK | PASS |
| 7 | 한컴 PDF와 비교 | rhwp-fidelity-compare | rhwp-fidelity-compare | compare-fidelity | 0 | OK | PASS |
| 8 | 요청 큐 돌려 | rhwp-chief | rhwp-chief | run-request-queue | 0 | OK | PASS |
| 9 | rhwp 처음인데 온보딩 | rhwp-onboarding | rhwp-onboarding | onboard | 0 | OK | PASS |

Counts: 9/9 pass. Repeat=1. stderr empty on all nine. 각 봉투 `schemaVersion` 1.0,
`untrustedContent` false, `untrustedFields` []. BOM 없음.

## Selected skill detail

| # | Skill path | Confidence | Graph nodes |
|---|------------|------------|-------------|
| 1 | `.agents/skills/rhwp-form-fill/SKILL.md` | 0.99 | fields → dry-run-fill → fill-verify → sanitize |
| 2 | `.agents/skills/rhwp-contributor/SKILL.md` | 0.9 | issue → analyze → branch → implement → fmt-clippy-test → working-doc → pr |
| 3 | `.agents/skills/rhwp-skill-router/SKILL.md` | 0.876 | capabilities → info → export-svg |
| 4 | `.agents/skills/rhwp-table-exchange/SKILL.md` | 0.96 | export-tables → table-to-csv → csv-dry-run → csv-verify |
| 5 | `.agents/skills/rhwp-security-sweep/SKILL.md` | 0.864 | hidden-text → injection → unicode → redact-dry-run → redact-sanitize → resweep |
| 6 | `.agents/skills/rhwp-bug-hunter/SKILL.md` | 0.844 | info → export-svg → render-diff → inspect |
| 7 | `.agents/skills/rhwp-fidelity-compare/SKILL.md` | 0.95 | info → export-svg → export-render-tree |
| 8 | `.agents/skills/rhwp-chief/SKILL.md` | 0.932 | info → export-pdf → export-tables → fill |
| 9 | `.agents/skills/rhwp-onboarding/SKILL.md` | 0.91 | doctor → binary → selftest → mcp-json → first-5-min |

`requiredCapabilities[0]` 는 각 행의 Selected 와 같다.
