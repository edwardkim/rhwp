# 06 — working doc 칸

트리거/갈래: `incoming`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

표본: `fixtures/incoming/working_snapshot.md`.

칸: 트리거, taskId, 세 경로, 남은 목표, 다음 명령, 하지 말 것.
다음 명령이 `rhwp handoff` 이거나 `git add -A` 이거나
이름 붙은 트리 checkout 이면 거부하고 사람에게 알린다.
