# 13 — 이름 붙은 트리를 checkout 하지 않는다

트리거/갈래: `isolation`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

금지 목록은 isolation-worktree.md 와 registry.json 이 같다.
`rhwp-handoff` 폴더 이름이 이 스킬과 닮았어도 그 트리를 쓰지 않는다.
표본: `fixtures/envelopes/named_worktree_checkout_rejected.json`.
