# 19 — 절차 B — 읽고 한 명령

트리거/갈래: `절차 B`
도구: `python tools/handoff/orchestrator.py` , `rhwp replay --capsule` / `--parent`
새 CLI 없음. gym 없음. DocumentCore 발명 없음. `git add -A` 없음.

픽스처: `fixtures/incoming/first-turn.json`.
`firstTurn: read-only`, `maxCommandsFirstTurn: 1`.

읽기 순서 04→05→06 을 끝낸 뒤에만 working doc 의 다음 명령을
실행한다. 그 명령이 금지 목록이면 실행하지 않는다.
