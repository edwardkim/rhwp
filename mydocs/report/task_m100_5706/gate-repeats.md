# Gate repeats

PASS — 6/6 runs exit 0.

## 1. `py -3 tools/skill_router/gate_new_skill.py`

| Repeat | Exit | Summary |
|--------|------|---------|
| 1 | 0 | OK: 26 skills x 3 scans, catalog=25, route_probes=75 |
| 2 | 0 | OK: 26 skills x 3 scans, catalog=25, route_probes=75 |
| 3 | 0 | OK: 26 skills x 3 scans, catalog=25, route_probes=75 |

Counts: 3/3 pass. Each run: 26 skills × 3 scans, catalog=25, route_probes=75.

## 2. `py -3 -m unittest tools/skill_router/test_route.py tools.skill_router.test_skills_repeat`

| Repeat | Exit | Summary |
|--------|------|---------|
| 1 | 0 | Ran 12 tests in 1.119s — OK |
| 2 | 0 | Ran 12 tests in 1.118s — OK |
| 3 | 0 | Ran 12 tests in 1.063s — OK |

Counts: 3/3 pass. Each run: 12 tests OK.

PASS — 6/6 additional runs exit 0. Catalog now 26 (includes rhwp-skill-router).

## 3. `py -3 tools/skill_router/gate_new_skill.py` (second batch)

| Repeat | Exit | Summary |
|--------|------|---------|
| 1 | 0 | OK: 26 skills x 3 scans, catalog=26, route_probes=78 |
| 2 | 0 | OK: 26 skills x 3 scans, catalog=26, route_probes=78 |
| 3 | 0 | OK: 26 skills x 3 scans, catalog=26, route_probes=78 |

Counts: 3/3 pass. Each run: 26 skills × 3 scans, catalog=26 (includes rhwp-skill-router), route_probes=78.

## 4. `py -3 -m unittest tools/skill_router/test_route.py tools.skill_router.test_skills_repeat` (second batch)

| Repeat | Exit | Summary |
|--------|------|---------|
| 1 | 0 | Ran 12 tests in 0.733s — OK |
| 2 | 0 | Ran 12 tests in 0.928s — OK |
| 3 | 0 | Ran 12 tests in 0.690s — OK |

Counts: 3/3 pass. Each run: 12 tests OK.
