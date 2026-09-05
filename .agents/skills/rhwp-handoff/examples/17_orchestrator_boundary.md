# 17 — boundary 는 한 번에 거절

트리거/갈래: `rejected`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

표본: `fixtures/envelopes/orch_boundary.json`.
`code=4000`, `outcome=rejected`, attempts 길이 1.
finding code 예: `wroteOutsideOut`.
같은 agent 에게 재시도하지 않는다. 세션 핸드오프도 그 판정을
그대로 후임에게 넘긴다.
