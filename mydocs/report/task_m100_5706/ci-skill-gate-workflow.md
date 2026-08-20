# Skill router gate CI workflow

Standalone cheap Python gate. It is not `ci.yml` (no cargo, rust, nextest, or the full test suite).

## Why

A new or edited skill can ship a `rhwp <cmd>` that does not exist. The Python gate (`tools/skill_router/gate_new_skill.py`) already extracts those tokens and, when a binary is present, checks them against `rhwp capabilities` ∪ `--help`. Waiting on rust shards to surface that is slow; a missing command should fail as soon as the skill or router files change.

This workflow runs only that Python surface, on PRs that touch it, so the hole is closed even while Build & Test is still compiling.

## File

`.github/workflows/skill-router-gate.yml`  
Workflow name: `Skill router gate`

## Triggers

`pull_request` (branches `main`, `devel`; types opened / reopened / synchronize) when any of:

- `.claude/skills/**`
- `.agents/skills/**`
- `tools/skill_router/**`
- `scripts/tests/test_skill_router_gate.py`
- `.github/workflows/skill-router-gate.yml`

Also `workflow_dispatch` (manual, no path filter).

Does not run cargo/rust. `PYTHONUTF8=1` for the whole job.

## What it runs

One job, `ubuntu-latest`, Python 3.12. Fail-fast: any non-zero exit fails the job (`set -euo pipefail` on the 3× loop; later steps do not run if an earlier step fails).

1. `python tools/skill_router/gate_new_skill.py` three times (loop). Frontmatter, executable `rhwp <cmd>` refs, catalog paths, route.py envelope probes. Live `rhwp` command check only if a binary is on disk or PATH (CI does not build one).
2. `python -m unittest tools.skill_router.test_route`
3. `python -m unittest scripts.tests.test_skill_router_gate` (itself repeats the gate and route unittest 3×)

Hang bound: 15 minutes (not a performance target).
