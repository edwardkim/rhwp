# skill_router

사용자 요청을 `request → intent → requiredCapabilities → skillSelection → executionGraph`
한 장의 JSON 봉투로 만든다. 의존성 0 (Python 3 표준 라이브러리).

```bash
python tools/skill_router/route.py "이 서식 채워줘" --json
python tools/skill_router/route.py "PR 올려" --json
```

봉투 키는 고정이다: `schemaVersion`, `request`, `intent`, `requiredCapabilities`,
`skillSelection`, `executionGraph`, `untrustedContent`, `untrustedFields`.

`executionGraph` 는 `{nodes, edges}` 다. 노드는 `id, skill, action, command`.
가장자리가 `from → to`.

## 새 스킬을 만들면 반드시 3회

새 스킬을 만들면 아래 명령을 반드시 3회 실행한다.

```bash
python tools/skill_router/gate_new_skill.py
python -m unittest tools/skill_router/test_route.py
cargo test --test regression_suite_015 skills_have_valid_frontmatter -- --nocapture
```

## git pre-commit

Skill-path changes (`.agents/skills/`, `.agents/skills/`, `tools/skill_router/`)
run the 3-pass gate automatically on commit.

```bash
python tools/skill_router/install_git_hook.py
python tools/skill_router/precommit_skill_gate.py
```

The hook execs `precommit_skill_gate.py` with the same Python. That script
runs `gate_new_skill.py` three times and
`python -m unittest tools.skill_router.test_route` once. Unrelated commits
skip the gate (exit 0). Linked worktrees install into that worktree's
git-dir hooks, not the main repo common hooks.

## Catalog PROBES and CI

- The gate fails if a PROBE selects a different skill.
- Every catalog skill needs 3 unique PROBES.
- `python tools/skill_router/check_catalog_sync.py`
- `python tools/skill_router/precommit_skill_gate.py`
- CI: `.github/workflows/skill-router-gate.yml`
