# 28 — 예외 네 갈래를 한 표로

트리거/갈래: `exceptions`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

| id | exit | 다음 |
|---|---:|---|
| missing_capsule | 1 | 체인 단절, 날조 금지 |
| parent_hash_mismatch | 3 | 후속 --parent 금지 |
| dirty_named_worktree | 2 | 새 isolation |
| disk_full | 1 | 추가 쓰기 금지 |

정본: `references/exception-index.md`.
픽스처: `fixtures/exceptions/*.json`.
