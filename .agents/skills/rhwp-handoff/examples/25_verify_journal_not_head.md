# 25 — verifyJournal 봉투는 머리가 아니다

트리거/갈래: `pitfall 11`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

표본: `fixtures/results/verify_journal_only.json`.
`operation` 이 `agent.handoff.verifyJournal` 이면
`collectedOutputs` 가 없다. incoming 머리가 될 수 없다.
