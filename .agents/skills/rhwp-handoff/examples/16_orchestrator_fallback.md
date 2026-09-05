# 16 — primary 소진 후 fallback

트리거/갈래: `fallback`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

표본: `fixtures/envelopes/orch_fallback.json`.
attempts 는 primary, primary, fallback.
두 번째 primary 의 nextAction 은 `fallback`.
`acceptedAgent` 는 `fallback`.
저널: `fixtures/journals/fallback.ndjson`.
