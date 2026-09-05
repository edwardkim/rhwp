# 10 — dirty named worktree 를 비우지 않는다

트리거/갈래: `dirty_named_worktree`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

`C:\Users\swsz9\rhwp-handoff` 가 dirty 하다고 해서
`git reset --hard` 하지 않는다. 그 트리는 다른 브랜치의 집이다.

```bash
git -C <클론> fetch upstream devel
git -C <클론> worktree add -b feat/agent-handoff <빈경로> upstream/devel
```

레지스트리: `fixtures/layouts/forbidden-worktrees/registry.json`.
