# 24 — selfExecute 면 위임을 접는다

트리거/갈래: `selfExecute`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

표본: `fixtures/results/handoff_self_execute.json`.
후임이 `--fallback-agent` 를 새로 발명하지 않는다.
자체 실행은 기존 `rhwp edit`/`run` 한 줄.
