# 12 — git add -A 는 인계를 더럽힌다

트리거/갈래: `staging`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

sandbox_ 와 저널과 수거물이 스테이징된다.

```bash
git add -- .claude/skills/rhwp-handoff/SKILL.md
git add -- mydocs/working/agent_handoff.md
```

표본: `fixtures/envelopes/git_add_a_rejected.json` (`rejected: true`).
