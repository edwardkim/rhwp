# rhwp-skill-author — 3-pass gate evidence

PASS — 3 consecutive gate runs and 3 consecutive unittest runs, all exit 0.
Route smoke `"새 스킬 만들어"` selects `rhwp-skill-author`.

No registry row added. `CAP-5706` already identifies `rhwp-skill-router`.
A new `CAP-<Issue>` would require a dedicated GitHub Issue; inventing or
duplicating IDs is forbidden. Catalog `registrationId` is `CAP-5706` (same
issue bundle). Catalog `capabilityId` is unique: `rhwp-skill-author`.

## Files changed

| Path | Change |
|------|--------|
| `.claude/skills/rhwp-skill-author/SKILL.md` | NEW skill |
| `tools/skill_router/catalog.json` | append one object, id `rhwp-skill-author` |
| `tools/skill_router/intents.py` | append `author-skill` spec, specificity 96 |
| `tools/skill_router/graph.py` | append `author_skill_graph` (scaffold → catalog → gate 3x → unittest 3x) |
| `mydocs/report/task_m100_5706/skill-author.md` | this evidence |

Not changed: `mydocs/manual/agent_capability_registry.md`, other skills'
`SKILL.md`, `test_route.py`. No commit, no push.

## 1. `python tools/skill_router/gate_new_skill.py`

| Repeat | Exit | Summary |
|--------|------|---------|
| 1 | 0 | OK: 27 skills x 3 scans, catalog=27, route_probes=81, rhwp=207 |
| 2 | 0 | OK: 27 skills x 3 scans, catalog=27, route_probes=81, rhwp=207 |
| 3 | 0 | OK: 27 skills x 3 scans, catalog=27, route_probes=81, rhwp=207 |

Counts: 3/3 pass. Each run: 27 skills × 3 scans (includes `rhwp-skill-author`),
catalog=27, route_probes=81. Live rhwp: `target/release/rhwp.exe`,
`rhwp commands: pass (207 known, 189 refs)`.

Author probes each run: `'새 스킬'`, `'스킬 만들어'`, `'create a skill'` — pass.

## 2. `python -m unittest tools/skill_router/test_route.py`

| Repeat | Exit | Summary |
|--------|------|---------|
| 1 | 0 | Ran 8 tests in 0.851s — OK |
| 2 | 0 | Ran 8 tests in 0.658s — OK |
| 3 | 0 | Ran 8 tests in 0.643s — OK |

Counts: 3/3 pass.

## Route smoke

```bash
python tools/skill_router/route.py "새 스킬 만들어" --json
```

```json
{
  "schemaVersion": "1.0",
  "request": "새 스킬 만들어",
  "intent": {
    "id": "author-skill",
    "label": "스킬 작성",
    "confidence": 0.972
  },
  "requiredCapabilities": [
    "rhwp-skill-author"
  ],
  "skillSelection": [
    {
      "id": "rhwp-skill-author",
      "path": ".claude/skills/rhwp-skill-author/SKILL.md",
      "reason": "스킬 작성 요청이므로 rhwp-skill-author 을(를) 선택한다 (겹치면 더 구체적인 스킬)"
    }
  ]
}
```

`intent.id` = `author-skill`. `skillSelection[0].id` = `rhwp-skill-author`.

Graph: scaffold → catalog → gate-3x → unittest-3x.
