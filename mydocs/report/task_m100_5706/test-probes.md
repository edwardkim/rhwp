# Probe route unittest

PASS — `tools/skill_router/test_probes.py` 3 consecutive runs, all exit 0.
Failing probes: **none**.

`python -m unittest tools.skill_router.test_probes` from repo root
(`C:\Users\swsz9\rhwp-skill-router`). `PYTHONUTF8=1`, `PYTHONIOENCODING=utf-8`.
CLI: `python tools/skill_router/route.py "<request>" --json`, timeout 20s.

Loads `PROBES` from `tools/skill_router/gate_new_skill.py`. Every tuple
member is routed; selected skill is `skillSelection[0].id` and must equal
the catalog skill key. Probes were not edited (main owns those).

## Files

| Path | Change |
|------|--------|
| `tools/skill_router/test_probes.py` | NEW unittest |
| `mydocs/report/task_m100_5706/test-probes.md` | this evidence |

Not changed: `tools/skill_router/gate_new_skill.py`.
No commit, no push.

## Counts

- unittest methods: **1** (`test_every_probe_routes_to_its_skill_id`)
- catalog skills in `PROBES`: **27**
- probes asserted: **81** (27 × 3, via `subTest`)
- mismatches (selected ≠ catalog skill): **0**

## 3x `python -m unittest tools.skill_router.test_probes`

| Repeat | Exit | Summary |
|--------|------|---------|
| 1 | 0 | Ran 1 test in 9.687s — OK |
| 2 | 0 | Ran 1 test in 10.000s — OK |
| 3 | 0 | Ran 1 test in 9.942s — OK |

Counts: 3/3 pass. 81/81 probes routed to the owning skill id.

## Failing probes

None.
