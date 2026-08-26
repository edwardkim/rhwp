# 22 — 수용 직후 끊기면 수거물부터

트리거/갈래: `session_interrupt`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

`result.json.outcome` 이 이미 `accepted` 다.
후임은 위임을 다시 돌리지 않는다. `collectedOutputs` 와
캡슐을 대조한 뒤 working doc 다음 명령.
트랜스크립트: `T22_interrupt_accepted.json`.
